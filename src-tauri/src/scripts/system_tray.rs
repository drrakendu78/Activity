use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])
        .map_err(|e| e.to_string())?;

    let default_icon = app
        .default_window_icon()
        .ok_or_else(|| "Could not get default window icon".to_string())?;

    let _tray = TrayIconBuilder::new()
        .icon(default_icon.clone())
        .menu(&menu)
        .tooltip("Discord RPC Manager")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
