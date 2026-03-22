use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ButtonConfig {
    pub label: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub name: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub state: String,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_key: Option<String>,
    pub small_image_text: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default = "default_true")]
    pub show_timestamp: bool,
    #[serde(default)]
    pub buttons: Vec<ButtonConfig>,
}

fn default_true() -> bool {
    true
}

fn default_listening() -> String {
    "listening".to_string()
}

fn default_player() -> String {
    "player".to_string()
}

fn default_player_logo() -> String {
    "player".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MusicConfig {
    // Présence
    #[serde(default = "default_listening")]
    pub activity_type: String,
    #[serde(default = "default_player")]
    pub display_text: String,
    #[serde(default)]
    pub display_text_custom: String,
    #[serde(default = "default_player")]
    pub profile_text: String,
    #[serde(default)]
    pub profile_text_custom: String,

    // Infos musique
    #[serde(default)]
    pub title_artist_one_line: bool,
    #[serde(default)]
    pub artist_album_one_line: bool,
    #[serde(default)]
    pub reverse_title_artist: bool,
    #[serde(default)]
    pub prefix_artist_by: bool,
    #[serde(default)]
    pub prefix_album_on: bool,
    #[serde(default = "default_true")]
    pub show_album: bool,
    #[serde(default = "default_true")]
    pub show_album_when_no_artist: bool,
    #[serde(default = "default_true")]
    pub show_playback_info: bool,
    #[serde(default)]
    pub hide_music_info: bool,

    // Média en pause
    #[serde(default = "default_true")]
    pub show_paused: bool,
    #[serde(default = "default_true")]
    pub show_pause_icon: bool,
    #[serde(default = "default_true")]
    pub freeze_progress_when_paused: bool,
    #[serde(default)]
    pub show_paused_duration: bool,

    // Lecteurs hors-ligne
    #[serde(default)]
    pub show_play_icon: bool,
    #[serde(default = "default_true")]
    pub show_player_logo: bool,

    // Boutons
    #[serde(default = "default_true")]
    pub show_get_status_button: bool,
    #[serde(default = "default_true")]
    pub show_listen_button: bool,

    // Divers
    #[serde(default = "default_player_logo")]
    pub fallback_cover: String,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            activity_type: default_listening(),
            display_text: default_player(),
            display_text_custom: String::new(),
            profile_text: default_player(),
            profile_text_custom: String::new(),
            title_artist_one_line: false,
            artist_album_one_line: false,
            reverse_title_artist: false,
            prefix_artist_by: false,
            prefix_album_on: false,
            show_album: true,
            show_album_when_no_artist: true,
            show_playback_info: true,
            hide_music_info: false,
            show_paused: true,
            show_pause_icon: true,
            freeze_progress_when_paused: true,
            show_paused_duration: false,
            show_play_icon: false,
            show_player_logo: true,
            show_get_status_button: true,
            show_listen_button: true,
            fallback_cover: default_player_logo(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DefaultConfig {
    pub details: String,
    #[serde(default)]
    pub state: String,
    #[serde(default = "default_image_key")]
    pub large_image_key: String,
}

fn default_image_key() -> String {
    "default_icon".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RpcConfig {
    #[serde(default)]
    pub apps: HashMap<String, AppConfig>,
    #[serde(default = "default_config")]
    pub default: DefaultConfig,
    #[serde(default)]
    pub discord_app_id: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub steamgriddb_api_key: String,
    #[serde(default = "default_true")]
    pub music_enabled: bool,
    #[serde(default)]
    pub music_config: MusicConfig,
    #[serde(default = "default_true")]
    pub auto_start_service: bool,
    #[serde(default)]
    pub hide_tray_icon: bool,
}

fn default_poll_interval() -> u64 {
    3000
}

fn default_config() -> DefaultConfig {
    DefaultConfig {
        details: "Using {app_name}".to_string(),
        state: String::new(),
        large_image_key: "default_icon".to_string(),
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            apps: HashMap::new(),
            default: default_config(),
            discord_app_id: String::new(),
            poll_interval_ms: 3000,
            steamgriddb_api_key: String::new(),
            music_enabled: true,
            music_config: MusicConfig::default(),
            auto_start_service: true,
            hide_tray_icon: false,
        }
    }
}

/// Built-in database of known apps — auto-detected, no manual config needed
pub fn get_builtin_apps() -> HashMap<String, AppConfig> {
    let mut db = HashMap::new();

    // (exe, name, details, state, category)
    // NO hardcoded icons — all icons are auto-fetched from Iconify API at runtime
    // The poller automatically searches for colorful icons and saves them to config.json
    let apps: Vec<(&str, &str, &str, &str, &str)> = vec![
        // Browsers
        ("chrome.exe", "Google Chrome", "Browsing the web", "", "browser"),
        ("firefox.exe", "Firefox", "Browsing the web", "", "browser"),
        ("msedge.exe", "Microsoft Edge", "Browsing the web", "", "browser"),
        ("brave.exe", "Brave", "Browsing the web", "", "browser"),
        ("opera.exe", "Opera", "Browsing the web", "", "browser"),
        ("vivaldi.exe", "Vivaldi", "Browsing the web", "", "browser"),
        ("arc.exe", "Arc", "Browsing the web", "", "browser"),
        // Code editors / IDEs
        ("code.exe", "Visual Studio Code", "Coding", "Writing code", "dev"),
        ("code - insiders.exe", "VS Code Insiders", "Coding", "Writing code", "dev"),
        ("devenv.exe", "Visual Studio", "Coding", "Developing", "dev"),
        ("idea64.exe", "IntelliJ IDEA", "Coding", "Developing", "dev"),
        ("pycharm64.exe", "PyCharm", "Coding", "Python dev", "dev"),
        ("webstorm64.exe", "WebStorm", "Coding", "Web dev", "dev"),
        ("rider64.exe", "JetBrains Rider", "Coding", "C# dev", "dev"),
        ("clion64.exe", "CLion", "Coding", "C/C++ dev", "dev"),
        ("goland64.exe", "GoLand", "Coding", "Go dev", "dev"),
        ("sublime_text.exe", "Sublime Text", "Coding", "Editing", "dev"),
        ("notepad++.exe", "Notepad++", "Editing", "Text editing", "dev"),
        ("cursor.exe", "Cursor", "Coding", "AI-assisted coding", "dev"),
        ("zed.exe", "Zed", "Coding", "Writing code", "dev"),
        ("windsurf.exe", "Windsurf", "Coding", "AI-assisted coding", "dev"),
        // AI
        ("claude.exe", "Claude", "Using Claude", "AI Assistant", "dev"),
        ("chatgpt.exe", "ChatGPT", "Using ChatGPT", "AI Assistant", "dev"),
        ("copilot.exe", "Copilot", "Using Copilot", "AI Assistant", "dev"),
        // Terminals
        ("windowsterminal.exe", "Windows Terminal", "In the terminal", "", "dev"),
        ("wt.exe", "Windows Terminal", "In the terminal", "", "dev"),
        ("powershell.exe", "PowerShell", "In the terminal", "", "dev"),
        ("cmd.exe", "Command Prompt", "In the terminal", "", "dev"),
        ("alacritty.exe", "Alacritty", "In the terminal", "", "dev"),
        ("wezterm-gui.exe", "WezTerm", "In the terminal", "", "dev"),
        // Gaming
        ("steam.exe", "Steam", "On Steam", "Browsing games", "gaming"),
        ("steamwebhelper.exe", "Steam", "On Steam", "", "gaming"),
        ("epicgameslauncher.exe", "Epic Games", "On Epic Games", "", "gaming"),
        ("gog galaxy.exe", "GOG Galaxy", "On GOG Galaxy", "", "gaming"),
        ("battle.net.exe", "Battle.net", "On Battle.net", "", "gaming"),
        ("riotclientux.exe", "Riot Client", "Riot Games", "", "gaming"),
        ("leagueclient.exe", "League of Legends", "Playing LoL", "", "gaming"),
        ("valorant.exe", "Valorant", "Playing Valorant", "", "gaming"),
        ("overwatch.exe", "Overwatch 2", "Playing Overwatch", "", "gaming"),
        ("cs2.exe", "Counter-Strike 2", "Playing CS2", "", "gaming"),
        ("gta5.exe", "GTA V", "Playing GTA V", "", "gaming"),
        ("rdr2.exe", "Red Dead Redemption 2", "Playing RDR2", "", "gaming"),
        ("minecraft.exe", "Minecraft", "Playing Minecraft", "", "gaming"),
        ("javaw.exe", "Minecraft (Java)", "Playing Minecraft", "", "gaming"),
        ("fortnite.exe", "Fortnite", "Playing Fortnite", "", "gaming"),
        ("rocketleague.exe", "Rocket League", "Playing Rocket League", "", "gaming"),
        ("starcitizen.exe", "Star Citizen", "Playing Star Citizen", "", "gaming"),
        ("eldenring.exe", "Elden Ring", "Playing Elden Ring", "", "gaming"),
        ("baldursgate3.exe", "Baldur's Gate 3", "Playing BG3", "", "gaming"),
        ("cyberpunk2077.exe", "Cyberpunk 2077", "Playing Cyberpunk", "", "gaming"),
        ("palworld.exe", "Palworld", "Playing Palworld", "", "gaming"),
        ("helldivers2.exe", "Helldivers 2", "Playing Helldivers 2", "", "gaming"),
        // Communication
        ("discord.exe", "Discord", "On Discord", "Chatting", "social"),
        ("teams.exe", "Microsoft Teams", "On Teams", "", "social"),
        ("slack.exe", "Slack", "On Slack", "", "social"),
        ("telegram.exe", "Telegram", "On Telegram", "", "social"),
        ("whatsapp.exe", "WhatsApp", "On WhatsApp", "", "social"),
        ("signal.exe", "Signal", "On Signal", "", "social"),
        ("zoom.exe", "Zoom", "On Zoom", "In a meeting", "social"),
        // Media
        ("spotify.exe", "Spotify", "Listening to music", "", "media"),
        ("vlc.exe", "VLC", "Watching media", "", "media"),
        ("foobar2000.exe", "foobar2000", "Listening to music", "", "media"),
        ("musicbee.exe", "MusicBee", "Listening to music", "", "media"),
        ("obs64.exe", "OBS Studio", "Streaming / Recording", "", "media"),
        // Creative
        ("photoshop.exe", "Photoshop", "Editing images", "Creative work", "creative"),
        ("illustrator.exe", "Illustrator", "Creating vectors", "", "creative"),
        ("afterfx.exe", "After Effects", "Editing video", "", "creative"),
        ("premiere pro.exe", "Premiere Pro", "Editing video", "", "creative"),
        ("blender.exe", "Blender", "3D Modeling", "", "creative"),
        ("figma.exe", "Figma", "Designing", "UI/UX Design", "creative"),
        ("gimp-2.10.exe", "GIMP", "Editing images", "", "creative"),
        ("krita.exe", "Krita", "Drawing", "Digital art", "creative"),
        ("davinci resolve.exe", "DaVinci Resolve", "Editing video", "", "creative"),
        ("clip studio paint.exe", "Clip Studio Paint", "Drawing", "", "creative"),
        // Productivity
        ("winword.exe", "Microsoft Word", "Writing a document", "", "productivity"),
        ("excel.exe", "Microsoft Excel", "Working on a spreadsheet", "", "productivity"),
        ("powerpnt.exe", "Microsoft PowerPoint", "Making a presentation", "", "productivity"),
        ("outlook.exe", "Outlook", "Checking emails", "", "productivity"),
        ("notion.exe", "Notion", "Taking notes", "", "productivity"),
        ("obsidian.exe", "Obsidian", "Taking notes", "", "productivity"),
        ("notepad.exe", "Notepad", "Editing text", "", "productivity"),
        // System
        ("explorer.exe", "File Explorer", "Browsing files", "", "system"),
        ("taskmgr.exe", "Task Manager", "Managing tasks", "", "system"),
    ];

    for (exe, name, details, state, category) in apps {
        db.insert(
            exe.to_string(),
            AppConfig {
                name: name.to_string(),
                details: details.to_string(),
                state: state.to_string(),
                large_image_key: None, // Auto-fetched from Iconify at runtime
                large_image_text: Some(name.to_string()),
                small_image_key: None,
                small_image_text: None,
                category: category.to_string(),
                show_timestamp: true,
                buttons: vec![],
            },
        );
    }

    db
}

/// Resolve an exe name to its config: user overrides > builtin DB > default
pub fn resolve_app_config(
    exe_name: &str,
    window_title: &str,
    config: &RpcConfig,
) -> (AppConfig, Option<String>, Option<String>) {
    // 1. Check user custom config
    if let Some(custom) = config.apps.get(exe_name) {
        return (custom.clone(), None, None);
    }

    // 2. Check built-in database
    let builtin = get_builtin_apps();
    if let Some(known) = builtin.get(exe_name) {
        return (known.clone(), None, None);
    }

    // 3. Fallback: derive a nice name from exe
    let display_name = if !window_title.is_empty() {
        window_title.to_string()
    } else {
        let base = exe_name.strip_suffix(".exe").unwrap_or(exe_name);
        let mut chars = base.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    let details = config.default.details.replace("{app_name}", &display_name);
    let state_text = config.default.state.replace("{app_name}", &display_name);

    let fallback = AppConfig {
        name: display_name,
        details: String::new(),
        state: String::new(),
        large_image_key: Some(config.default.large_image_key.clone()),
        large_image_text: None,
        small_image_key: None,
        small_image_text: None,
        category: "other".to_string(),
        show_timestamp: true,
        buttons: vec![],
    };
    (fallback, Some(details), Some(state_text))
}

pub fn get_config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("discord-rpc-manager")
}

fn get_config_file() -> PathBuf {
    get_config_dir().join("config.json")
}

pub fn load_config_internal() -> RpcConfig {
    let path = get_config_file();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => RpcConfig::default(),
        }
    } else {
        RpcConfig::default()
    }
}

#[command]
pub fn load_config() -> Result<RpcConfig, String> {
    Ok(load_config_internal())
}

#[command]
pub fn save_config(config: RpcConfig) -> Result<(), String> {
    let dir = get_config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {}", e))?;

    let path = dir.join("config.json");
    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("Serialization error: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

/// Returns the full built-in apps database so the frontend can display it
#[command]
pub fn get_builtin_apps_list() -> Result<HashMap<String, AppConfig>, String> {
    Ok(get_builtin_apps())
}

/// Apps that should be ignored (system processes, background services)
pub fn is_ignored_app(exe_name: &str) -> bool {
    const IGNORED: &[&str] = &[
        "searchhost.exe",
        "searchui.exe",
        "snippingtool.exe",
        "textinputhost.exe",
        "applicationframehost.exe",
        "shellexperiencehost.exe",
        "startmenuexperiencehost.exe",
        "systemsettings.exe",
        "lockapp.exe",
        "securityhealthsystray.exe",
        "runtimebroker.exe",
        "dwm.exe",
        "csrss.exe",
        "svchost.exe",
        "sihost.exe",
        "ctfmon.exe",
        "discord-rpc-manager.exe",
        "msedgewebview2.exe",
        "webviewhost.exe",
        "discord.exe",
        "activity.exe",
    ];
    IGNORED.contains(&exe_name)
}

/// Auto-register a newly discovered app into the user's config.
/// Only adds it if not already in user config AND not in builtin DB.
/// Returns true if a new app was added.
pub fn auto_register_app(exe_name: &str, window_title: &str) -> bool {
    // Skip system/background processes
    if is_ignored_app(exe_name) {
        return false;
    }

    let mut config = load_config_internal();
    let builtin = get_builtin_apps();

    // Skip if already known
    if config.apps.contains_key(exe_name) || builtin.contains_key(exe_name) {
        return false;
    }

    // Derive a nice display name from the exe
    let display_name = if !window_title.is_empty() {
        // Use the window title but trim it if too long (take the app name part)
        let title = window_title.to_string();
        // Many apps show "Document - AppName", try to extract app name
        if let Some(last) = title.rsplit(" - ").next() {
            if last.len() > 2 {
                last.to_string()
            } else {
                title
            }
        } else {
            title
        }
    } else {
        let base = exe_name.strip_suffix(".exe").unwrap_or(exe_name);
        let mut chars = base.chars();
        match chars.next() {
            None => return false,
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    let new_app = AppConfig {
        name: display_name.clone(),
        details: format!("Using {}", display_name),
        state: String::new(),
        large_image_key: None,
        large_image_text: Some(display_name),
        small_image_key: None,
        small_image_text: None,
        category: "discovered".to_string(),
        show_timestamp: true,
        buttons: vec![],
    };

    config.apps.insert(exe_name.to_string(), new_app);

    // Save to disk
    let dir = get_config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("config.json");
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(&path, json);
    }

    true
}
