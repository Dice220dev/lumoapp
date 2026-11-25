use crate::config::{APP_NAME, APP_VERSION};
use crate::files;
use crate::input;
use crate::state::{now_secs, AppState};
use crate::util;
use axum::body::{Body, Bytes};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, DefaultBodyLimit, Path as AxPath, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio_util::io::ReaderStream;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/"]
struct Assets;

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "ok": false, "error": self.message })),
        )
            .into_response()
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(StatusCode::BAD_REQUEST, value.to_string())
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    let public = Router::new()
        .route("/", get(index))
        .route("/pair", get(pair))
        .route("/health", get(health))
        .route("/{*path}", get(static_file));

    let protected = Router::new()
        .route("/api/bootstrap", get(api_bootstrap))
        .route("/api/shares", get(api_shares))
        .route("/api/received", get(api_received))
        .route(
            "/api/clipboard",
            get(api_clipboard_get).post(api_clipboard_set),
        )
        .route(
            "/api/upload",
            put(api_upload).layer(DefaultBodyLimit::disable()),
        )
        .route("/api/dl/{id}", get(api_download))
        .route("/api/power", post(api_power))
        .route("/api/text", post(api_text))
        .route("/api/unpair", post(api_unpair))
        .route("/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), require_token));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}

/// Bind the first free port at or after `preferred`.
pub fn bind(preferred: u16) -> std::io::Result<(std::net::TcpListener, u16)> {
    for offset in 0..20u16 {
        let port = preferred.saturating_add(offset);
        if let Ok(listener) = std::net::TcpListener::bind(("0.0.0.0", port)) {
            listener.set_nonblocking(true)?;
            return Ok((listener, port));
        }
    }
    let listener = std::net::TcpListener::bind(("0.0.0.0", 0))?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

pub async fn serve(state: Arc<AppState>, listener: std::net::TcpListener) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)?;
    tracing::info!("Lumo serving on {}", state.base_url());
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn index() -> Response {
    match Assets::get("mobile.html") {
        Some(file) => html_response(file.data.into_owned()),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "UI assets are missing").into_response(),
    }
}

async fn pair(State(state): State<Arc<AppState>>, Query(q): Query<HashMap<String, String>>) -> Response {
    let token = q.get("t").cloned().unwrap_or_default();
    if !state.token_matches(&token) {
        return (
            StatusCode::UNAUTHORIZED,
            Html(
                "<!doctype html><meta name=viewport content=\"width=device-width,initial-scale=1\">\
                 <title>Lumo · pairing expired</title>\
                 <body style=\"font-family:system-ui;background:#000000;color:#E8F0F7;padding:32px;line-height:1.5\">\
                 <h1 style=\"font-size:22px\">Pairing link expired</h1>\
                 <p style=\"color:#9AA7B4\">Open Lumo on your PC and scan the fresh QR code, or regenerate the pairing link.</p>\
                 </body>",
            ),
        )
            .into_response();
    }
    let cookie = format!(
        "lumo_auth={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000"
    );
    // Keep the token in the URL as well: home-screen apps may have a separate
    // cookie store, and the phone client mirrors it into localStorage as a fallback.
    let mut response = Redirect::to(&format!("/?t={token}")).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "name": APP_NAME,
        "version": APP_VERSION,
        "device": state.cfg().device_name,
        "port": state.port(),
    }))
}

async fn static_file(AxPath(path): AxPath<String>) -> Response {
    serve_asset(&path)
}

fn serve_asset(path: &str) -> Response {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return index_sync();
    }
    if path == "index.html" || path.starts_with("desktop") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    let lookup = if path == "manifest.webmanifest" {
        "manifest.webmanifest"
    } else {
        path
    };
    let Some(file) = Assets::get(lookup) else {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    };
    let mime = if lookup.ends_with(".webmanifest") {
        "application/manifest+json".to_string()
    } else {
        mime_guess::from_path(lookup)
            .first_or_octet_stream()
            .essence_str()
            .to_string()
    };
    let mut res = Response::new(Body::from(file.data.into_owned()));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    res.headers_mut()
        .insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    res
}

fn index_sync() -> Response {
    match Assets::get("mobile.html") {
        Some(file) => html_response(file.data.into_owned()),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "UI assets are missing").into_response(),
    }
}

