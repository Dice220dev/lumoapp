use crate::autostart;
use crate::clipboard::ClipContent;
use crate::config::{self, Config, APP_VERSION};
use crate::files;
use crate::input;
use crate::server;
use crate::state::{
    now_secs, Activity, AppState, ClipboardView, DeviceView, ItemView, MediaView, PendingPower,
};
use crate::util;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub version: String,
    pub device_name: String,
    pub port: u16,
    pub lan_ip: String,
    pub base_url: String,
    pub pair_url: String,
    pub qr_svg: String,
    pub addresses: Vec<String>,
    pub autostart: bool,
    pub start_minimized: bool,
    pub clipboard_sync: bool,
    pub theme: String,
    pub inbox_dir: String,
    pub data_dir: String,
    pub log_path: String,
    pub connected: Vec<DeviceView>,
    pub shares: Vec<ItemView>,
    pub received: Vec<ItemView>,
    pub clipboard: ClipboardView,
    pub media: MediaView,
    pub activity: Vec<Activity>,
    pub pending_power: Option<PendingPower>,
    pub server_error: Option<String>,
    pub platform: String,
    pub uptime_secs: u64,
    pub server_time: u64,
    pub configured_port: u16,
}

fn build_bootstrap(state: &Arc<AppState>) -> Bootstrap {
    let cfg = state.cfg();
    let pair_url = state.pair_url();
    let qr_svg = util::qr_svg(&pair_url, 240).unwrap_or_default();
    let port = state.port();
    let addresses = util::lan_ips()
        .into_iter()
        .map(|ip| format!("http://{ip}:{port}"))
        .collect();
    Bootstrap {
        version: APP_VERSION.into(),
        device_name: cfg.device_name,
        port,
        lan_ip: state.lan_ip(),
        base_url: state.base_url(),
        pair_url,
        qr_svg,
        addresses,
        autostart: autostart::enabled(),
        start_minimized: cfg.start_minimized,
        clipboard_sync: state.clipboard_sync(),
        theme: cfg.theme,
        inbox_dir: state.inbox_dir().to_string_lossy().to_string(),
        data_dir: state.data_dir.to_string_lossy().to_string(),
        log_path: state
            .data_dir
            .join("logs")
            .join("lumo.log")
            .to_string_lossy()
            .to_string(),
        connected: state.device_views(),
        shares: state.share_list(),
        received: state.received_list(),
        clipboard: state.clipboard(),
        media: state.media(),
        activity: state.activity_list(),
        pending_power: state.pending_power(),
        server_error: state.server_error(),
        platform: std::env::consts::OS.into(),
        uptime_secs: state
            .started
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0),
        server_time: now_secs(),
        configured_port: cfg.port,
    }
}

#[tauri::command]
fn bootstrap(state: State<'_, Arc<AppState>>) -> Bootstrap {
    build_bootstrap(state.inner())
}

