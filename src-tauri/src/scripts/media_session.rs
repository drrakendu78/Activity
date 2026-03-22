use serde::Serialize;
use tauri::command;

#[derive(Serialize, Clone, Debug)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub player_name: String,
    pub playback_status: String,
    pub start_time_unix: Option<i64>,
    pub position_secs: Option<f64>,
    pub duration_secs: Option<f64>,
    /// True when media comes from a browser (Firefox, Chrome, Edge, etc.)
    /// Used to skip album art lookup (would find wrong results for videos)
    pub is_browser: bool,
    /// The raw GSMTC source app ID (used to find the audio session for per-app volume)
    pub source_app_id: String,
}

/// Known browser player names
pub const BROWSER_NAMES: &[&str] = &[
    "chrome", "firefox", "edge", "brave", "opera", "vivaldi", "arc", "browser",
];

/// Find the browser window title by enumerating all top-level windows.
/// This works even when the browser is NOT the foreground window.
/// Browser window titles typically contain the site name, e.g.:
///   "Video Title - YouTube — Mozilla Firefox"
///   "Streamer - Twitch - Google Chrome"
#[cfg(target_os = "windows")]
fn find_browser_window_title() -> Option<String> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
    use std::sync::Mutex;

    let results: Mutex<Vec<String>> = Mutex::new(Vec::new());

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};
        use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};

        // Skip invisible windows
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        // Get window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        if title_len == 0 {
            return BOOL(1);
        }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        // Get process ID
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL(1);
        }

        // Get process name
        if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut name_buf = [0u16; 512];
            let mut size = name_buf.len() as u32;
            if QueryFullProcessImageNameW(handle, Default::default(), windows::core::PWSTR(name_buf.as_mut_ptr()), &mut size).is_ok() {
                let exe_path = String::from_utf16_lossy(&name_buf[..size as usize]).to_lowercase();
                let results_ptr = lparam.0 as *const Mutex<Vec<String>>;
                let browser_exes = ["firefox.exe", "chrome.exe", "msedge.exe", "brave.exe", "opera.exe", "vivaldi.exe", "arc.exe"];
                for browser_exe in &browser_exes {
                    if exe_path.ends_with(browser_exe) {
                        if let Ok(mut vec) = (*results_ptr).lock() {
                            vec.push(title);
                        }
                        break;
                    }
                }
            }
            let _ = windows::Win32::Foundation::CloseHandle(handle);
        }

        BOOL(1) // Continue enumeration
    }

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(&results as *const _ as isize));
    }

    let titles = results.into_inner().ok()?;
    // Return the first browser window title that looks meaningful (not empty/minimal)
    titles.into_iter().find(|t| t.len() > 3)
}

/// Parse the browser window title to detect the website.
/// Browser titles follow patterns like:
///   "Video Title - YouTube — Mozilla Firefox"
///   "Page Title - Site Name — Mozilla Firefox"
///   "Page Title - Site Name - Google Chrome"
///   "Page Title - Site Name - Brave"
#[cfg(target_os = "windows")]
fn detect_website_from_window_title() -> Option<String> {
    let window_title = find_browser_window_title()?;
    let lower = window_title.to_lowercase();


    // Known site patterns to search for in the window title
    let sites: &[(&[&str], &str)] = &[
        (&["youtube music"], "YouTube Music"),
        (&["youtube"], "YouTube"),
        (&["twitch"], "Twitch"),
        (&["netflix"], "Netflix"),
        (&["disney+", "disney plus"], "Disney+"),
        (&["prime video", "primevideo"], "Prime Video"),
        (&["crunchyroll"], "Crunchyroll"),
        (&["spotify"], "Spotify"),
        (&["soundcloud"], "SoundCloud"),
        (&["deezer"], "Deezer"),
        (&["apple music"], "Apple Music"),
        (&["tidal"], "TIDAL"),
        (&["dailymotion"], "Dailymotion"),
        (&["vimeo"], "Vimeo"),
        (&["kick.com", " - kick"], "Kick"),
        (&["reddit"], "Reddit"),
        (&["twitter", "x.com"], "X"),
    ];

    for (patterns, site_name) in sites {
        for pattern in *patterns {
            if lower.contains(pattern) {
                return Some(site_name.to_string());
            }
        }
    }

    None
}

