mod scripts;

use scripts::config::{load_config, save_config, get_builtin_apps_list};
use scripts::discord_rpc::{disconnect_rpc, get_rpc_status, DiscordRpcState};
use scripts::iconify::{search_icons, set_app_icon};
use scripts::poller::{is_poller_running, start_poller, stop_poller, get_detected_apps, PollerState};
use scripts::startup::{disable_auto_startup, enable_auto_startup, is_auto_startup_enabled};
use scripts::system_tray::setup_system_tray;
use scripts::window_detect::get_active_window;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Manage states
            app.manage(DiscordRpcState::default());
            app.manage(PollerState::default());

            // Setup system tray
            if let Err(e) = setup_system_tray(&app.handle()) {
                eprintln!("Failed to setup system tray: {}", e);
            }

            // If launched with --minimized flag, hide the window
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            get_active_window,
            start_poller,
            stop_poller,
            is_poller_running,
            get_rpc_status,
            disconnect_rpc,
            enable_auto_startup,
            disable_auto_startup,
            is_auto_startup_enabled,
            get_builtin_apps_list,
            get_detected_apps,
            search_icons,
            set_app_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
