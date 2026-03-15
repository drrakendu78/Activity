use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{command, State};

use super::config::AppConfig;

pub struct DiscordRpcState {
    pub client: Mutex<Option<DiscordIpcClient>>,
    pub connected: Mutex<bool>,
    pub current_app: Mutex<Option<String>>,
}

impl Default for DiscordRpcState {
    fn default() -> Self {
        Self {
            client: Mutex::new(None),
            connected: Mutex::new(false),
            current_app: Mutex::new(None),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct RpcStatus {
    pub connected: bool,
    pub current_app: Option<String>,
}

pub fn connect_rpc_internal(
    state: &DiscordRpcState,
    app_id: &str,
) -> Result<(), String> {
    let mut client_lock = state.client.lock().map_err(|e| e.to_string())?;
    let mut connected_lock = state.connected.lock().map_err(|e| e.to_string())?;

    // Close existing connection if any
    if let Some(ref mut old_client) = *client_lock {
        let _ = old_client.close();
    }

    let mut client = DiscordIpcClient::new(app_id);

    match client.connect() {
        Ok(_) => {
            *client_lock = Some(client);
            *connected_lock = true;
            Ok(())
        }
        Err(e) => {
            *client_lock = None;
            *connected_lock = false;
            Err(format!("Failed to connect to Discord: {}. Make sure Discord is running.", e))
        }
    }
}

pub fn set_activity_internal(
    state: &DiscordRpcState,
    app_config: &AppConfig,
    details_override: Option<&str>,
    state_override: Option<&str>,
    start_timestamp: Option<i64>,
) -> Result<(), String> {
    let mut client_lock = state.client.lock().map_err(|e| e.to_string())?;
    let connected = *state.connected.lock().map_err(|e| e.to_string())?;

    if !connected {
        return Err("Not connected to Discord".to_string());
    }

    if let Some(ref mut client) = *client_lock {
        let details_text = details_override.unwrap_or(&app_config.details);
        let state_text = state_override.unwrap_or(&app_config.state);

        let mut discord_activity = activity::Activity::new();

        // Set the activity name to the app name (overrides the Discord App name)
        discord_activity = discord_activity.name(&app_config.name);

        if !details_text.is_empty() {
            discord_activity = discord_activity.details(details_text);
        }
        if !state_text.is_empty() {
            discord_activity = discord_activity.state(state_text);
        }

        // Timestamps — show elapsed time (Unix seconds)
        if app_config.show_timestamp {
            if let Some(ts) = start_timestamp {
                let timestamps = activity::Timestamps::new().start(ts);
                discord_activity = discord_activity.timestamps(timestamps);
            }
        }

        // Assets — images (supports both asset keys and URLs)
        // Auto-convert SVG URLs to PNG via weserv.nl proxy (Discord only accepts PNG/JPG)
        let large_image_url = app_config.large_image_key.as_ref().map(|key| {
            if key.starts_with("http") && key.to_lowercase().ends_with(".svg") {
                format!("https://images.weserv.nl/?url={}&w=512&h=512&output=png", key)
            } else {
                key.clone()
            }
        });

        let mut assets = activity::Assets::new();
        let mut has_assets = false;

        if let Some(ref key) = large_image_url {
            if !key.is_empty() {
                assets = assets.large_image(key.as_str());
                has_assets = true;
            }
        }
        if let Some(ref text) = app_config.large_image_text {
            if !text.is_empty() {
                assets = assets.large_text(text.as_str());
                has_assets = true;
            }
        }
        if let Some(ref key) = app_config.small_image_key {
            if !key.is_empty() {
                assets = assets.small_image(key.as_str());
                has_assets = true;
            }
        }
        if let Some(ref text) = app_config.small_image_text {
            if !text.is_empty() {
                assets = assets.small_text(text.as_str());
                has_assets = true;
            }
        }

        if has_assets {
            discord_activity = discord_activity.assets(assets);
        }

        // Buttons — clickable links (max 2)
        if !app_config.buttons.is_empty() {
            let btns: Vec<activity::Button> = app_config
                .buttons
                .iter()
                .take(2) // Discord allows max 2 buttons
                .map(|b| activity::Button::new(&b.label, &b.url))
                .collect();
            discord_activity = discord_activity.buttons(btns);
        }

        match client.set_activity(discord_activity) {
            Ok(_) => {
                let mut current = state.current_app.lock().map_err(|e| e.to_string())?;
                *current = Some(app_config.name.clone());
                Ok(())
            }
            Err(e) => {
                drop(client_lock);
                let mut connected_lock = state.connected.lock().map_err(|e| e.to_string())?;
                *connected_lock = false;
                Err(format!("Discord connection lost: {}", e))
            }
        }
    } else {
        Err("Discord client not initialized".to_string())
    }
}

pub fn disconnect_rpc_internal(state: &DiscordRpcState) -> Result<(), String> {
    let mut client_lock = state.client.lock().map_err(|e| e.to_string())?;
    let mut connected_lock = state.connected.lock().map_err(|e| e.to_string())?;
    let mut current_lock = state.current_app.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut client) = *client_lock {
        let _ = client.close();
    }

    *client_lock = None;
    *connected_lock = false;
    *current_lock = None;
    Ok(())
}

#[command]
pub fn disconnect_rpc(state: State<DiscordRpcState>) -> Result<(), String> {
    disconnect_rpc_internal(&state)
}

#[command]
pub fn get_rpc_status(state: State<DiscordRpcState>) -> Result<RpcStatus, String> {
    let connected = *state.connected.lock().map_err(|e| e.to_string())?;
    let current_app = state.current_app.lock().map_err(|e| e.to_string())?.clone();
    Ok(RpcStatus {
        connected,
        current_app,
    })
}