/// Detect which website/service is playing media based on GSMTC metadata
/// AND the browser window title (much more reliable for Firefox/Chromium).
fn detect_website(title: &str, artist: &str, album: &str) -> Option<String> {
    // First try: check the browser window title (most reliable)
    #[cfg(target_os = "windows")]
    if let Some(site) = detect_website_from_window_title() {
        return Some(site);
    }

    // Fallback: combine GSMTC metadata for pattern matching
    let all = format!("{} {} {}", title, artist, album).to_lowercase();

    // YouTube — artist field is often the channel name,
    // but GSMTC from YouTube typically sets album to empty or site name
    // Check common YouTube patterns
    if all.contains("youtube") {
        if all.contains("music.youtube") || all.contains("youtube music") {
            return Some("YouTube Music".to_string());
        }
        return Some("YouTube".to_string());
    }

    // Twitch — GSMTC from Twitch often has "Twitch" in the metadata
    if all.contains("twitch") {
        return Some("Twitch".to_string());
    }

    // Netflix
    if all.contains("netflix") {
        return Some("Netflix".to_string());
    }

    // Disney+
    if all.contains("disney") {
        return Some("Disney+".to_string());
    }

    // Prime Video
    if all.contains("prime video") || all.contains("primevideo") || all.contains("amazon prime") {
        return Some("Prime Video".to_string());
    }

    // Crunchyroll
    if all.contains("crunchyroll") {
        return Some("Crunchyroll".to_string());
    }

    // Spotify Web
    if all.contains("spotify") {
        return Some("Spotify".to_string());
    }

    // SoundCloud
    if all.contains("soundcloud") {
        return Some("SoundCloud".to_string());
    }

    // Deezer Web
    if all.contains("deezer") {
        return Some("Deezer".to_string());
    }

    // Apple Music web
    if all.contains("apple music") {
        return Some("Apple Music".to_string());
    }

    // TIDAL web
    if all.contains("tidal") {
        return Some("TIDAL".to_string());
    }

    // Dailymotion
    if all.contains("dailymotion") {
        return Some("Dailymotion".to_string());
    }

    // Vimeo
    if all.contains("vimeo") {
        return Some("Vimeo".to_string());
    }

    // Kick
    if all.contains("kick.com") || all.contains(" - kick") {
        return Some("Kick".to_string());
    }

    // Reddit (video player)
    if all.contains("reddit") {
        return Some("Reddit".to_string());
    }

    // Twitter/X
    if all.contains("twitter") || all.contains(" / x") || all.contains("x.com") {
        return Some("X".to_string());
    }

    None
}

/// Check if a string looks like a hex hash (Chromium browser GSMTC IDs)
fn is_hex_hash(s: &str) -> bool {
    !s.is_empty() && s.len() >= 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Try to detect the active browser by checking which browser processes are running.
/// Uses a simple approach: check if known browser exes exist in common paths.
#[cfg(target_os = "windows")]
fn detect_browser_from_processes() -> Option<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // Use tasklist to check for running browsers (most reliable cross-version approach)
    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();

    // Check in priority order (most specific first)
    // Firefox uses GSMTC natively with its name, so if we're here it's likely Chromium
    let browsers = [
        ("brave.exe", "Brave"),
        ("vivaldi.exe", "Vivaldi"),
        ("opera.exe", "Opera"),
        ("arc.exe", "Arc"),
        ("msedge.exe", "Edge"),
        ("chrome.exe", "Chrome"),
        ("firefox.exe", "Firefox"),
    ];

    for (exe, name) in &browsers {
        if text.contains(exe) {
            return Some(name.to_string());
        }
    }
    None
}

