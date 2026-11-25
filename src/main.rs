#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod autostart;
mod clipboard;
mod config;
mod files;
mod input;
mod media;
mod power;
mod server;
mod state;
mod util;

use config::Config;
use state::AppState;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--headless") {
        headless();
        return;
    }
    if args.iter().any(|arg| arg == "--diag-power") {
        println!(
            "shutdown_privilege={}",
            power::shutdown_privilege_available()
        );
        return;
    }
    if already_running() {
        return;
    }
    app::run();
}

fn already_running() -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;
    unsafe {
        match CreateMutexW(None, true, w!("Local\\LumoBridgeInstance")) {
            Ok(handle) => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    true
                } else {
                    let _ = handle;
                    false
                }
            }
            Err(_) => false,
        }
    }
}

fn headless() {
    let data_dir = config::data_dir();
    app::init_logging(&data_dir);
    let cfg = Config::load(&data_dir);
    let state = AppState::new(cfg, data_dir);
    crate::media::start(state.clone());
    let (listener, port) = server::bind(state.port()).expect("bind local server");
    state.set_port(port);
    println!("LUMO_HEADLESS_URL {}", state.pair_url());
    println!("LUMO_BASE_URL {}", state.base_url());
    println!("LUMO_TOKEN {}", state.token());
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime
        .block_on(async move { server::serve(state, listener).await })
        .expect("server stopped");
}