fn html_response(data: Vec<u8>) -> Response {
    let mut res = Response::new(Body::from(data));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    res.headers_mut()
        .insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    res
}

async fn require_token(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let token = extract_token(req.headers(), req.uri());
    match token {
        Some(t) if state.token_matches(&t) => next.run(req).await,
        _ => ApiError::new(StatusCode::UNAUTHORIZED, "pairing required").into_response(),
    }
}

fn extract_token(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    if let Some(value) = headers.get("x-lumo-token").and_then(|v| v.to_str().ok()) {
        if !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }
    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            if k == "t" {
                return Some(util::decode_header(v));
            }
        }
    }
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for part in cookie.split(';') {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("lumo_auth=") {
                return Some(rest.to_string());
            }
        }
    }
    None
}

async fn api_bootstrap(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "deviceName": state.cfg().device_name,
        "version": APP_VERSION,
        "clipboardSync": state.clipboard_sync(),
        "shares": state.share_list(),
        "clipboard": state.clipboard(),
        "media": state.media(),
        "receivedCount": state.received_list().len(),
        "pendingPower": state.pending_power(),
        "serverTime": now_secs(),
    }))
}

async fn api_shares(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({ "items": state.share_list() }))
}

async fn api_received(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({ "items": state.received_list() }))
}

async fn api_clipboard_get(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({ "clipboard": state.clipboard() }))
}

#[derive(Deserialize)]
struct SetClipboardReq {
    text: String,
}

async fn api_clipboard_set(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetClipboardReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .set_clipboard_text(&req.text)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "ok": true, "clipboard": state.clipboard() })))
}

async fn api_upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Body,
) -> Result<Json<serde_json::Value>, ApiError> {
    let raw_name = headers
        .get("x-lumo-name")
        .and_then(|v| v.to_str().ok())
        .map(util::decode_header)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "file".into());
    let rel_raw = headers
        .get("x-lumo-rel")
        .and_then(|v| v.to_str().ok())
        .map(util::decode_header)
        .unwrap_or_default();
    let rel = if rel_raw.is_empty() {
        files::safe_relative(&raw_name)
            .unwrap_or_else(|| PathBuf::from(files::sanitize_component(&raw_name)))
    } else {
        files::safe_relative(&rel_raw)
            .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "invalid relative path"))?
    };
    let file_name = rel
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| files::sanitize_component(&raw_name));
    let parent_rel = rel.parent().unwrap_or(Path::new(""));
    let inbox = state.inbox_dir();
    let dest_dir = inbox.join(parent_rel);
    files::ensure_dir(&dest_dir)?;

    let target = files::unique_path(&dest_dir, &file_name);
    let tmp = dest_dir.join(format!(".upload-{}.part", uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut written: u64 = 0;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk: Bytes = chunk.map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
        file.write_all(&chunk).await?;
        written += chunk.len() as u64;
    }
    file.flush().await?;
    drop(file);
    tokio::fs::rename(&tmp, &target).await?;

    let mut final_path = target.clone();
    let is_zip = target
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false);
    if is_zip && q.get("extract").map(|v| v == "1").unwrap_or(false) {
        let zip_path = target.clone();
        let dest = inbox.clone();
        let extracted = tokio::task::spawn_blocking(move || files::extract_zip(&zip_path, &dest))
            .await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))??;
        let _ = tokio::fs::remove_file(&target).await;
        final_path = extracted;
    }

    let view = state
        .add_received_path(&final_path)
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not record upload"))?;

    if q.get("clipboard").map(|v| v == "1").unwrap_or(false) && final_path.is_file() {
        if let Some(mime) = view.mime.strip_prefix("image/") {
            let _ = mime;
            if let Ok(bytes) = tokio::fs::read(&final_path).await {
                let _ = state.set_clipboard_image(&bytes);
            }
        } else {
            let _ = state.set_clipboard_files(vec![final_path.clone()]);
        }
    }

    Ok(Json(json!({ "ok": true, "item": view, "bytes": written })))
}