#[tauri::command]
async fn pick_and_share(
    state: State<'_, Arc<AppState>>,
    kind: String,
) -> Result<Vec<ItemView>, String> {
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || {
        let dialog = rfd::FileDialog::new().set_title(if kind == "folder" {
            "Share a folder with your phone"
        } else {
            "Share files with your phone"
        });
        let paths: Vec<PathBuf> = if kind == "folder" {
            dialog.pick_folder().into_iter().collect()
        } else {
            dialog.pick_files().unwrap_or_default()
        };
        if paths.is_empty() {
            return Vec::new();
        }
        state.add_share_paths(paths)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_share_paths(
    state: State<'_, Arc<AppState>>,
    paths: Vec<String>,
) -> Result<Vec<ItemView>, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    Ok(state.add_share_paths(paths))
}

#[tauri::command]
fn remove_share(state: State<'_, Arc<AppState>>, id: String) {
    state.remove_share(&id);
}

#[tauri::command]
fn clear_shares(state: State<'_, Arc<AppState>>) {
    state.clear_shares();
}

#[tauri::command]
fn clear_received(state: State<'_, Arc<AppState>>) {
    state.clear_received();
}

#[tauri::command]
fn clear_activity(state: State<'_, Arc<AppState>>) {
    state.clear_activity();
}

#[tauri::command]
fn open_item(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    let item = state.find_item(&id).ok_or("item not found")?;
    util::open_path(&item.path)
}

#[tauri::command]
fn reveal_item(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    let item = state.find_item(&id).ok_or("item not found")?;
    util::reveal_path(&item.path)
}

#[tauri::command]
fn copy_item_to_clipboard(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    state.copy_item_to_clipboard(&id)
}

#[tauri::command]
fn item_preview(state: State<'_, Arc<AppState>>, id: String) -> Result<String, String> {
    use base64::Engine;
    let item = state.find_item(&id).ok_or("item not found")?;
    if item.is_dir {
        return Err("folders have no preview".into());
    }
    if item.size > 8 * 1024 * 1024 {
        return Err("preview is too large".into());
    }
    if !item.mime.starts_with("image/") {
        return Err("not an image".into());
    }
    let bytes = std::fs::read(&item.path).map_err(|e| e.to_string())?;
    Ok(format!(
        "data:{};base64,{}",
        item.mime,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[tauri::command]
fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    crate::clipboard::write_text(&text)
}

#[tauri::command]
fn send_clipboard_to_phone(state: State<'_, Arc<AppState>>) -> ClipboardView {
    let view = state.send_clipboard_to_phone();
    state.toast("Clipboard sent to your phone", "success");
    view
}

#[tauri::command]
fn refresh_clipboard(state: State<'_, Arc<AppState>>) -> ClipboardView {
    state.refresh_from_system(false);
    state.clipboard()
}

#[tauri::command]
fn send_text_to_phone(state: State<'_, Arc<AppState>>, text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Type something to send first".into());
    }
    if state.device_count() == 0 {
        return Err("No phone is connected right now".into());
    }
    state.push_message(serde_json::json!({ "t": "shared-text", "text": text }).to_string());
    state.add_activity("clipboard", "Text sent to phone", "", true);
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    port: Option<u16>,
    autostart: Option<bool>,
    start_minimized: Option<bool>,
    clipboard_sync: Option<bool>,
    theme: Option<String>,
    device_name: Option<String>,
    inbox_dir: Option<String>,
}

#[tauri::command]
fn update_settings(
    state: State<'_, Arc<AppState>>,
    patch: SettingsPatch,
) -> Result<serde_json::Value, String> {
    let mut restart_required = false;
    let mut cfg = state.cfg();
    if let Some(port) = patch.port {
        if !(1024..=65535).contains(&port) {
            return Err("port must be between 1024 and 65535".into());
        }
        if port != cfg.port {
            cfg.port = port;
            restart_required = true;
        }
    }
    if let Some(on) = patch.autostart {
        autostart::set(on, cfg.start_minimized).map_err(|e| e.to_string())?;
        cfg.autostart = on;
    }
    if let Some(minimized) = patch.start_minimized {
        cfg.start_minimized = minimized;
        if cfg.autostart {
            autostart::set(true, minimized).map_err(|e| e.to_string())?;
        }
    }
    if let Some(on) = patch.clipboard_sync {
        cfg.clipboard_sync = on;
    }
    if let Some(theme) = patch.theme {
        if !matches!(theme.as_str(), "system" | "light" | "dark") {
            return Err("unknown theme".into());
        }
        cfg.theme = theme;
    }
    if let Some(name) = patch.device_name {
        let name = name.trim().chars().take(60).collect::<String>();
        if !name.is_empty() {
            cfg.device_name = name;
        }
    }
    if let Some(dir) = patch.inbox_dir {
        let path = PathBuf::from(&dir);
        files::ensure_dir(&path).map_err(|e| e.to_string())?;
        cfg.inbox_dir = dir;
    }
    {
        let mut current = state.config.write().map_err(|e| e.to_string())?;
        *current = cfg;
    }
    state.save_config();
    state.refresh("settings");
    Ok(serde_json::json!({ "ok": true, "restartRequired": restart_required }))
}

#[tauri::command]
fn regenerate_token(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let token = config::new_token();
    state.mutate_config(|c| c.token = token);
    state.bump_token_generation();
    state.push_message(serde_json::json!({ "t": "session-expired" }).to_string());
    state.add_activity(
        "settings",
        "Pairing link reset",
        "Previously paired phones must pair again",
        true,
    );
    state.refresh("settings");
    Ok(state.pair_url())
}

#[tauri::command]
fn relaunch(app: AppHandle) {
    // The new process must start after this one releases the single-instance mutex
    // and the listening port, otherwise it exits immediately (or binds another port).
    if let Ok(exe) = std::env::current_exe() {
        let escaped = exe.to_string_lossy().replace('\'', "''");
        let script = format!("Start-Sleep -Milliseconds 1400; Start-Process -FilePath '{escaped}'");
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .creation_flags(0x08000000)
            .spawn();
    }
    app.exit(0);
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    util::open_path(std::path::Path::new(&path))
}

#[tauri::command]
async fn pick_inbox_dir() -> Option<String> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choose where received files are saved")
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
fn send_text(text: String) {
    let _ = input::type_text(&text);
}

pub fn run() {
    let data_dir = config::data_dir();
    init_logging(&data_dir);
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("panic: {info}");
        previous_hook(info);
    }));
    let minimized = std::env::args().any(|a| a == "--minimized");
    let cfg = Config::load(&data_dir);
    let state = AppState::new(cfg.clone(), data_dir.clone());
    let state_for_setup = state.clone();
    let state_for_window = state.clone();

    let (listener, port) = match server::bind(cfg.port) {
        Ok((listener, port)) => {
            state.set_port(port);
            (Some(listener), port)
        }
        Err(e) => {
            tracing::error!("could not bind local server: {e}");
            state.set_server_error(Some(format!("Could not start the local server: {e}")));
            (None, cfg.port)
        }
    };
    if port != cfg.port {
        tracing::warn!("port {} busy, using {}", cfg.port, port);
    }

    tracing::info!(
        "autostart: enabled={} command={:?}",
        autostart::enabled(),
        autostart::command()
    );
    if cfg.autostart {
        match autostart::set(true, cfg.start_minimized) {
            Ok(()) => tracing::info!("autostart: registry re-applied"),
            Err(e) => tracing::warn!("autostart: could not re-apply registry entry: {e}"),
        }
    }

    let result = tauri::Builder::default()
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            pick_and_share,
            add_share_paths,
            remove_share,
            clear_shares,
            clear_received,
            clear_activity,
            open_item,
            reveal_item,
            copy_item_to_clipboard,
            item_preview,
            copy_text_to_clipboard,
            send_clipboard_to_phone,
            refresh_clipboard,
            send_text_to_phone,
            update_settings,
            regenerate_token,
            relaunch,
            quit_app,
            open_path,
            pick_inbox_dir,
            send_text
        ])
        .setup(move |app| {
            tracing::info!("setup: attach");
            state_for_setup.attach_app(app.handle().clone());
            tracing::info!("setup: tray");
            if let Err(e) = setup_tray(app, state_for_setup.clone()) {
                tracing::error!("tray unavailable: {e}");
            } else {
                tracing::info!("setup: tray ok");
            }
            // Start the LAN server before the WebView, so the phone never waits for
            // WebView2 initialization (or a failed window) to finish.
            if let Some(listener) = listener {
                let server_state = state_for_setup.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = server::serve(server_state.clone(), listener).await {
                        tracing::error!("server stopped: {e}");
                        server_state.set_server_error(Some(format!("Server stopped: {e}")));
                        server_state.refresh("server");
                    }
                });
            }
            spawn_clipboard_monitor(state_for_setup.clone());
            crate::media::start(state_for_setup.clone());
            match create_main_window(app.handle(), &state_for_setup) {
                Ok(window) => {
                    tracing::info!("setup: window created");
                    if !minimized {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                Err(e) => {
                    // A tray-only session is still useful (the phone UI keeps working).
                    tracing::error!("could not create the desktop window: {e}");
                }
            }
            Ok(())
        })
        .on_window_event(move |window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) => {
                if !paths.is_empty() {
                    state_for_window.add_share_paths(paths.clone());
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("fatal: {e}");
        eprintln!("Lumo failed to start: {e}");
    }
}