/// Map known GSMTC source app IDs to friendly player names
fn friendly_player_name(source_app_id: &str) -> String {
    let id = source_app_id.to_lowercase();
    if id.contains("spotify") { return "Spotify".to_string(); }
    if id.contains("apple") || id.contains("amp") || id.contains("music.ui") { return "Apple Music".to_string(); }
    if id.contains("foobar2000") { return "foobar2000".to_string(); }
    if id.contains("musicbee") { return "MusicBee".to_string(); }
    if id.contains("vlc") { return "VLC".to_string(); }
    if id.contains("aimp") { return "AIMP".to_string(); }
    if id.contains("dopamine") { return "Dopamine".to_string(); }
    if id.contains("winamp") { return "Winamp".to_string(); }
    if id.contains("itunes") { return "iTunes".to_string(); }
    if id.contains("tidal") { return "TIDAL".to_string(); }
    if id.contains("deezer") { return "Deezer".to_string(); }
    if id.contains("amazon") { return "Amazon Music".to_string(); }
    if id.contains("youtube") || id.contains("ytmusic") { return "YouTube Music".to_string(); }
    if id.contains("chrome") { return "Chrome".to_string(); }
    if id.contains("msedge") || id.contains("edge") { return "Edge".to_string(); }
    if id.contains("firefox") { return "Firefox".to_string(); }
    if id.contains("brave") { return "Brave".to_string(); }
    if id.contains("opera") { return "Opera".to_string(); }
    if id.contains("vivaldi") { return "Vivaldi".to_string(); }
    if id.contains("arc") { return "Arc".to_string(); }
    if id.contains("mpv") { return "mpv".to_string(); }
    if id.contains("groove") || id.contains("zune") { return "Groove Music".to_string(); }
    if id.contains("mediamonkey") { return "MediaMonkey".to_string(); }
    if id.contains("plex") { return "Plex".to_string(); }
    if id.contains("soundcloud") { return "SoundCloud".to_string(); }
    if id.contains("obs") { return "OBS".to_string(); }

    // Chromium-based browsers report hex hash IDs (e.g. "308046B0AF4A39CB")
    // Try to detect which browser is actually running
    #[cfg(target_os = "windows")]
    if is_hex_hash(source_app_id) {
        if let Some(browser) = detect_browser_from_processes() {
            return browser;
        }
        return "Browser".to_string();
    }

    // Fallback: clean up the ID
    let name = source_app_id
        .split('!')
        .last()
        .unwrap_or(source_app_id)
        .replace('_', " ")
        .replace(".exe", "");
    if name.is_empty() { "Unknown Player".to_string() } else { name }
}

#[cfg(target_os = "windows")]
pub async fn get_current_media() -> Option<MediaInfo> {
    tokio::task::spawn_blocking(|| {
        get_current_media_sync()
    })
    .await
    .ok()?
}