async fn api_download(
    State(state): State<Arc<AppState>>,
    AxPath(id): AxPath<String>,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let Some(item) = state.find_item(&id) else {
        return (StatusCode::NOT_FOUND, "Unknown item").into_response();
    };
    let inline = q.get("inline").map(|v| v == "1").unwrap_or(false);
    if item.is_dir {
        let cache = state.data_dir.join("cache").join("zips");
        let _ = files::ensure_dir(&cache);
        let zip_path = cache.join(format!("{id}.zip"));
        if !zip_path.exists() {
            let src = item.path.clone();
            let dst = zip_path.clone();
            let result = tokio::task::spawn_blocking(move || files::zip_into(&src, &dst)).await;
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
                Err(e) => return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        let name = format!("{}.zip", item.name);
        return stream_file(
            &zip_path,
            &name,
            "application/zip",
            false,
            headers.get(header::RANGE).and_then(|v| v.to_str().ok()),
        )
        .await;
    }
    stream_file(
        &item.path,
        &item.name,
        &item.mime,
        inline,
        headers.get(header::RANGE).and_then(|v| v.to_str().ok()),
    )
    .await
}

fn parse_range(raw: &str, len: u64) -> Option<(u64, u64)> {
    let spec = raw.strip_prefix("bytes=")?;
    if spec.contains(',') || len == 0 {
        return None;
    }
    let (a, b) = spec.split_once('-')?;
    let (start, end) = if a.trim().is_empty() {
        let suffix: u64 = b.trim().parse().ok()?;
        if suffix == 0 {
            return None;
        }
        (len.saturating_sub(suffix), len - 1)
    } else {
        let start: u64 = a.trim().parse().ok()?;
        if start >= len {
            return None;
        }
        let end = if b.trim().is_empty() {
            len - 1
        } else {
            b.trim().parse::<u64>().ok()?.min(len - 1)
        };
        if end < start {
            return None;
        }
        (start, end)
    };
    Some((start, end))
}

async fn stream_file(
    path: &Path,
    name: &str,
    mime: &str,
    inline: bool,
    range: Option<&str>,
) -> Response {
    let Ok(mut file) = tokio::fs::File::open(path).await else {
        return (StatusCode::NOT_FOUND, "File is gone").into_response();
    };
    let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    let disposition = util::content_disposition(name, inline);
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");

    if let Some(raw) = range {
        match parse_range(raw, len) {
            Some((start, end)) => {
                let count = end - start + 1;
                if file.seek(SeekFrom::Start(start)).await.is_err() {
                    return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "seek failed").into_response();
                }
                let reader = file.take(count);
                let stream = ReaderStream::new(reader);
                builder = builder
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
                    .header(header::CONTENT_LENGTH, count);
                return builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream error").into_response());
            }
            None => {
                return Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_RANGE, format!("bytes */{len}"))
                    .body(Body::empty())
                    .unwrap();
            }
        }
    }

    let stream = ReaderStream::new(file);
    builder
        .header(header::CONTENT_LENGTH, len)
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream error").into_response())
}

#[derive(Deserialize)]
struct PowerReq {
    action: String,
}

async fn api_power(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PowerReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let action = req.action.to_ascii_lowercase();
    tracing::info!("power API request: {action}");
    match action.as_str() {
        "diag" => Ok(Json(json!({
            "ok": true,
            "shutdownPrivilege": crate::power::shutdown_privilege_available(),
        }))),
        "cancel" => {
            let had = state.cancel_power(true);
            Ok(Json(json!({ "ok": true, "cancelled": had })))
        }
        "lock" | "sleep" => {
            let act = action.clone();
            let result = tokio::task::spawn_blocking(move || crate::power::execute(&act))
                .await
                .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            match result {
                Ok(()) => {
                    state.add_activity(
                        "power",
                        &format!("{} requested", if action == "lock" { "Lock" } else { "Sleep" }),
                        "From your phone",
                        true,
                    );
                    Ok(Json(json!({ "ok": true, "action": action })))
                }
                Err(e) => {
                    state.add_activity("power", "Power action failed", &e, false);
                    Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e))
                }
            }
        }
        "restart" | "shutdown" => {
            let seconds = 8u64;
            state.schedule_power(&action, seconds);
            Ok(Json(json!({
                "ok": true,
                "scheduled": true,
                "action": action,
                "seconds": seconds,
            })))
        }
        _ => Err(ApiError::new(StatusCode::BAD_REQUEST, "unknown action")),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_range;

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_range("bytes=0-9", 100), Some((0, 9)));
        assert_eq!(parse_range("bytes=10-", 100), Some((10, 99)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=0-999", 100), Some((0, 99)));
        assert_eq!(parse_range("bytes=100-120", 100), None);
        assert_eq!(parse_range("bytes=9-2", 100), None);
        assert_eq!(parse_range("items=0-1", 100), None);
    }
}

