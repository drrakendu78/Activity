mod scripts;

use scripts::config::{load_config, load_config_internal, save_config, get_builtin_apps_list};
use scripts::discord_rpc::{disconnect_rpc, get_rpc_status, DiscordRpcState};
use scripts::iconify::{search_icons, set_app_icon};
use scripts::album_art::{validate_spotify, get_album_tracks_cmd};
use scripts::media_session::{get_current_media_cmd, media_play_pause, media_next, media_previous, media_seek, get_system_volume, set_system_volume, toggle_mute};
use scripts::poller::{is_poller_running, start_poller, stop_poller, get_detected_apps, PollerState};
use scripts::startup::{disable_auto_startup, enable_auto_startup, is_auto_startup_enabled};
use scripts::updater::{check_for_updates, start_silent_update};
use scripts::system_tray::{setup_system_tray, update_tray_language};
use scripts::window_detect::get_active_window;
use tauri::Manager;

#[tauri::command]
fn relaunch_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
fn set_tray_language(lang: String) {
    update_tray_language(&lang);
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/C", "start", "", &url]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Manage states
            app.manage(DiscordRpcState::default());
            app.manage(PollerState::default());

            let config = load_config_internal();

            // Setup system tray (unless hidden by config)
            if !config.hide_tray_icon {
                let _ = setup_system_tray(&app.handle());
            }

            // Apply acrylic/mica effect
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::apply_acrylic;
                    if apply_acrylic(&window, Some((18, 18, 20, 100))).is_err() {
                        let _ = window_vibrancy::apply_blur(&window, Some((18, 18, 20, 100)));
                    }
                }

                // If launched with --minimized flag, hide the window
                if std::env::args().any(|arg| arg == "--minimized") {
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
            get_current_media_cmd,
            media_play_pause,
            media_next,
            media_previous,
            media_seek,
            get_system_volume,
            set_system_volume,
            toggle_mute,
            validate_spotify,
            get_album_tracks_cmd,
            open_url,
            check_for_updates,
            start_silent_update,
            relaunch_app,
            set_tray_language,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
