use tauri::{
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use super::poller::PollerState;
use std::sync::{atomic::Ordering, OnceLock, Mutex};

/// Store tray menu item references for dynamic updates
struct TrayItems {
    status: MenuItem<tauri::Wry>,
    current_app: MenuItem<tauri::Wry>,
    now_playing: MenuItem<tauri::Wry>,
    start_service: MenuItem<tauri::Wry>,
    stop_service: MenuItem<tauri::Wry>,
}

static TRAY_ITEMS: OnceLock<Mutex<Option<TrayItems>>> = OnceLock::new();

/// Update tray menu items dynamically from the poller
pub fn update_tray_status(_app: &AppHandle, connected: bool, current_app: Option<&str>, now_playing: Option<&str>) {
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(items) = guard.as_ref() {
            let status_text = if connected { "Discord: connecté ✓" } else { "Discord: déconnecté" };
            let _ = items.status.set_text(status_text);
            let _ = items.current_app.set_text(current_app.unwrap_or("Aucune app détectée"));
            let _ = items.now_playing.set_text(now_playing.unwrap_or("Aucune musique"));
        }
    }
}

/// Update service toggle items
pub fn update_tray_service(running: bool) {
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(items) = guard.as_ref() {
            let _ = items.start_service.set_enabled(!running);
            let _ = items.stop_service.set_enabled(running);
        }
    }
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    // -- Status (read-only) --
    let status_item = MenuItem::with_id(app, "status", "Discord: déconnecté", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let current_app_item = MenuItem::with_id(app, "current_app", "Aucune app détectée", false, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Service --
    let start_service = MenuItem::with_id(app, "start_service", "▶ Démarrer le service", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let stop_service = MenuItem::with_id(app, "stop_service", "■ Arrêter le service", false, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Music controls --
    let now_playing = MenuItem::with_id(app, "now_playing", "Aucune musique", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let play_pause = MenuItem::with_id(app, "play_pause", "⏯ Lecture / Pause", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let prev_track = MenuItem::with_id(app, "prev_track", "⏮ Précédent", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let next_track = MenuItem::with_id(app, "next_track", "⏭ Suivant", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep_music1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let music_submenu = Submenu::with_items(
        app,
        "Musique",
        true,
        &[&now_playing, &sep_music1, &play_pause, &prev_track, &next_track],
    )
    .map_err(|e| e.to_string())?;

    // -- Shortcuts --
    let show_item = MenuItem::with_id(app, "show", "Ouvrir Activity", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_item = MenuItem::with_id(app, "settings", "Paramètres", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let check_update = MenuItem::with_id(app, "check_update", "Vérifier les mises à jour", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Quit --
    let quit_item = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep4 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &status_item,
            &current_app_item,
            &sep1,
            &start_service,
            &stop_service,
            &sep2,
            &music_submenu,
            &sep3,
            &show_item,
            &settings_item,
            &check_update,
            &sep4,
            &quit_item,
        ])
        .build()
        .map_err(|e| e.to_string())?;

    let default_icon = app
        .default_window_icon()
        .ok_or_else(|| "Could not get default window icon".to_string())?;

    let _tray = TrayIconBuilder::with_id("main_tray")
        .icon(default_icon.clone())
        .menu(&menu)
        .tooltip("Activity")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate", "settings");
                }
            }
            "check_update" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("check-update", ());
                }
            }
            "start_service" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let poller = app_handle.state::<PollerState>();
                    if !poller.running.load(Ordering::SeqCst) {
                        let rpc = app_handle.state::<super::discord_rpc::DiscordRpcState>();
                        let _ = super::poller::start_poller(poller, rpc, app_handle.clone()).await;
                    }
                    update_tray_service(true);
                    // Notify frontend
                    let _ = app_handle.emit("service-changed", true);
                });
            }
            "stop_service" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<PollerState>();
                    let _ = super::poller::stop_poller(state).await;
                    update_tray_service(false);
                    // Notify frontend
                    let _ = app_handle.emit("service-changed", false);
                });
            }
            "play_pause" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_play_pause().await; });
            }
            "next_track" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_next().await; });
            }
            "prev_track" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_previous().await; });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    // Store references for dynamic updates
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = lock.lock() {
        *guard = Some(TrayItems {
            status: status_item,
            current_app: current_app_item,
            now_playing,
            start_service,
            stop_service,
        });
    }

    Ok(())
}
