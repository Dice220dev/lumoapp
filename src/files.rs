use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
    Ok(())
}

pub fn sanitize_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let trimmed = out.trim().trim_matches('.').trim();
    let mut s: String = trimmed.chars().take(120).collect();
    if s.is_empty() {
        s = "file".into();
    }
    s
}

/// Turn an untrusted relative path into a safe path below a base directory.
pub fn safe_relative(raw: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    let mut depth = 0usize;
    for part in raw.replace('\\', "/").split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return None;
        }
        let clean = sanitize_component(part);
        if clean.is_empty() {
            continue;
        }
        out.push(clean);
        depth += 1;
        if depth > 8 {
            return None;
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let safe = sanitize_component(name);
    let path = dir.join(&safe);
    if !path.exists() {
        return path;
    }
    let p = Path::new(&safe);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".into());
    let ext = p
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    for n in 2..10000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem}-{}{ext}", uuid::Uuid::new_v4()))
}

pub fn mime_for(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "inode/directory".into();
    }
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}

pub fn kind_for(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return "folder";
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" | "heif" | "bmp" | "svg" | "avif" => "image",
        "mp4" | "mov" | "m4v" | "webm" | "mkv" | "avi" => "video",
        "mp3" | "m4a" | "wav" | "aac" | "flac" | "ogg" => "audio",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "archive",
        "txt" | "md" | "log" | "csv" | "json" | "xml" | "yaml" | "yml" | "toml" | "ini" => "text",
        "pdf" => "pdf",
        "doc" | "docx" | "odt" | "rtf" => "document",
        "xls" | "xlsx" | "ods" => "sheet",
        "ppt" | "pptx" | "odp" => "slides",
        _ => "file",
    }
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

/// Zip a directory (or a single file) into `dest`. Blocking; call from a worker thread.
pub fn zip_into(src: &Path, dest: &Path) -> Result<u64> {
    if let Some(parent) = dest.parent() {
        ensure_dir(parent)?;
    }
    let file = BufWriter::new(File::create(dest).context("create zip")?);
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let dir_opts = zip::write::SimpleFileOptions::default();

    if src.is_file() {
        let name = display_name(src);
        zip.start_file(name, opts)?;
        let mut f = File::open(src)?;
        std::io::copy(&mut f, &mut zip)?;
    } else {
        let base_name = display_name(src);
        zip.add_directory(format!("{base_name}/"), dir_opts)?;
        for entry in WalkDir::new(src).follow_links(false).sort_by_file_name() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path == src {
                continue;
            }
            let rel = match path.strip_prefix(src) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut name = format!("{base_name}/");
            name.push_str(&rel.to_string_lossy().replace('\\', "/"));
            if entry.file_type().is_dir() {
                zip.add_directory(name, dir_opts)?;
            } else if entry.file_type().is_file() {
                zip.start_file(&name, opts)?;
                let mut f = BufReader::new(File::open(path)?);
                std::io::copy(&mut f, &mut zip)?;
            }
        }
    }
    zip.finish()?;
    Ok(fs::metadata(dest).map(|m| m.len()).unwrap_or(0))
}

/// Extract a zip archive into `<dest>/<stem>` guarding against path traversal.
pub fn extract_zip(zip_path: &Path, dest: &Path) -> Result<PathBuf> {
    let stem = zip_path
        .file_stem()
        .map(|s| sanitize_component(&s.to_string_lossy()))
        .unwrap_or_else(|| "imported".into());
    let root = unique_path(dest, &stem);
    ensure_dir(&root)?;
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    if archive.len() > 20_000 {
        return Err(anyhow!("archive has too many entries"));
    }
    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };
        let out = root.join(&rel);
        if !out.starts_with(&root) {
            continue;
        }
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        total = total.saturating_add(entry.size());
        if total > 8 * 1024 * 1024 * 1024 {
            return Err(anyhow!("archive expands beyond 8 GB"));
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = BufWriter::new(File::create(&out)?);
        std::io::copy(&mut entry, &mut f)?;
        f.flush()?;
    }
    Ok(root)
}

/// Normalize a user supplied path (from a native picker) into a canonical path.
pub fn canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok().filter(|p| p.exists())
}

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name.eq_ignore_ascii_case("desktop.ini")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_traversal() {
        assert!(safe_relative("../secret.txt").is_none());
        assert!(safe_relative("a/../../secret.txt").is_none());
        assert!(safe_relative("a/../secret.txt").is_none());
    }

    #[test]
    fn keeps_safe_relative_paths() {
        assert_eq!(
            safe_relative("photos/2026/img.jpg").unwrap(),
            PathBuf::from("photos").join("2026").join("img.jpg")
        );
        assert!(safe_relative("").is_none());
    }

    #[test]
    fn sanitizes_windows_forbidden_characters() {
        assert_eq!(sanitize_component("a:b?.txt"), "a_b_.txt");
        assert_eq!(sanitize_component(".."), "file");
        assert_eq!(sanitize_component("  report  "), "report");
    }
}
