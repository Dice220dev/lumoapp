use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

#[derive(Clone, Debug)]
pub enum ClipContent {
    Empty,
    Text(String),
    Files(Vec<PathBuf>),
    Image { png: Vec<u8> },
}

fn open() -> Option<Clipboard> {
    for _ in 0..8 {
        if let Ok(cb) = Clipboard::new() {
            return Some(cb);
        }
        sleep(Duration::from_millis(25));
    }
    None
}

pub fn sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

pub fn read() -> ClipContent {
    for attempt in 0..4 {
        let Some(mut cb) = open() else {
            sleep(Duration::from_millis(40));
            continue;
        };
        if let Ok(files) = cb.get().file_list() {
            let existing: Vec<PathBuf> = files.into_iter().filter(|p| p.exists()).collect();
            if !existing.is_empty() {
                return ClipContent::Files(existing);
            }
        }
        if let Ok(img) = cb.get().image() {
            if let (Ok(w), Ok(h)) = (u32::try_from(img.width), u32::try_from(img.height)) {
                if let Some(rgba) = image::RgbaImage::from_raw(w, h, img.bytes.into_owned()) {
                    let mut png = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut png);
                    if image::DynamicImage::ImageRgba8(rgba)
                        .write_to(&mut cursor, image::ImageFormat::Png)
                        .is_ok()
                        && !png.is_empty()
                    {
                        return ClipContent::Image { png };
                    }
                }
            }
        }
        if let Ok(text) = cb.get().text() {
            let text = text.trim_end_matches('\u{0}').to_string();
            if !text.is_empty() {
                return ClipContent::Text(text);
            }
        }
        if attempt == 3 {
            return ClipContent::Empty;
        }
        sleep(Duration::from_millis(30));
    }
    ClipContent::Empty
}

pub fn write_text(text: &str) -> Result<(), String> {
    for attempt in 0..6 {
        if let Some(mut cb) = open() {
            match cb.set().text(text.to_string()) {
                Ok(()) => return Ok(()),
                Err(e) if attempt == 5 => return Err(format!("clipboard busy: {e}")),
                Err(_) => sleep(Duration::from_millis(35)),
            }
        }
    }
    Err("clipboard unavailable".into())
}

pub fn write_files(paths: &[PathBuf]) -> Result<(), String> {
    let existing: Vec<PathBuf> = paths.iter().filter(|p| p.exists()).cloned().collect();
    if existing.is_empty() {
        return Err("no files to copy".into());
    }
    for attempt in 0..6 {
        if let Some(mut cb) = open() {
            match cb.set().file_list(&existing) {
                Ok(()) => return Ok(()),
                Err(e) if attempt == 5 => return Err(format!("clipboard busy: {e}")),
                Err(_) => sleep(Duration::from_millis(35)),
            }
        }
    }
    Err("clipboard unavailable".into())
}

pub fn write_image_png(png: &[u8]) -> Result<(), String> {
    let decoded = image::load_from_memory(png).map_err(|e| format!("image: {e}"))?;
    let rgba = decoded.to_rgba8();
    let (w, h) = rgba.dimensions();
    let data = ImageData {
        width: w as usize,
        height: h as usize,
        bytes: Cow::Owned(rgba.into_raw()),
    };
    for attempt in 0..6 {
        if let Some(mut cb) = open() {
            match cb.set().image(data.clone()) {
                Ok(()) => return Ok(()),
                Err(e) if attempt == 5 => return Err(format!("clipboard busy: {e}")),
                Err(_) => sleep(Duration::from_millis(35)),
            }
        }
    }
    Err("clipboard unavailable".into())
}


