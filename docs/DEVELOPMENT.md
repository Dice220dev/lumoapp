# Development

This document collects the details that do not belong in the short project README.

## Stack

- **Rust 2021 / Tauri 2** desktop shell, tray icon and IPC commands.
- **Axum 0.8 + Tokio** HTTP/WebSocket server embedded in the same binary.
- **Vanilla HTML/CSS/JS** phone app and desktop window; assets embedded with `rust-embed`.
- **Win32 APIs** for mouse/keyboard input, power actions, clipboard state and autostart.

## Layout

```
src/
  main.rs       entry, single-instance mutex, --headless, --diag-power
  app.rs        Tauri setup, tray, window creation, IPC commands
  server.rs     Axum router, auth, upload/download/Range, WebSocket input
  state.rs      shares, inbox, clipboard, devices, activity, power timer
  config.rs     JSON config, token, data paths
  input.rs      SendInput mouse/keyboard + Win+L fallback
  media.rs      Windows SMTC metadata, artwork and transport controls
  power.rs      lock/sleep/restart/shutdown
  clipboard.rs  arboard text/files/image bridge
  files.rs      path sanitising, zip/unzip, MIME/kind
  util.rs       LAN IP selection, QR SVG, Content-Disposition, reveal/open
  autostart.rs  HKCU Run key + StartupApproved entry
ui/
  tokens.css    design tokens and shared components
  mobile.*      phone PWA (touchpad/files/clipboard/power)
  index.html, desktop.*  Tauri window UI
icons/          app icons embedded in the exe
tools/          icon generator, build script, smoke test
```

## API

| Method | Path | Notes |
| --- | --- | --- |
| GET | `/` | phone UI |
| GET | `/pair?t=` | sets HttpOnly cookie, redirects to `/?t=` |
| GET | `/api/bootstrap` | phone state |
| GET | `/api/shares`, `/api/received` | file lists |
| GET/POST | `/api/clipboard` | read / set text |
| PUT | `/api/upload` | stream (`X-Lumo-Name`, optional `extract=1`, `clipboard=1`) |
| GET | `/api/dl/{id}` | download or stream with HTTP Range |
| POST | `/api/power` | `lock`, `sleep`, `restart`, `shutdown`, `cancel` |
| POST | `/api/text` | type text on the PC |
| POST | `/api/unpair` | clear the phone cookie |
| GET | `/ws` | input frames + clipboard/media pushes |

The pairing token is accepted as the `lumo_auth` cookie, `X-Lumo-Token` header or `?t=`
query parameter. Reset the link to rotate it and close live sessions.

## Data

| Item | Path |
| --- | --- |
| Config/token | `%LOCALAPPDATA%\Lumo\config.json` |
| Log | `%LOCALAPPDATA%\Lumo\logs\lumo.log` |
| Cache | `%LOCALAPPDATA%\Lumo\cache` |
| Inbox | `%USERPROFILE%\Downloads\Lumo Inbox` by default |
| Autostart | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Lumo` |

## Build and test

```powershell
cargo check
cargo test
powershell -ExecutionPolicy Bypass -File tools\build.ps1 -Release
```

The local GNU toolchain used during development lives in `..\.toolchain`; the build script
uses it when present. The release needs `WebView2Loader.dll` next to `Lumo.exe` for GNU
builds.

`tools/smoke.mjs` exercises pairing, authentication, upload/download, Range, zip upload,
WebSocket ping/pong and unpair against a running headless server:

```powershell
Lumo.exe --headless
node tools\smoke.mjs http://127.0.0.1:47821 <token>
```

## Design

The UI follows the Blizzard Blue on black design bible (semantic tokens, pure black background,
Blizzard Blue accent, both themes). Theme defaults to **dark**. The phone and desktop share
`ui/tokens.css`; components use only semantic tokens.

## Security notes

- LAN HTTP only: no service worker, no secure-context clipboard API on the phone; text copy
  uses the `execCommand` fallback.
- Any paired phone has full remote-control rights. Treat the link as a password.
- Synthetic input cannot reach elevated windows or the secure desktop.
- Power actions are phone-only; the desktop exposes no power controls.

## Known limitations

- Plain HTTP is an intentional local-network tradeoff.
- Most mobile browsers cannot select folders in their file picker (notably Safari);
  compress a folder in Files and let Lumo unpack the zip on the PC.
- Clipboard auto-sync may include sensitive text; it can be disabled.
- Name, logo and product copy are placeholders.