#[cfg(target_os = "windows")]
fn get_current_media_sync() -> Option<MediaInfo> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .get()
        .ok()?;

    // Prefer non-browser Playing sessions, then browser Playing, then Paused
    let session = {
        let mut app_playing = None;     // Non-browser playing (e.g., Spotify)
        let mut browser_playing = None; // Browser playing (e.g., YouTube in Firefox)
        let mut fallback_session = None;
        if let Ok(sessions) = manager.GetSessions() {
            for i in 0..sessions.Size().unwrap_or(0) {
                if let Ok(s) = sessions.GetAt(i) {
                    if let Ok(info) = s.GetPlaybackInfo() {
                        if let Ok(status) = info.PlaybackStatus() {
                            let source_id = s.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default();
                            let name = friendly_player_name(&source_id).to_lowercase();
                            let is_browser = BROWSER_NAMES.iter().any(|b| name.contains(b));

                            if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                                if !is_browser && app_playing.is_none() {
                                    app_playing = Some(s);
                                } else if is_browser && browser_playing.is_none() {
                                    browser_playing = Some(s);
                                }
                            } else if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused && fallback_session.is_none() {
                                fallback_session = Some(s);
                            }
                        }
                    }
                }
            }
        }
        app_playing.or(browser_playing).or(fallback_session).or_else(|| manager.GetCurrentSession().ok())?
    };

    // Get player name
    let source_id = session.SourceAppUserModelId().ok()?.to_string();
    let player_name = friendly_player_name(&source_id);

    // Check playback status
    let playback_info = session.GetPlaybackInfo().ok()?;
    let status = playback_info.PlaybackStatus().ok()?;
    let playback_status = match status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "Playing",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "Paused",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => "Stopped",
        _ => "Unknown",
    };

    // Get media properties (title, artist, album)
    let properties = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = properties.Title().ok()?.to_string();
    let artist = properties.Artist().ok()?.to_string();
    let album = properties.AlbumTitle().ok()?.to_string();

    // Skip if no meaningful title
    if title.is_empty() {
        return None;
    }


    // Get timeline for elapsed time, position, and duration
    let mut start_time_unix = None;
    let mut position_secs_val: Option<f64> = None;
    let mut duration_secs_val: Option<f64> = None;

    if let Ok(timeline) = session.GetTimelineProperties() {
        if let Ok(position) = timeline.Position() {
            let pos = position.Duration as f64 / 10_000_000.0;
            position_secs_val = Some(pos);

            // Only calculate start_time_unix when playing
            if playback_status == "Playing" {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                start_time_unix = Some(now - pos as i64);
            }
        }
        if let Ok(end_time) = timeline.EndTime() {
            let dur = end_time.Duration as f64 / 10_000_000.0;
            if dur > 0.0 {
                duration_secs_val = Some(dur);
            }
        }
    }

    let is_browser = BROWSER_NAMES.iter().any(|b| player_name.to_lowercase().contains(b));

    // When media comes from a browser, detect the actual website/service.
    // Strategy: 1) Check the browser window title (most reliable — contains site name)
    //           2) Check GSMTC metadata (title, artist, album fields)
    //           3) Fallback to "YouTube" (most common browser media source)
    let effective_player = if is_browser {
        detect_website(&title, &artist, &album).unwrap_or_else(|| "YouTube".to_string())
    } else {
        player_name.clone()
    };

    Some(MediaInfo {
        title,
        artist,
        album,
        player_name: effective_player,
        playback_status: playback_status.to_string(),
        start_time_unix,
        position_secs: position_secs_val,
        duration_secs: duration_secs_val,
        is_browser,
        source_app_id: source_id,
    })
}

