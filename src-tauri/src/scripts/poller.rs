use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{command, AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as TokioMutex;

use super::album_art;
use super::config::{auto_register_app, load_config_internal, resolve_app_config, save_config, AppConfig};
use super::discord_rpc::{connect_rpc_internal, disconnect_rpc_internal, set_activity_internal, DiscordRpcState};
use super::media_session;
use super::steamgriddb;
use super::window_detect::get_foreground_app_internal;

const APP_ID: &str = "1482735786653778101";

/// Check if the foreground app IS the music player itself (skip showing its own icon)
fn resolved_is_music_player(exe_name: &str, player_name: &str) -> bool {
    let exe = exe_name.to_lowercase();
    let player = player_name.to_lowercase();
    // Common music players
    if exe.contains("spotify") && player.contains("spotify") { return true; }
    if (exe.contains("applemusic") || exe.contains("itunes")) && player.contains("apple") { return true; }
    if exe.contains("tidal") && player.contains("tidal") { return true; }
    if exe.contains("deezer") && player.contains("deezer") { return true; }
    false
}

/// Tracks a detected app with last-seen timestamp
#[derive(Serialize, Clone, Debug)]
pub struct DetectedApp {
    pub exe_name: String,
    pub name: String,
    pub category: String,
    pub details: String,
    pub last_seen: u64,
    pub is_builtin: bool,
    pub large_image_key: Option<String>,
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
    #[serde(default)]
    pub is_music: bool,
    pub small_image_key: Option<String>,
    pub album_name: Option<String>,
    pub listen_url: Option<String>,
    pub playback_status: Option<String>,
    pub position_secs: Option<f64>,
    pub duration_secs: Option<f64>,
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
        // Heartbeat: re-send activity every 15min to keep RPC alive
        let mut last_activity_sent: u64 = 0;
        const HEARTBEAT_SECS: u64 = 900; // 15 minutes
        // Music detection state
        let mut last_media_key = String::new();
        let mut media_art_url: Option<String> = None;
        let mut media_player_icon: Option<String> = None;
        while running.load(Ordering::Relaxed) {
            let config = load_config_internal();
            let builtin = super::config::get_builtin_apps();

            // ═══ MEDIA DETECTION (highest priority) ═══
            // 3 modes:
            //   1. Real music player → music presence with album art
            //   2. Browser + real song → music presence with album art
            //   3. Browser + video/stream → show site icon (YouTube, Twitch...) + video title
            let mut media_handled = false;
            let mut skip_browser_video = false;
            if config.music_enabled {
                if let Some(media) = media_session::get_current_media().await {
                    let is_playing = media.playback_status == "Playing";
                    // Only show paused for real music players, not browser videos
                    let show_when_paused = config.music_config.show_paused
                        && media.playback_status == "Paused"
                        && !media.is_browser;
                    // For browser video/streams: only show if the user is actually watching
                    // Music players always take priority regardless of focus
                    if media.is_browser && is_playing {
                        if let Some(fg) = get_foreground_app_internal() {
                            let fg_title_lower = fg.window_title.to_lowercase();
                            let media_title_lower = media.title.to_lowercase();
                            let service_names = ["twitch", "youtube", "netflix", "disney+", "crunchyroll", "prime video"];
                            let is_watching = fg_title_lower.contains(&media_title_lower)
                                || service_names.iter().any(|s| fg_title_lower.contains(s));
                            // println!("[Poller] Browser video check: fg='{}' media='{}' is_watching={}", fg.window_title, media.title, is_watching);
                            if !is_watching {
                                skip_browser_video = true;
                                // Clear media state so normal detection picks up
                                last_media_key.clear();
                                media_art_url = None;
                                media_player_icon = None;
                            }
                        }
                    }

                    if (is_playing || show_when_paused) && !media.title.is_empty() && !skip_browser_video {
                        let media_key = format!("{}|{}", media.artist.to_lowercase(), media.title.to_lowercase());
                        let song_changed = media_key != last_media_key;

                        // Track whether this is confirmed music (for browser sources)
                        let mut is_confirmed_music = !media.is_browser; // Real players are always music

                        if song_changed {
                            last_media_key = media_key;

                            if media.is_browser {
                                // Browser: check if it's real music via Deezer
                                is_confirmed_music = album_art::is_known_music(&media.artist, &media.title).await;
                                if is_confirmed_music {
                                    media_art_url = album_art::fetch_album_art_url(&media.artist, &media.title).await;
                                    media_player_icon = super::iconify::fetch_icon(&media.player_name).await;
                                } else {
                                    // Not music → use site icon as large image
                                    media_art_url = super::iconify::fetch_icon(&media.player_name).await;
                                    media_player_icon = None;
                                }
                            } else {
                                // Real music player: always fetch album art
                                media_art_url = album_art::fetch_album_art_url(&media.artist, &media.title).await;
                                media_player_icon = super::iconify::fetch_icon(&media.player_name).await;
                            }
                        }

                        // For non-changed browser video, remember if it was music or not
                        if !song_changed && media.is_browser {
                            // If we have a player icon but no small icon, it's video mode
                            is_confirmed_music = media_player_icon.is_some();
                        }

                        let mc = &config.music_config;
                        let mut music_buttons: Vec<super::config::ButtonConfig> = Vec::new();

                        if mc.show_get_status_button {
                            music_buttons.push(super::config::ButtonConfig {
                                label: "Get this status".to_string(),
                                url: "https://github.com".to_string(), // Placeholder
                            });
                        }
                        if mc.show_listen_button {
                            if let Some(btn) = album_art::get_listen_url(&media.player_name, &media.artist, &media.title) {
                                music_buttons.push(btn);
                            }
                        }
                        // Discord allows max 2 buttons
                        music_buttons.truncate(2);

                        let category = if is_confirmed_music { "music" } else { "media" };

                        // Build display name (what shows as the activity name)
                        let display_name = match mc.display_text.as_str() {
                            "artist" => if media.artist.is_empty() { media.player_name.clone() } else { media.artist.clone() },
                            "title" => media.title.clone(),
                            "media_type" => "Music".to_string(),
                            "custom" => if mc.display_text_custom.is_empty() { media.player_name.clone() } else { mc.display_text_custom.clone() },
                            _ => media.player_name.clone(), // "player"
                        };

                        // Build details and state based on music config options
                        let (details_text, state_text) = if mc.hide_music_info {
                            (String::new(), String::new())
                        } else {
                            let mut title_part = media.title.clone();
                            let mut artist_part = if media.artist.is_empty() { String::new() } else { media.artist.clone() };

                            if mc.prefix_artist_by && !artist_part.is_empty() {
                                artist_part = format!("by {}", artist_part);
                            }

                            let album_part = if mc.show_album && !media.album.is_empty() {
                                if mc.prefix_album_on {
                                    format!("on {}", media.album)
                                } else {
                                    media.album.clone()
                                }
                            } else if mc.show_album_when_no_artist && media.artist.is_empty() && !media.album.is_empty() {
                                if mc.prefix_album_on {
                                    format!("on {}", media.album)
                                } else {
                                    media.album.clone()
                                }
                            } else {
                                String::new()
                            };

                            if mc.reverse_title_artist && !artist_part.is_empty() {
                                std::mem::swap(&mut title_part, &mut artist_part);
                            }

                            if mc.title_artist_one_line && !artist_part.is_empty() {
                                let combined = format!("{} - {}", title_part, artist_part);
                                (combined, album_part)
                            } else if mc.artist_album_one_line && !artist_part.is_empty() && !album_part.is_empty() {
                                let combined_state = format!("{} - {}", artist_part, album_part);
                                (title_part, combined_state)
                            } else {
                                // Default: details = title, state = artist (+ album on separate thought)
                                let state = if !artist_part.is_empty() && !album_part.is_empty() {
                                    format!("{} - {}", artist_part, album_part)
                                } else if !artist_part.is_empty() {
                                    artist_part
                                } else if !album_part.is_empty() {
                                    album_part
                                } else {
                                    media.player_name.clone()
                                };
                                (title_part, state)
                            }
                        };

                        let show_timestamp = mc.show_playback_info && (is_playing || (show_when_paused && !mc.freeze_progress_when_paused));

                        // Detect foreground app while music is playing (reload config for latest icons)
                        let fresh_config = load_config_internal();
                        let (fg_app_label, fg_app_icon) = if let Some(fg) = get_foreground_app_internal() {
                            if !super::config::is_ignored_app(&fg.exe_name)
                                && !resolved_is_music_player(&fg.exe_name, &media.player_name) {
                                let (resolved, _, _) = resolve_app_config(&fg.exe_name, &fg.window_title, &fresh_config);
                                let mut icon = resolved.large_image_key.clone()
                                    .filter(|u| !u.is_empty() && u.starts_with("http"));
                                // If no icon saved, fetch it on the fly
                                if icon.is_none() {
                                    icon = super::iconify::fetch_icon(&resolved.name).await;
                                }
                                // For browsers, extract the site/page name from window title
                                let label = if resolved.category == "browser" {
                                    let title = fg.window_title.clone();
                                    // Remove browser name suffix (e.g. " — Mozilla Firefox")
                                    let browsers = [" — Mozilla Firefox", " - Mozilla Firefox",
                                        " — Google Chrome", " - Google Chrome",
                                        " — Microsoft Edge", " - Microsoft Edge",
                                        " — Brave", " - Brave", " — Opera", " - Opera",
                                        " — Vivaldi", " - Vivaldi", " — Arc", " - Arc"];
                                    let mut clean = title.clone();
                                    for b in &browsers {
                                        if let Some(pos) = clean.rfind(b) {
                                            clean = clean[..pos].to_string();
                                            break;
                                        }
                                    }
                                    // Take just the first meaningful part (before " — " or " - " or " | ")
                                    let site = clean.split(" — ").next()
                                        .unwrap_or(&clean)
                                        .split(" - ").next()
                                        .unwrap_or(&clean)
                                        .split(" | ").next()
                                        .unwrap_or(&clean)
                                        .trim()
                                        .to_string();
                                    // Truncate long site names
                                    let site = if site.len() > 60 { format!("{}…", &site[..58]) } else { site };
                                    if site.is_empty() { resolved.name.clone() } else { site }
                                } else {
                                    resolved.name.clone()
                                };
                                (Some(label), icon)
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };

                        // Small icon: pause when paused, foreground app icon, or player logo
                        let is_paused = media.playback_status == "Paused";
                        let (small_icon, small_text) = if is_paused && mc.show_pause_icon {
                            (
                                Some("https://images.weserv.nl/?url=https%3A%2F%2Fapi.iconify.design%2Fmdi%2Fpause-circle.svg%3Fcolor%3D%2523000000%26width%3D512&w=512&h=512&output=png".to_string()),
                                Some("En pause".to_string()),
                            )
                        } else if let (Some(ref label), Some(ref icon)) = (&fg_app_label, &fg_app_icon) {
                            // Discord small_text limit is 128 chars — truncate
                            let tooltip = format!("Sur {}", label);
                            let tooltip = if tooltip.len() > 128 { format!("{}…", &tooltip[..126]) } else { tooltip };
                            (Some(icon.clone()), Some(tooltip))
                        } else if mc.show_player_logo {
                            (media_player_icon.clone(), Some(media.player_name.clone()))
                        } else {
                            (None, Some(media.player_name.clone()))
                        };

                        // Append foreground app to state text so Discord re-renders
                        let final_state = if let Some(ref label) = fg_app_label {
                            if state_text.is_empty() {
                                format!("Sur {}", label)
                            } else {
                                format!("{} · sur {}", state_text, label)
                            }
                        } else {
                            state_text.clone()
                        };

                        let music_config = AppConfig {
                            name: display_name.clone(),
                            details: details_text.clone(),
                            state: final_state.clone(),
                            large_image_key: media_art_url.clone(),
                            large_image_text: if media.album.is_empty() { None } else { Some(media.album.clone()) },
                            small_image_key: small_icon.clone(),
                            small_image_text: small_text,
                            category: category.to_string(),
                            show_timestamp,
                            buttons: music_buttons.clone(),
                        };

                        let timestamp = if show_timestamp { media.start_time_unix } else { None };

                        let rpc_state = app_handle_clone.state::<DiscordRpcState>();
                        let result = set_activity_internal(
                            &rpc_state,
                            &music_config,
                            None,
                            None,
                            timestamp,
                        );

                        if result.is_err() {
                            let _ = connect_rpc_internal(&rpc_state, APP_ID);
                            let _ = set_activity_internal(
                                &rpc_state,
                                &music_config,
                                None,
                                None,
                                timestamp,
                            );
                        }

                        let connected = *rpc_state.connected.lock().unwrap_or_else(|e| e.into_inner());

                        let _ = app_handle_clone.emit(
                            "rpc-status-changed",
                            RpcStatusEvent {
                                connected,
                                current_app: Some(display_name),
                                exe_name: Some(media.source_app_id.clone()),
                                window_title: fg_app_label.clone(),
                                category: Some(category.to_string()),
                                details: Some(details_text),
                                state: Some(final_state),
                                large_image_key: media_art_url.clone(),
                                start_timestamp: if show_timestamp { media.start_time_unix } else { None },
                                is_music: is_confirmed_music,
                                small_image_key: small_icon.clone(),
                                album_name: if media.album.is_empty() { None } else { Some(media.album) },
                                listen_url: music_buttons.iter().find(|b| b.label != "Get this status").map(|b| b.url.clone()),
                                playback_status: Some(media.playback_status.clone()),
                                position_secs: media.position_secs,
                                duration_secs: media.duration_secs,
                            },
                        );

                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        last_activity_sent = now;
                        media_handled = true;
                    }
                }
                // No media playing — clear state and force app re-detection
                if !media_handled && !last_media_key.is_empty() {
                    last_media_key.clear();
                    media_art_url = None;
                    media_player_icon = None;
                    last_exe.clear(); // Force re-emit on next app detection
                }
            }
            if media_handled {
                tokio::time::sleep(Duration::from_millis(poll_interval)).await;
                continue; // Skip normal app detection
            }

            // ═══ NORMAL APP DETECTION ═══
            // println!("[Poller] Normal app detection");
            if let Some(fg) = get_foreground_app_internal() {
                // println!("[Poller] Foreground: exe={} title={}", fg.exe_name, fg.window_title);
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
                            large_image_key: resolved.large_image_key.clone(),
                        });
                    }
                }

                // Send activity on app change OR heartbeat (keeps RPC connection alive)
                let app_changed = fg.exe_name != last_exe;
                let heartbeat_due = now - last_activity_sent >= HEARTBEAT_SECS;
                let force_emit = skip_browser_video; // Always emit when transitioning from browser video
                // println!("[Poller] app_changed={} force_emit={}", app_changed, force_emit);
                if app_changed || heartbeat_due || force_emit {
                    last_exe = fg.exe_name.clone();

                    // Use first-seen timestamp (never resets during session)
                    let original_start = *app_first_seen
                        .entry(fg.exe_name.clone())
                        .or_insert(now as i64);
                    // Heartbeat trick: re-send with same elapsed time but a "fresh" call
                    // so Discord doesn't silently drop the presence after ~2h
                    let app_start = original_start;

                    let rpc_state = app_handle_clone.state::<DiscordRpcState>();

                    let (app_config, details_override, state_override) =
                        resolve_app_config(&fg.exe_name, &fg.window_title, &config);

                    // For browsers: extract site name and show it
                    let is_browser_app = super::media_session::BROWSER_NAMES.iter().any(|b| fg.exe_name.to_lowercase().contains(b));
                    let (details_override, state_override) = if is_browser_app && !fg.window_title.is_empty() {
                        let browsers_suffixes = [" — Mozilla Firefox", " - Mozilla Firefox",
                            " — Google Chrome", " - Google Chrome",
                            " — Microsoft Edge", " - Microsoft Edge",
                            " — Brave", " - Brave", " — Opera", " - Opera",
                            " — Vivaldi", " - Vivaldi", " — Arc", " - Arc"];
                        let mut clean = fg.window_title.clone();
                        for b in &browsers_suffixes {
                            if let Some(pos) = clean.rfind(b) {
                                clean = clean[..pos].to_string();
                                break;
                            }
                        }
                        let site = clean.split(" — ").next()
                            .unwrap_or(&clean)
                            .split(" - ").next()
                            .unwrap_or(&clean)
                            .split(" | ").next()
                            .unwrap_or(&clean)
                            .trim()
                            .to_string();
                        let site = if site.len() > 60 {
                            format!("{}…", &site.chars().take(58).collect::<String>())
                        } else {
                            site
                        };
                        if !site.is_empty() {
                            (Some(format!("Sur {}", site)), state_override)
                        } else {
                            (details_override, state_override)
                        }
                    } else {
                        (details_override, state_override)
                    };

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

                    // println!("[Poller] Emitting event: connected={} current_app={:?}", connected, current_app);
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
                            is_music: false,
                            small_image_key: None,
                            album_name: None,
                            listen_url: None,
                            playback_status: None,
                            position_secs: None,
                            duration_secs: None,
                        },
                    );

                    last_activity_sent = now;
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
            is_music: false,
            small_image_key: None,
            album_name: None,
            listen_url: None,
            playback_status: None,
            position_secs: None,
            duration_secs: None,
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
