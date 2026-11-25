use crate::clipboard::{self, ClipContent};
use crate::config::{self, Config};
use crate::files;
use crate::util;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub mime: String,
    pub added_at: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub kind: String,
    pub mime: String,
    pub added_at: u64,
}

impl Item {
    pub fn view(&self) -> ItemView {
        ItemView {
            id: self.id.clone(),
            name: self.name.clone(),
            size: self.size,
            is_dir: self.is_dir,
            kind: files::kind_for(&self.path, self.is_dir).to_string(),
            mime: self.mime.clone(),
            added_at: self.added_at,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardView {
    pub kind: String,
    pub text: Option<String>,
    pub files: Vec<ItemView>,
    pub at: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaView {
    pub active: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub app: String,
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub can_seek: bool,
    pub art: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub at: u64,
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub connected_at: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPower {
    pub action: String,
    pub execute_at: u64,
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub config: RwLock<Config>,
    pub port: AtomicU16,
    pub bind_error: Mutex<Option<String>>,
    pub items: RwLock<HashMap<String, Item>>,
    pub shares: RwLock<Vec<String>>,
    pub received: RwLock<Vec<String>>,
    pub activity: RwLock<Vec<Activity>>,
    pub clipboard: RwLock<ClipboardView>,
    pub media: RwLock<MediaView>,
    pub clip_seq: AtomicU32,
    pub token_generation: AtomicU64,
    pub devices: Mutex<HashMap<String, DeviceView>>,
    pub push: broadcast::Sender<String>,
    pub app: OnceLock<AppHandle>,
    pub started: SystemTime,
    pub power_task: Mutex<Option<JoinHandle<()>>>,
    pub pending_power: Mutex<Option<PendingPower>>,
}

impl AppState {
    pub fn new(cfg: Config, data_dir: PathBuf) -> Arc<Self> {
        let (push, _) = broadcast::channel(64);
        let port = cfg.port;
        let state = Arc::new(Self {
            data_dir,
            config: RwLock::new(cfg),
            port: AtomicU16::new(port),
            bind_error: Mutex::new(None),
            items: RwLock::new(HashMap::new()),
            shares: RwLock::new(Vec::new()),
            received: RwLock::new(Vec::new()),
            activity: RwLock::new(Vec::new()),
            clipboard: RwLock::new(ClipboardView {
                kind: "empty".into(),
                ..Default::default()
            }),
            media: RwLock::new(MediaView::default()),
            clip_seq: AtomicU32::new(0),
            token_generation: AtomicU64::new(0),
            devices: Mutex::new(HashMap::new()),
            push,
            app: OnceLock::new(),
            started: SystemTime::now(),
            power_task: Mutex::new(None),
            pending_power: Mutex::new(None),
        });
        state.cleanup_cache();
        state.scan_inbox();
        state
    }

    fn cleanup_cache(&self) {
        // Folder zips are cheap to regenerate; clipboard images stay for two days.
        let _ = std::fs::remove_dir_all(self.data_dir.join("cache").join("zips"));
        let cache = self.data_dir.join("cache");
        let Ok(entries) = std::fs::read_dir(&cache) else {
            return;
        };
        let now = SystemTime::now();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("clipboard-") {
                continue;
            }
            let old = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| now.duration_since(t).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if old > 2 * 86400 {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    pub fn attach_app(&self, handle: AppHandle) {
        let _ = self.app.set(handle);
    }

    pub fn cfg(&self) -> Config {
        self.config.read().map(|c| c.clone()).unwrap_or_default()
    }

    pub fn token(&self) -> String {
        self.cfg().token
    }

    pub fn bump_token_generation(&self) {
        self.token_generation.fetch_add(1, Ordering::SeqCst);
    }

    pub fn token_matches(&self, candidate: &str) -> bool {
        let token = self.token();
        if token.is_empty() || token.len() != candidate.len() {
            return false;
        }
        token
            .bytes()
            .zip(candidate.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }

    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    pub fn set_port(&self, port: u16) {
        self.port.store(port, Ordering::SeqCst);
    }

    pub fn lan_ip(&self) -> String {
        util::lan_ip()
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.lan_ip(), self.port())
    }

    pub fn pair_url(&self) -> String {
        format!("{}/pair?t={}", self.base_url(), self.token())
    }

    pub fn server_error(&self) -> Option<String> {
        self.bind_error.lock().ok().and_then(|e| e.clone())
    }

    pub fn set_server_error(&self, err: Option<String>) {
        if let Ok(mut slot) = self.bind_error.lock() {
            *slot = err;
        }
    }

    pub fn save_config(&self) {
        let cfg = self.cfg();
        if let Err(e) = cfg.save(&self.data_dir) {
            tracing::error!("failed to save config: {e}");
        }
    }

    pub fn mutate_config<T>(&self, f: impl FnOnce(&mut Config) -> T) -> T {
        let out = {
            let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
            f(&mut cfg)
        };
        self.save_config();
        out
    }

    pub fn emit<T: Serialize + Clone>(&self, event: &str, payload: T) {
        if let Some(app) = self.app.get() {
            if let Err(e) = app.emit(event, payload) {
                tracing::warn!("emit {event} failed: {e}");
            }
        }
    }

    pub fn refresh(&self, reason: &str) {
        self.emit("lumo-refresh", serde_json::json!({ "reason": reason }));
    }

    pub fn toast(&self, message: &str, tone: &str) {
        self.emit(
            "lumo-toast",
            serde_json::json!({ "message": message, "tone": tone }),
        );
    }

    pub fn add_activity(&self, kind: &str, title: &str, detail: &str, ok: bool) {
        let item = Activity {
            id: Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
            at: now_secs(),
            ok,
        };
        {
            let mut log = self.activity.write().unwrap_or_else(|e| e.into_inner());
            log.insert(0, item.clone());
            log.truncate(80);
        }
        self.emit("lumo-activity", item);
        self.refresh("activity");
    }

    pub fn activity_list(&self) -> Vec<Activity> {
        self.activity.read().map(|v| v.clone()).unwrap_or_default()
    }

    pub fn clear_activity(&self) {
        if let Ok(mut log) = self.activity.write() {
            log.clear();
        }
        self.refresh("activity");
    }

    pub fn device_views(&self) -> Vec<DeviceView> {
        let mut list: Vec<DeviceView> = self
            .devices
            .lock()
            .map(|d| d.values().cloned().collect())
            .unwrap_or_default();
        list.sort_by_key(|d| d.connected_at);
        list
    }

    pub fn device_count(&self) -> usize {
        self.devices.lock().map(|d| d.len()).unwrap_or(0)
    }

    pub fn add_device(&self, id: &str, name: &str, ip: &str) {
        if let Ok(mut devices) = self.devices.lock() {
            devices.insert(
                id.to_string(),
                DeviceView {
                    id: id.to_string(),
                    name: name.to_string(),
                    ip: ip.to_string(),
                    connected_at: now_secs(),
                },
            );
        }
        self.refresh("devices");
    }

    pub fn remove_device(&self, id: &str) {
        if let Ok(mut devices) = self.devices.lock() {
            devices.remove(id);
        }
        self.refresh("devices");
    }

    fn item_from_path(&self, path: &Path, at: u64) -> Option<Item> {
        let meta = std::fs::metadata(path).ok()?;
        let is_dir = meta.is_dir();
        Some(Item {
            id: Uuid::new_v4().to_string(),
            name: files::display_name(path),
            path: path.to_path_buf(),
            size: if is_dir { 0 } else { meta.len() },
            is_dir,
            mime: files::mime_for(path, is_dir),
            added_at: at,
        })
    }

    pub fn find_item(&self, id: &str) -> Option<Item> {
        self.items.read().ok().and_then(|m| m.get(id).cloned())
    }

    pub fn list_views(&self, ids: &[String]) -> Vec<ItemView> {
        let items = match self.items.read() {
            Ok(i) => i,
            Err(e) => e.into_inner(),
        };
        ids.iter()
            .filter_map(|id| items.get(id).map(|i| i.view()))
            .collect()
    }

    pub fn share_list(&self) -> Vec<ItemView> {
        let ids = self.shares.read().map(|s| s.clone()).unwrap_or_default();
        self.list_views(&ids)
    }

    pub fn received_list(&self) -> Vec<ItemView> {
        let ids = self.received.read().map(|s| s.clone()).unwrap_or_default();
        self.list_views(&ids)
    }

    pub fn add_share_paths(&self, paths: Vec<PathBuf>) -> Vec<ItemView> {
        let at = now_secs();
        let mut added: Vec<String> = Vec::new();
        {
            let mut items = self.items.write().unwrap_or_else(|e| e.into_inner());
            let mut shares = self.shares.write().unwrap_or_else(|e| e.into_inner());
            let existing: Vec<PathBuf> = shares
                .iter()
                .filter_map(|id| items.get(id))
                .map(|i| i.path.clone())
                .collect();
            for raw in paths {
                let Some(path) = files::canonical(&raw) else {
                    continue;
                };
                if existing.iter().any(|p| *p == path) {
                    continue;
                }
                if let Some(item) = self.item_from_path(&path, at) {
                    added.push(item.id.clone());
                    items.insert(item.id.clone(), item);
                    shares.push(added.last().unwrap().clone());
                }
            }
        }
        if !added.is_empty() {
            self.add_activity(
                "share",
                &format!("Shared {} item{}", added.len(), if added.len() == 1 { "" } else { "s" }),
                "Available on your phone under Files → From PC",
                true,
            );
            self.refresh("shares");
            self.push_refresh("shares");
        }
        self.list_views(&added)
    }

    pub fn remove_share(&self, id: &str) {
        let removed = {
            let mut shares = self.shares.write().unwrap_or_else(|e| e.into_inner());
            if let Some(pos) = shares.iter().position(|s| s == id) {
                shares.remove(pos);
                true
            } else {
                false
            }
        };
        if removed {
            if let Ok(mut items) = self.items.write() {
                items.remove(id);
            }
            self.refresh("shares");
            self.push_refresh("shares");
        }
    }

    pub fn clear_shares(&self) {
        let ids: Vec<String> = {
            let mut shares = self.shares.write().unwrap_or_else(|e| e.into_inner());
            let ids = shares.clone();
            shares.clear();
            ids
        };
        if let Ok(mut items) = self.items.write() {
            for id in ids {
                items.remove(&id);
            }
        }
        self.refresh("shares");
        self.push_refresh("shares");
    }

    pub fn add_received_path(&self, path: &Path) -> Option<ItemView> {
        let at = now_secs();
        let item = self.item_from_path(path, at)?;
        let view = item.view();
        {
            let mut items = self.items.write().unwrap_or_else(|e| e.into_inner());
            items.insert(item.id.clone(), item.clone());
            let mut received = self.received.write().unwrap_or_else(|e| e.into_inner());
            received.insert(0, item.id.clone());
            if received.len() > 200 {
                received.truncate(200);
            }
        }
        self.add_activity(
            "receive",
            &format!("Received {}", view.name),
            &format!("Saved to {}", path.display()),
            true,
        );
        self.refresh("received");
        self.push_refresh("received");
        Some(view)
    }

    pub fn clear_received(&self) {
        let ids: Vec<String> = {
            let mut rec = self.received.write().unwrap_or_else(|e| e.into_inner());
            let ids = rec.clone();
            rec.clear();
            ids
        };
        if let Ok(mut items) = self.items.write() {
            for id in &ids {
                items.remove(id);
            }
        }
        self.refresh("received");
        self.push_refresh("received");
    }

    fn scan_inbox(&self) {
        let dir = self.inbox_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return;
        };
        let mut found: Vec<(u64, PathBuf)> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(".upload-") || name.ends_with(".part") {
                let _ = std::fs::remove_file(entry.path());
                continue;
            }
            if files::is_hidden(&name) {
                continue;
            }
            let path = entry.path();
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            found.push((modified, path));
        }
        found.sort_by(|a, b| b.0.cmp(&a.0));
        found.truncate(100);
        let mut items = self.items.write().unwrap_or_else(|e| e.into_inner());
        let mut received = self.received.write().unwrap_or_else(|e| e.into_inner());
        for (_, path) in found {
            if let Some(item) = self.item_from_path(&path, now_secs()) {
                received.push(item.id.clone());
                items.insert(item.id.clone(), item);
            }
        }
    }

    pub fn inbox_dir(&self) -> PathBuf {
        let cfg = self.cfg();
        let dir = if cfg.inbox_dir.trim().is_empty() {
            config::default_inbox()
        } else {
            PathBuf::from(cfg.inbox_dir)
        };
        let _ = files::ensure_dir(&dir);
        dir
    }

    pub fn clipboard(&self) -> ClipboardView {
        self.clipboard
            .read()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    pub fn clipboard_sync(&self) -> bool {
        self.cfg().clipboard_sync
    }

    pub fn set_clipboard_sync(&self, on: bool) {
        self.mutate_config(|c| c.clipboard_sync = on);
        self.refresh("settings");
    }

    pub fn push_message(&self, json: String) {
        let _ = self.push.send(json);
    }

    pub fn media(&self) -> MediaView {
        self.media
            .read()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    pub fn set_media(&self, view: MediaView) {
        if let Ok(mut media) = self.media.write() {
            *media = view.clone();
        }
        let payload = serde_json::json!({ "t": "media", "media": view });
        self.push_message(payload.to_string());
    }

    pub fn push_clipboard(&self) {
        let view = self.clipboard();
        let payload = serde_json::json!({ "t": "clipboard", "clipboard": view });
        self.push_message(payload.to_string());
    }

    pub fn push_refresh(&self, scope: &str) {
        let payload = serde_json::json!({ "t": "refresh", "scope": scope });
        self.push_message(payload.to_string());
    }

    pub fn set_clipboard_content(&self, content: ClipContent, notify: bool) {
        let at = now_secs();
        let old_ids: Vec<String> = self
            .clipboard()
            .files
            .iter()
            .map(|f| f.id.clone())
            .collect();
        let mut view = ClipboardView {
            kind: "empty".into(),
            text: None,
            files: Vec::new(),
            at,
        };
        let mut new_items: Vec<Item> = Vec::new();
        match content {
            ClipContent::Empty => {}
            ClipContent::Text(text) => {
                view.kind = "text".into();
                let clipped: String = text.chars().take(200_000).collect();
                view.text = Some(clipped);
            }
            ClipContent::Files(paths) => {
                for path in paths {
                    if let Some(item) = self.item_from_path(&path, at) {
                        new_items.push(item);
                    }
                }
                if !new_items.is_empty() {
                    view.kind = "files".into();
                }
            }
            ClipContent::Image { png } => {
                if let Some(item) = self.store_clip_image(&png, at) {
                    view.kind = "image".into();
                    new_items.push(item);
                }
            }
        }
        {
            let mut items = self.items.write().unwrap_or_else(|e| e.into_inner());
            for id in &old_ids {
                items.remove(id);
            }
            for item in &new_items {
                items.insert(item.id.clone(), item.clone());
            }
        }
        view.files = new_items.iter().map(|i| i.view()).collect();
        if let Ok(mut cb) = self.clipboard.write() {
            *cb = view.clone();
        }
        if notify && self.clipboard_sync() && self.device_count() > 0 {
            self.push_clipboard();
        }
        self.refresh("clipboard");
    }

    fn store_clip_image(&self, png: &[u8], at: u64) -> Option<Item> {
        let cache = self.data_dir.join("cache");
        let _ = files::ensure_dir(&cache);
        let path = cache.join(format!("clipboard-{}.png", Uuid::new_v4()));
        std::fs::write(&path, png).ok()?;
        Some(Item {
            id: Uuid::new_v4().to_string(),
            name: "Clipboard image.png".into(),
            path,
            size: png.len() as u64,
            is_dir: false,
            mime: "image/png".into(),
            added_at: at,
        })
    }

    pub fn refresh_from_system(&self, notify: bool) {
        let content = clipboard::read();
        self.set_clipboard_content(content, notify);
        self.clip_seq.store(clipboard::sequence(), Ordering::SeqCst);
    }

    pub fn set_clipboard_text(&self, text: &str) -> Result<(), String> {
        clipboard::write_text(text)?;
        self.set_clipboard_content(ClipContent::Text(text.to_string()), true);
        self.clip_seq.store(clipboard::sequence(), Ordering::SeqCst);
        Ok(())
    }

    pub fn set_clipboard_files(&self, paths: Vec<PathBuf>) -> Result<(), String> {
        clipboard::write_files(&paths)?;
        self.set_clipboard_content(ClipContent::Files(paths), true);
        self.clip_seq.store(clipboard::sequence(), Ordering::SeqCst);
        Ok(())
    }

    pub fn set_clipboard_image(&self, png: &[u8]) -> Result<(), String> {
        clipboard::write_image_png(png)?;
        self.set_clipboard_content(ClipContent::Image { png: png.to_vec() }, true);
        self.clip_seq.store(clipboard::sequence(), Ordering::SeqCst);
        Ok(())
    }

    pub fn copy_item_to_clipboard(&self, id: &str) -> Result<(), String> {
        let item = self.find_item(id).ok_or("item not found")?;
        self.set_clipboard_files(vec![item.path])
    }

    pub fn send_clipboard_to_phone(&self) -> ClipboardView {
        self.refresh_from_system(false);
        self.push_clipboard();
        self.clipboard()
    }

    pub fn schedule_power(self: &Arc<Self>, action: &str, seconds: u64) {
        self.cancel_power(false);
        let state = self.clone();
        let action_owned = action.to_string();
        let pending = PendingPower {
            action: action_owned.clone(),
            execute_at: now_secs() + seconds,
        };
        if let Ok(mut slot) = self.pending_power.lock() {
            *slot = Some(pending.clone());
        }
        tracing::info!("power: scheduled {action_owned} in {seconds}s");
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(seconds)).await;
            tracing::info!("power: timer fired for {action_owned}");
            if let Ok(mut slot) = state.pending_power.lock() {
                *slot = None;
            }
            let act = action_owned.clone();
            let result = tokio::task::spawn_blocking(move || crate::power::execute(&act)).await;
            let (ok, detail) = match result {
                Ok(Ok(())) => (true, String::new()),
                Ok(Err(e)) => (false, e),
                Err(e) => (false, e.to_string()),
            };
            if !ok {
                state.push_message(
                    serde_json::json!({
                        "t": "toast",
                        "message": format!("{} failed: {detail}", capitalize(&action_owned)),
                        "tone": "error",
                    })
                    .to_string(),
                );
            }
            state.add_activity(
                "power",
                &format!("{} {}", capitalize(&action_owned), if ok { "sent" } else { "failed" }),
                &detail,
                ok,
            );
            state.refresh("power");
        });
        if let Ok(mut slot) = self.power_task.lock() {
            *slot = Some(handle);
        }
        self.add_activity(
            "power",
            &format!("{} scheduled", capitalize(action)),
            &format!("Running in {seconds} seconds"),
            true,
        );
    }

    pub fn cancel_power(&self, announce: bool) -> bool {
        let had = self
            .pending_power
            .lock()
            .map(|p| p.is_some())
            .unwrap_or(false);
        if let Ok(mut task) = self.power_task.lock() {
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }
        if let Ok(mut slot) = self.pending_power.lock() {
            *slot = None;
        }
        if had && announce {
            self.add_activity("power", "Cancelled", "Scheduled action aborted", true);
        }
        if had {
            self.refresh("power");
        }
        had
    }

    pub fn pending_power(&self) -> Option<PendingPower> {
        self.pending_power.lock().ok().and_then(|p| p.clone())
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
