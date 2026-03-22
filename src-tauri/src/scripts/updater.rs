use serde::{Deserialize, Serialize};
use tauri::command;

const GITHUB_REPO: &str = "drrakendu78/Activity";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub install_type: String,
}

/// Detect how the app was installed (exe portable vs msi)
fn detect_install_type() -> String {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_str = exe_path.to_string_lossy().to_lowercase();

    if exe_str.contains("program files") || exe_str.contains("programfiles") {
        return "msi".to_string();
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(uninstall) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall") {
            for key_name in uninstall.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(subkey) = uninstall.open_subkey(&key_name) {
                    let display_name: String = subkey.get_value("DisplayName").unwrap_or_default();
                    if display_name.to_lowercase().contains("activity") {
                        let install_source: String = subkey.get_value("InstallSource").unwrap_or_default();
                        if !install_source.is_empty() {
                            return "msi".to_string();
                        }
                    }
                }
            }
        }
    }

    "exe".to_string()
}

/// Compare version strings (e.g., "0.1.0" < "0.2.0")
fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let c = parse(current);
    let l = parse(latest);
    for i in 0..c.len().max(l.len()) {
        let cv = c.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if lv > cv { return true; }
        if lv < cv { return false; }
    }
    false
}

/// Sanitize file name: strip dangerous chars, prevent path traversal
fn safe_file_name(name: &str) -> String {
    let basename = std::path::Path::new(name)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "update.exe".to_string());
    let sanitized: String = basename
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => ch,
        })
        .collect();
    if sanitized.is_empty() || sanitized.starts_with('.') || sanitized.contains("..") {
        "update.exe".to_string()
    } else {
        sanitized
    }
}

#[command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let install_type = detect_install_type();

    let client = reqwest::Client::builder()
        .user_agent("Activity-Updater")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if resp.status() == 404 {
        return Ok(UpdateInfo {
            current_version: current_version.clone(),
            latest_version: current_version,
            has_update: false,
            release_url: format!("https://github.com/{}/releases", GITHUB_REPO),
            release_notes: None,
            download_url: None,
            file_name: None,
            file_size: None,
            install_type,
        });
    }

    let release: GithubRelease = resp.json().await.map_err(|e| e.to_string())?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let has_update = is_newer(&current_version, &latest_version);

    let ext = if install_type == "msi" { ".msi" } else { ".exe" };
    let asset = release.assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.ends_with(ext) && !name.contains("uninstall")
    });

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        release_url: release.html_url,
        release_notes: release.body,
        download_url: asset.map(|a| a.browser_download_url.clone()),
        file_name: asset.map(|a| a.name.clone()),
        file_size: asset.map(|a| a.size),
        install_type,
    })
}

/// Spawn the Activity-Updater.exe with download args, then exit
#[command]
pub async fn start_silent_update(download_url: String, file_name: Option<String>) -> Result<(), String> {
    let url = download_url.trim();
    if url.is_empty() {
        return Err("Missing update download URL".to_string());
    }
    if !url.starts_with("https://") {
        return Err("Update URL must use https://".to_string());
    }

    // SECURITY: Only allow downloads from our GitHub repo
    if !url.starts_with(&format!("https://github.com/{}/releases/", GITHUB_REPO))
        && !url.starts_with("https://objects.githubusercontent.com/")
    {
        return Err("URL de téléchargement non autorisée".to_string());
    }

    let preferred_name = file_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(safe_file_name)
        .unwrap_or_else(|| {
            safe_file_name(url.split('/').last().unwrap_or("update.exe"))
        });

    if preferred_name.is_empty() || preferred_name.len() > 255
        || preferred_name.starts_with('-') || preferred_name.contains('\0')
    {
        return Err("Invalid update file name".to_string());
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot locate app executable: {}", e))?;
    let current_pid = std::process::id();

    // Look for Activity-Updater.exe next to the main executable
    let exe_dir = current_exe.parent().ok_or("Cannot determine app directory")?;
    let updater_exe = exe_dir.join("Activity-Updater.exe");

    if !updater_exe.exists() {
        return Err("Updater not found. Please reinstall the application.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new(&updater_exe)
            .args([
                "--url", url,
                "--name", &preferred_name,
                "--app", &current_exe.to_string_lossy(),
                "--pid", &current_pid.to_string(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Cannot start updater: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("Silent update is currently supported on Windows only".to_string());
    }

    // Exit app so updater can proceed
    std::process::exit(0);
}
