use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{command, AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as TokioMutex;

use super::config::{auto_register_app, load_config_internal, resolve_app_config, save_config};
use super::discord_rpc::{connect_rpc_internal, disconnect_rpc_internal, set_activity_internal, DiscordRpcState};
use super::steamgriddb;
use super::window_detect::get_foreground_app_internal;

const APP_ID: &str = "1482735786653778101";

/// Tracks a detected app with last-seen timestamp
#[derive(Serialize, Clone, Debug)]
pub struct DetectedApp {
    pub exe_name: String,
    pub name: String,
    pub category: String,
    pub details: String,
    pub last_seen: u64,
    pub is_builtin: bool,
}

pub struct PollerState {
    pub running: Arc<AtomicBool>,
    pub handle: TokioMutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    pub detected_apps: Arc<Mutex<HashMap<String, DetectedApp>>>,
}

impl Default for PollerState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: TokioMutex::new(None),
            detected_apps: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct RpcStatusEvent {
    pub connected: bool,
    pub current_app: Option<String>,
    pub exe_name: Option<String>,
    pub window_title: Option<String>,
    pub category: Option<String>,
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image_key: Option<String>,
    pub start_timestamp: Option<i64>,
}

#[command]
pub async fn start_poller(
    poller: State<'_, PollerState>,
    rpc: State<'_, DiscordRpcState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    if poller.running.load(Ordering::Relaxed) {
        return Ok(());
    }

    let config = load_config_internal();

    connect_rpc_internal(&rpc, APP_ID)?;
    poller.running.store(true, Ordering::Relaxed);

    let running = poller.running.clone();
    let poll_interval = config.poll_interval_ms;
    let app_handle_clone = app_handle.clone();
    let detected_apps = poller.detected_apps.clone();

    let handle = tauri::async_runtime::spawn(async move {
        let mut last_exe = String::new();
        // Remember first-seen timestamp per app (persists across switches)
        let mut app_first_seen: HashMap<String, i64> = HashMap::new();
        while running.load(Ordering::Relaxed) {
            let config = load_config_internal();
            let builtin = super::config::get_builtin_apps();

            if let Some(fg) = get_foreground_app_internal() {
                // Skip ignored system apps
                if super::config::is_ignored_app(&fg.exe_name) {
                    tokio::time::sleep(Duration::from_millis(poll_interval)).await;
                    continue;
                }

                // Auto-register unknown apps into config.json
                let was_new = auto_register_app(&fg.exe_name, &fg.window_title);

                // Reload config if a new app was just added
                let mut config = if was_new { load_config_internal() } else { config };

                // ═══ AUTO-FETCH ICONS ═══
                // Resolve the current app config (from user config OR builtin DB)
                let (resolved_cfg, _, _) = resolve_app_config(&fg.exe_name, &fg.window_title, &config);
                let needs_icon = resolved_cfg.large_image_key.as_ref().map_or(true, |k| k.is_empty());

                if needs_icon {
                    let search_name = resolved_cfg.name.clone();
                    let category = resolved_cfg.category.clone();
                    let mut found_icon: Option<String> = None;

                    // 1. Try SteamGridDB first for games (better game-specific icons)
                    if (category == "gaming" || category == "discovered") && !config.steamgriddb_api_key.is_empty() {
                        let api_key = config.steamgriddb_api_key.clone();
                        found_icon = steamgriddb::fetch_icon(&search_name, &api_key).await;
                    }

                    // 2. Fallback: Iconify for ALL apps (colorful icons from 275k+ set)
                    if found_icon.is_none() {
                        found_icon = super::iconify::fetch_icon(&search_name).await;
                    }

                    // Save the found icon into user config (persists across restarts)
                    if let Some(icon_url) = found_icon {
                        if config.apps.contains_key(&fg.exe_name) {
                            // App already in user config — just update the icon
                            if let Some(app) = config.apps.get_mut(&fg.exe_name) {
                                app.large_image_key = Some(icon_url);
                            }
                        } else {
                            // App is from builtin DB — clone it into user config with the icon
                            let mut app_cfg = resolved_cfg.clone();
                            app_cfg.large_image_key = Some(icon_url);
                            config.apps.insert(fg.exe_name.clone(), app_cfg);
                        }
                        let _ = save_config(config.clone());
                        config = load_config_internal();
                    }
                }

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Track every detected app in memory
                {
                    let is_builtin = builtin.contains_key(&fg.exe_name);
                    let (resolved, _, _) = resolve_app_config(&fg.exe_name, &fg.window_title, &config);

                    if let Ok(mut apps) = detected_apps.lock() {
                        apps.insert(fg.exe_name.clone(), DetectedApp {
                            exe_name: fg.exe_name.clone(),
                            name: resolved.name.clone(),
                            category: resolved.category.clone(),
                            details: resolved.details.clone(),
                            last_seen: now,
                            is_builtin: is_builtin || was_new,
                        });
                    }
                }

                // Only send activity to Discord on app change (Discord auto-ticks the timer)
                if fg.exe_name != last_exe {
                    last_exe = fg.exe_name.clone();

                    // Use first-seen timestamp (never resets during session)
                    let app_start = *app_first_seen
                        .entry(fg.exe_name.clone())
                        .or_insert(now as i64);

                    let rpc_state = app_handle_clone.state::<DiscordRpcState>();

                    let (app_config, details_override, state_override) =
                        resolve_app_config(&fg.exe_name, &fg.window_title, &config);

                    let timestamp = if app_config.show_timestamp {
                        Some(app_start)
                    } else {
                        None
                    };

                    let result = set_activity_internal(
                        &rpc_state,
                        &app_config,
                        details_override.as_deref(),
                        state_override.as_deref(),
                        timestamp,
                    );

                    if result.is_err() {
                        let _ = connect_rpc_internal(&rpc_state, APP_ID);
                        let _ = set_activity_internal(
                            &rpc_state,
                            &app_config,
                            details_override.as_deref(),
                            state_override.as_deref(),
                            timestamp,
                        );
                    }

                    let connected =
                        *rpc_state.connected.lock().unwrap_or_else(|e| e.into_inner());
                    let current_app = rpc_state
                        .current_app
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .clone();

                    let shown_details = details_override.unwrap_or_else(|| app_config.details.clone());
                    let shown_state = state_override.unwrap_or_else(|| app_config.state.clone());

                    let _ = app_handle_clone.emit(
                        "rpc-status-changed",
                        RpcStatusEvent {
                            connected,
                            current_app,
                            exe_name: Some(fg.exe_name),
                            window_title: Some(fg.window_title),
                            category: Some(app_config.category),
                            details: Some(shown_details),
                            state: Some(shown_state),
                            large_image_key: app_config.large_image_key,
                            start_timestamp: Some(app_start),
                        },
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(poll_interval)).await;
        }

        if let Some(rpc_state) = app_handle_clone.try_state::<DiscordRpcState>() {
            let _ = disconnect_rpc_internal(&rpc_state);
        }
    });

    let mut handle_lock = poller.handle.lock().await;
    *handle_lock = Some(handle);

    let connected = *rpc.connected.lock().map_err(|e| e.to_string())?;
    let _ = app_handle.emit(
        "rpc-status-changed",
        RpcStatusEvent {
            connected,
            current_app: None,
            exe_name: None,
            window_title: None,
            category: None,
            details: None,
            state: None,
            large_image_key: None,
            start_timestamp: None,
        },
    );

    Ok(())
}

#[command]
pub async fn stop_poller(
    poller: State<'_, PollerState>,
) -> Result<(), String> {
    poller.running.store(false, Ordering::Relaxed);

    let mut handle_lock = poller.handle.lock().await;
    if let Some(handle) = handle_lock.take() {
        handle.abort();
    }

    Ok(())
}

#[command]
pub fn is_poller_running(poller: State<PollerState>) -> bool {
    poller.running.load(Ordering::Relaxed)
}

/// Returns all apps detected during this session
#[command]
pub fn get_detected_apps(poller: State<PollerState>) -> Result<Vec<DetectedApp>, String> {
    let apps = poller.detected_apps.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<DetectedApp> = apps.values().cloned().collect();
    list.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
    Ok(list)
}