/// Helper: obtain the current GSMTC session and run a closure on it
#[cfg(target_os = "windows")]
fn with_media_session<F>(f: F) -> Result<(), String>
where
    F: FnOnce(&windows::Media::Control::GlobalSystemMediaTransportControlsSession) -> windows::core::Result<()>,
{
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;

    // For controls: prefer non-browser apps (playing or paused) over browser sessions
    let session = {
        let mut app_playing = None;
        let mut app_paused = None;
        let mut browser_playing = None;
        let mut fallback = None;
        if let Ok(sessions) = manager.GetSessions() {
            for i in 0..sessions.Size().unwrap_or(0) {
                if let Ok(s) = sessions.GetAt(i) {
                    if let Ok(info) = s.GetPlaybackInfo() {
                        if let Ok(status) = info.PlaybackStatus() {
                            let source_id = s.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default();
                            let name = friendly_player_name(&source_id).to_lowercase();
                            let is_browser = BROWSER_NAMES.iter().any(|b| name.contains(b));

                            match status {
                                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => {
                                    if !is_browser && app_playing.is_none() {
                                        app_playing = Some(s);
                                    } else if is_browser && browser_playing.is_none() {
                                        browser_playing = Some(s);
                                    }
                                }
                                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => {
                                    if !is_browser && app_paused.is_none() {
                                        app_paused = Some(s);
                                    } else if fallback.is_none() {
                                        fallback = Some(s);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        // Priority: app playing > app paused > browser playing > any fallback > system default
        app_playing.or(app_paused).or(browser_playing).or(fallback)
            .or_else(|| manager.GetCurrentSession().ok())
            .ok_or_else(|| "No media session found".to_string())?
    };

    f(&session).map_err(|e| e.to_string())
}

/// Toggle play/pause on the current media session
#[cfg(target_os = "windows")]
#[command]
pub async fn media_play_pause() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        with_media_session(|session| {
            session.TryTogglePlayPauseAsync()?.get()?;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Skip to next track
#[cfg(target_os = "windows")]
#[command]
pub async fn media_next() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        with_media_session(|session| {
            session.TrySkipNextAsync()?.get()?;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Skip to previous track
#[cfg(target_os = "windows")]
#[command]
pub async fn media_previous() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        with_media_session(|session| {
            session.TrySkipPreviousAsync()?.get()?;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Seek to a specific position (in seconds) in the current media
#[cfg(target_os = "windows")]
#[command]
pub async fn media_seek(position_secs: f64) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        with_media_session(|session| {
            let ticks = (position_secs * 10_000_000.0) as i64;
            session.TryChangePlaybackPositionAsync(ticks)?.get()?;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn media_seek(_position_secs: f64) -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn media_play_pause() -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn media_next() -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn media_previous() -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
pub async fn get_current_media() -> Option<MediaInfo> {
    None
}

/// Tauri command to expose media info to frontend
#[command]
pub async fn get_current_media_cmd() -> Option<MediaInfo> {
    get_current_media().await
}

// ═══════════════════════════════════════════════════
//  PER-APP VOLUME CONTROL via Windows Audio Session API
// ═══════════════════════════════════════════════════

/// Find ALL audio sessions matching a process name and collect their ISimpleAudioVolume handles.
/// Falls back to system master volume if no matching session is found.
#[cfg(target_os = "windows")]
fn find_app_audio_sessions(exe_name: &str) -> Result<Vec<windows::Win32::Media::Audio::ISimpleAudioVolume>, String> {
    use windows::Win32::Media::Audio::{
        eRender, eConsole,
        IMMDeviceEnumerator, MMDeviceEnumerator,
        IAudioSessionManager2, IAudioSessionEnumerator,
        IAudioSessionControl2,
        ISimpleAudioVolume,
    };
    use windows::Win32::System::Com::{
        CoInitializeEx, CoCreateInstance, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
    use windows::core::Interface;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| e.to_string())?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let session_mgr: IAudioSessionManager2 = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())?;
        let session_enum: IAudioSessionEnumerator = session_mgr
            .GetSessionEnumerator()
            .map_err(|e| e.to_string())?;
        let count = session_enum.GetCount().map_err(|e| e.to_string())?;

        let exe_lower = exe_name.to_lowercase();
        let mut matched: Vec<ISimpleAudioVolume> = Vec::new();

        for i in 0..count {
            let Ok(session) = session_enum.GetSession(i) else { continue };
            let ctrl2: IAudioSessionControl2 = match session.cast() { Ok(c) => c, Err(_) => continue };
            let pid = ctrl2.GetProcessId().unwrap_or(0);
            if pid == 0 { continue; }

            let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { continue };
            let mut buf = [0u16; 260];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, Default::default(), windows::core::PWSTR(buf.as_mut_ptr()), &mut len);
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            if ok.is_err() { continue; }

            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let proc_name = path.rsplit('\\').next().unwrap_or("").to_lowercase();

            let exe_keywords: Vec<&str> = exe_lower.split(|c: char| !c.is_alphanumeric()).filter(|s| s.len() > 2).collect();
            let related: &[&[&str]] = &[
                &["applemusic", "amplibraryagent", "amp"],
                &["spotify"],
            ];
            let proc_base = proc_name.replace(".exe", "");
            let is_related = related.iter().any(|group| {
                let proc_in_group = group.iter().any(|g| proc_base.contains(g));
                let exe_in_group = group.iter().any(|g| exe_lower.contains(g));
                proc_in_group && exe_in_group
            });
            let matches = proc_name.contains(&exe_lower)
                || exe_lower.contains(&proc_base)
                || exe_keywords.iter().any(|kw| proc_name.contains(kw))
                || proc_base.split('.').next().map(|n| exe_keywords.iter().any(|kw| kw.contains(n))).unwrap_or(false)
                || is_related;
            if matches {
                if let Ok(simple_vol) = session.cast::<ISimpleAudioVolume>() {
                    matched.push(simple_vol);
                }
            }
        }

        Ok(matched)
    }
}

/// Get app volume (0.0 - 1.0) and mute state
#[cfg(target_os = "windows")]
#[command]
pub async fn get_system_volume(exe_name: Option<String>) -> Result<(f32, bool), String> {
    tokio::task::spawn_blocking(move || {
        if let Some(ref exe) = exe_name {
            if !exe.is_empty() {
                if let Ok(sessions) = find_app_audio_sessions(exe) {
                    if let Some(vol) = sessions.first() {
                        unsafe {
                            let level = vol.GetMasterVolume().unwrap_or(1.0);
                            let muted = vol.GetMute().map(|b| b.as_bool()).unwrap_or(false);
                            return Ok((level, muted));
                        }
                    }
                }
            }
        }
        with_system_volume(|vol| {
            unsafe {
                let level = vol.GetMasterVolumeLevelScalar()?;
                let muted = vol.GetMute()?.as_bool();
                Ok((level, muted))
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Set app volume (0.0 - 1.0)
#[cfg(target_os = "windows")]
#[command]
pub async fn set_system_volume(level: f32, exe_name: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if let Some(ref exe) = exe_name {
            if !exe.is_empty() {
                if let Ok(sessions) = find_app_audio_sessions(exe) {
                    if !sessions.is_empty() {
                        let clamped = level.clamp(0.0, 1.0);
                        for vol in &sessions {
                            unsafe {
                                let _ = vol.SetMasterVolume(clamped, std::ptr::null());
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }
        with_system_volume(|vol| {
            unsafe {
                vol.SetMasterVolumeLevelScalar(level.clamp(0.0, 1.0), std::ptr::null())?;
                Ok(())
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Toggle mute
#[cfg(target_os = "windows")]
#[command]
pub async fn toggle_mute(exe_name: Option<String>) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        if let Some(ref exe) = exe_name {
            if !exe.is_empty() {
                if let Ok(sessions) = find_app_audio_sessions(exe) {
                    if let Some(vol) = sessions.first() {
                        unsafe {
                            let muted = vol.GetMute().map(|b| b.as_bool()).unwrap_or(false);
                            for v in &sessions {
                                let _ = v.SetMute(!muted, std::ptr::null());
                            }
                            return Ok(!muted);
                        }
                    }
                }
            }
        }
        with_system_volume(|vol| {
            unsafe {
                let muted = vol.GetMute()?.as_bool();
                vol.SetMute(!muted, std::ptr::null())?;
                Ok(!muted)
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Fallback: system master volume
#[cfg(target_os = "windows")]
fn with_system_volume<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume) -> windows::core::Result<R>,
{
    use windows::Win32::Media::Audio::{
        eRender, eConsole,
        IMMDeviceEnumerator, MMDeviceEnumerator,
        Endpoints::IAudioEndpointVolume,
    };
    use windows::Win32::System::Com::{
        CoInitializeEx, CoCreateInstance, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| e.to_string())?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let volume: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())?;
        f(&volume).map_err(|e| e.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn get_system_volume(_exe_name: Option<String>) -> Result<(f32, bool), String> { Ok((1.0, false)) }

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn set_system_volume(_level: f32, _exe_name: Option<String>) -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
#[command]
pub async fn toggle_mute(_exe_name: Option<String>) -> Result<bool, String> { Ok(false) }