async fn api_unpair() -> Response {
    let mut response = Json(json!({ "ok": true })).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("lumo_auth=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );
    response
}

#[derive(Deserialize)]
struct TextReq {
    text: String,
}

async fn api_text(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TextReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let text: String = req.text.chars().take(4000).collect();
    tokio::task::spawn_blocking(move || input::type_text(&text))
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "ok": true })))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let name = q
        .get("name")
        .map(|s| util::decode_header(s))
        .map(|s| s.chars().take(40).collect::<String>())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Phone".into());
    let ip = addr.ip().to_string();
    ws.on_upgrade(move |socket| ws_session(socket, state, name, ip))
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
enum ClientMsg {
    Move { dx: f64, dy: f64 },
    Button { button: String, action: String },
    Wheel { x: i32, y: i32 },
    Text { text: String },
    Ping { id: u64 },
    Clipboard,
    Media {
        action: String,
        #[serde(default)]
        delta: f64,
    },
}

async fn ws_session(socket: WebSocket, state: Arc<AppState>, name: String, ip: String) {
    let id = uuid::Uuid::new_v4().to_string();
    state.add_device(&id, &name, &ip);
    state.add_activity("connect", &format!("{name} connected"), &ip, true);
    let cfg = state.cfg();
    let welcome = json!({
        "t": "welcome",
        "deviceName": cfg.device_name,
        "version": APP_VERSION,
        "clipboardSync": cfg.clipboard_sync,
        "clipboard": state.clipboard(),
        "media": state.media(),
        "pendingPower": state.pending_power(),
        "serverTime": now_secs(),
    });
    let mut push_rx = state.push.subscribe();
    let (mut sender, mut receiver) = socket.split();
    if sender
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        state.remove_device(&id);
        return;
    }

    let token_generation = state.token_generation.load(Ordering::SeqCst);
    let mut acc_x = 0.0f64;
    let mut acc_y = 0.0f64;
    loop {
        if state.token_generation.load(Ordering::SeqCst) != token_generation {
            tracing::info!("closing {name} because the pairing token changed");
            break;
        }
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if state.token_generation.load(Ordering::SeqCst) != token_generation {
                            tracing::info!("closing {name} because the pairing token changed");
                            break;
                        }
                        if let Ok(msg) = serde_json::from_str::<ClientMsg>(&text) {
                            match msg {
                                ClientMsg::Move { dx, dy } => {
                                    acc_x += dx.clamp(-400.0, 400.0);
                                    acc_y += dy.clamp(-400.0, 400.0);
                                    let sx = acc_x.trunc();
                                    let sy = acc_y.trunc();
                                    acc_x -= sx;
                                    acc_y -= sy;
                                    if sx != 0.0 || sy != 0.0 {
                                        input::mouse_move(sx as i32, sy as i32);
                                    }
                                }
                                ClientMsg::Button { button, action } => {
                                    input::mouse_button(&button, &action);
                                }
                                ClientMsg::Wheel { x, y } => {
                                    input::wheel(x.clamp(-40, 40), y.clamp(-40, 40));
                                }
                                ClientMsg::Text { text } => {
                                    let text: String = text.chars().take(4000).collect();
                                    let _ = tokio::task::spawn_blocking(move || input::type_text(&text)).await;
                                }
                                ClientMsg::Ping { id } => {
                                    let pong = json!({ "t": "pong", "id": id });
                                    if sender.send(Message::Text(pong.to_string().into())).await.is_err() {
                                        break;
                                    }
                                }
                                ClientMsg::Clipboard => {
                                    let payload = json!({ "t": "clipboard", "clipboard": state.clipboard() });
                                    if sender.send(Message::Text(payload.to_string().into())).await.is_err() {
                                        break;
                                    }
                                }
                                ClientMsg::Media { action, delta } => {
                                    let state = state.clone();
                                    std::thread::spawn(move || crate::media::control(state, &action, delta));
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            pushed = push_rx.recv() => {
                match pushed {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }

    input::release_all();
    state.remove_device(&id);
    tracing::info!("phone {name} ({ip}) disconnected");
}