pub fn init_logging(data_dir: &std::path::Path) {
    let log_dir = data_dir.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("lumo=info,warn"));
    // A read-only or sandboxed profile must never prevent Lumo from starting.
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("lumo.log"))
    {
        Ok(file) => {
            let (writer, guard) = tracing_appender::non_blocking(file);
            static LOG_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> =
                std::sync::OnceLock::new();
            let _ = LOG_GUARD.set(guard);
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(false)
                .try_init();
        }
        Err(e) => {
            eprintln!("Lumo: file logging unavailable ({e}); using stderr");
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .try_init();
        }
    }
}

fn spawn_clipboard_monitor(state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(600)).await;
        let initial = tokio::task::spawn_blocking(crate::clipboard::read)
            .await
            .unwrap_or(ClipContent::Empty);
        state.clip_seq.store(crate::clipboard::sequence(), Ordering::SeqCst);
        state.set_clipboard_content(initial, false);
        loop {
            tokio::time::sleep(Duration::from_millis(900)).await;
            let seq = crate::clipboard::sequence();
            if seq == state.clip_seq.load(Ordering::SeqCst) {
                continue;
            }
            state.clip_seq.store(seq, Ordering::SeqCst);
            let content = tokio::task::spawn_blocking(crate::clipboard::read)
                .await
                .unwrap_or(ClipContent::Empty);
            state.set_clipboard_content(content, true);
        }
    });
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let state = app.state::<Arc<AppState>>().inner().clone();
    if let Ok(window) = create_main_window(app, &state) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn create_main_window(app: &AppHandle, state: &Arc<AppState>) -> tauri::Result<WebviewWindow> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| state.data_dir.clone())
        .join("webview");
    let _ = std::fs::create_dir_all(&data_dir);
    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Lumo")
        .inner_size(1180.0, 800.0)
        .min_inner_size(960.0, 620.0)
        .center()
        .visible(false)
        .data_directory(data_dir)
        .build()
}

