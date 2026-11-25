//! Windows System Media Transport Controls (SMTC) bridge.
//!
//! Windows exposes media from Spotify, browsers (YouTube etc.), players and
//! anything else using the system media session APIs. This module polls the
//! current session, forwards metadata/artwork to the phone, and executes phone
//! transport commands.

use crate::state::{AppState, MediaView};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionMediaProperties as MediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

pub fn start(state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("lumo-media".into())
        .spawn(move || {
            init_com();
            let mut manager: Option<Manager> = None;
            let mut art_key = String::new();
            let mut art: Option<String> = None;
            loop {
                if manager.is_none() {
                    manager = Manager::RequestAsync()
                        .ok()
                        .and_then(|operation| operation.get().ok());
                }
                let view = match manager.as_ref() {
                    Some(manager) => read(manager, &mut art_key, &mut art),
                    None => MediaView::default(),
                };
                state.set_media(view);
                std::thread::sleep(Duration::from_millis(1000));
            }
        })
        .ok();
}

pub fn control(state: Arc<AppState>, action: &str, delta: f64) {
    let action = action.to_ascii_lowercase();
    std::thread::Builder::new()
        .name("lumo-media-control".into())
        .spawn(move || {
            init_com();
            let Ok(manager) = Manager::RequestAsync().and_then(|operation| operation.get()) else {
                return;
            };
            let Some(session) = pick_session(&manager) else {
                return;
            };
            let result = match action.as_str() {
                "toggle" => toggle(&session),
                "play" => session.TryPlayAsync().and_then(|operation| operation.get()).map(|_| ()),
                "pause" => session.TryPauseAsync().and_then(|operation| operation.get()).map(|_| ()),
                "prev" => session
                    .TrySkipPreviousAsync()
                    .and_then(|operation| operation.get())
                    .map(|_| ()),
                "next" => session
                    .TrySkipNextAsync()
                    .and_then(|operation| operation.get())
                    .map(|_| ()),
                "seek" => {
                    // Match player behaviour: tap the arrow key instead of
                    // forcing an absolute position. Works with YouTube, VLC,
                    // browsers and most desktop players.
                    crate::input::arrow_key(delta < 0.0);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                tracing::warn!("media control {action} failed: {error}");
            }
            // The monitor will publish the new state on its next tick; publish one
            // immediately as well so the phone feels responsive.
            let mut art_key = String::new();
            let mut art = None;
            state.set_media(read(&manager, &mut art_key, &mut art));
        })
        .ok();
}

fn init_com() {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
}

fn toggle(session: &Session) -> windows::core::Result<()> {
    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .map(|status| status == PlaybackStatus::Playing)
        .unwrap_or(false);
    if playing {
        session.TryPauseAsync().and_then(|operation| operation.get()).map(|_| ())
    } else {
        session.TryPlayAsync().and_then(|operation| operation.get()).map(|_| ())
    }
}

fn pick_session(manager: &Manager) -> Option<Session> {
    // Prefer any actively playing session so a paused player does not hide a
    // YouTube/browser video that is currently running.
    if let Ok(sessions) = manager.GetSessions() {
        let count = sessions.Size().unwrap_or(0);
        for index in 0..count {
            if let Ok(session) = sessions.GetAt(index) {
                if is_playing(&session) {
                    return Some(session);
                }
            }
        }
    }
    manager.GetCurrentSession().ok()
}

fn is_playing(session: &Session) -> bool {
    session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .map(|status| status == PlaybackStatus::Playing)
        .unwrap_or(false)
}

fn read(manager: &Manager, art_key: &mut String, art: &mut Option<String>) -> MediaView {
    let Some(session) = pick_session(manager) else {
        *art_key = String::new();
        *art = None;
        return MediaView::default();
    };
    let properties = session
        .TryGetMediaPropertiesAsync()
        .ok()
        .and_then(|operation| operation.get().ok());
    let title = properties
        .as_ref()
        .and_then(|value| value.Title().ok())
        .map(|value| value.to_string())
        .unwrap_or_default();
    let artist = properties
        .as_ref()
        .and_then(|value| value.Artist().ok())
        .map(|value| value.to_string())
        .unwrap_or_default();
    let album = properties
        .as_ref()
        .and_then(|value| value.AlbumTitle().ok())
        .map(|value| value.to_string())
        .unwrap_or_default();
    let app = session
        .SourceAppUserModelId()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let status = session
        .GetPlaybackInfo()
        .and_then(|value| value.PlaybackStatus())
        .ok();
    let playing = status == Some(PlaybackStatus::Playing);
    let stopped = matches!(
        status,
        Some(PlaybackStatus::Stopped) | Some(PlaybackStatus::Closed)
    );
    let active = !stopped && (playing || !title.trim().is_empty());

    // SMTC reports the position as of LastUpdatedTime, and some players only
    // refresh that a few seconds apart. Extrapolate while playing so the phone
    // receives a live clock even when the app is slow to publish.
    let now_1601 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() + 11_644_473_600.0)
        .unwrap_or(0.0);
    let (position, duration) = session
        .GetTimelineProperties()
        .map(|timeline| {
            let start = timeline.StartTime().map(seconds).unwrap_or(0.0);
            let end = timeline.EndTime().map(seconds).unwrap_or(0.0);
            let reported = timeline.Position().map(seconds).unwrap_or(0.0);
            let last_updated = timeline
                .LastUpdatedTime()
                .map(|value| value.UniversalTime as f64 / 10_000_000.0)
                .unwrap_or(0.0);
            let elapsed = if playing && last_updated > 0.0 {
                (now_1601 - last_updated).clamp(0.0, 3600.0)
            } else {
                0.0
            };
            let duration = (end - start).max(0.0);
            let position = if duration > 0.0 {
                (reported - start + elapsed).clamp(0.0, duration)
            } else {
                (reported - start + elapsed).max(0.0)
            };
            (position, duration)
        })
        .unwrap_or((0.0, 0.0));

    let key = format!("{app}|{title}|{artist}|{album}");
    if key != *art_key {
        *art_key = key;
        *art = properties
            .as_ref()
            .and_then(|properties| thumbnail(properties));
    }
    if !active {
        *art = None;
    }

    MediaView {
        active,
        title,
        artist,
        album,
        app,
        playing,
        position,
        duration,
        can_seek: duration > 1.0,
        art: art.clone(),
    }
}

fn thumbnail(properties: &MediaProperties) -> Option<String> {
    let reference = properties.Thumbnail().ok()?;
    let stream = reference.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()? as usize;
    if size == 0 || size > 8_000_000 {
        return None;
    }
    let input = stream.GetInputStreamAt(0).ok()?;
    let reader = windows::Storage::Streams::DataReader::CreateDataReader(&input).ok()?;
    reader.LoadAsync(size as u32).ok()?.get().ok()?;
    let mut bytes = vec![0u8; size];
    reader.ReadBytes(&mut bytes).ok()?;

    let image = image::load_from_memory(&bytes).ok()?;
    let thumb = image.thumbnail(128, 128);
    let mut encoded = Vec::new();
    thumb
        .write_to(
            &mut std::io::Cursor::new(&mut encoded),
            image::ImageFormat::Jpeg,
        )
        .ok()?;
    Some(format!("data:image/jpeg;base64,{}", STANDARD.encode(encoded)))
}

fn seconds(value: windows::Foundation::TimeSpan) -> f64 {
    value.Duration as f64 / 10_000_000.0
}
