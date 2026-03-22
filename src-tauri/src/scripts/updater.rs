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
    pub install_type: String, // "exe" or "msi"
}

/// Detect how the app was installed (exe portable vs msi)
fn detect_install_type() -> String {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_str = exe_path.to_string_lossy().to_lowercase();

    // MSI installs go to Program Files
    if exe_str.contains("program files") || exe_str.contains("programfiles") {
        return "msi".to_string();
    }

    // Check Windows registry for MSI install
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

    // Find the right asset based on install type
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

#[command]
pub async fn download_and_install_update(download_url: String, file_name: String, expected_size: u64) -> Result<(), String> {
    // SECURITY: Only allow downloads from our GitHub repo
    if !download_url.starts_with(&format!("https://github.com/{}/releases/", GITHUB_REPO))
        && !download_url.starts_with("https://objects.githubusercontent.com/")
    {
        return Err("URL de téléchargement non autorisée".to_string());
    }

    // SECURITY: Only allow .exe and .msi files
    let name_lower = file_name.to_lowercase();
    if !name_lower.ends_with(".exe") && !name_lower.ends_with(".msi") {
        return Err("Type de fichier non autorisé".to_string());
    }

    // SECURITY: Sanitize file name (no path traversal)
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("Nom de fichier invalide".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("Activity-Updater")
        .build()
        .map_err(|e| e.to_string())?;

    // Download to temp dir
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(&file_name);

    let resp = client.get(&download_url).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Erreur de téléchargement: {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // SECURITY: Verify file size matches expected (prevents truncated/tampered downloads)
    if expected_size > 0 && bytes.len() as u64 != expected_size {
        return Err(format!(
            "Taille du fichier incorrecte: attendu {} octets, reçu {}",
            expected_size,
            bytes.len()
        ));
    }

    // SECURITY: Max 500MB to prevent disk fill attacks
    if bytes.len() > 500 * 1024 * 1024 {
        return Err("Fichier trop volumineux".to_string());
    }

    std::fs::write(&file_path, &bytes).map_err(|e| e.to_string())?;

    // Launch the installer and exit the app
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if name_lower.ends_with(".msi") {
            Command::new("msiexec")
                .args(["/i", &file_path.to_string_lossy(), "/passive"])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new(&file_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    // Exit app so installer can replace files
    std::process::exit(0);
}