fn setup_tray(app: &mut tauri::App, state: Arc<AppState>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Open Lumo", true, None::<&str>)?;
    let copy_link = MenuItem::with_id(app, "copy_link", "Copy phone link", true, None::<&str>)?;
    let sync = CheckMenuItem::with_id(
        app,
        "clipboard_sync",
        "Clipboard sync",
        true,
        state.clipboard_sync(),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Lumo", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &copy_link,
            &PredefinedMenuItem::separator(app)?,
            &sync,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let tooltip = if state.server_error().is_some() {
        "Lumo — Server stopped".to_string()
    } else {
        format!("Lumo — Server running on port {}", state.port())
    };
    let mut builder = TrayIconBuilder::with_id("lumo")
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let state = app.state::<Arc<AppState>>().inner().clone();
            match event.id.as_ref() {
                "show" => show_main(app),
                "copy_link" => {
                    let _ = crate::clipboard::write_text(&state.pair_url());
                    state.toast("Phone link copied to clipboard", "success");
                }
                "clipboard_sync" => {
                    let next = !state.clipboard_sync();
                    state.set_clipboard_sync(next);
                    let _ = sync.set_checked(next);
                    state.toast(
                        if next {
                            "Clipboard sync on"
                        } else {
                            "Clipboard sync off"
                        },
                        "neutral",
                    );
                }
                "quit" => app.exit(0),
                _ => {}
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
