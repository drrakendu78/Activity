use serde::Serialize;
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

static UPDATE_ARGS: OnceLock<UpdateArgs> = OnceLock::new();

/// Max download size: 50 MB
const MAX_DOWNLOAD_SIZE: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
struct UpdateArgs {
    url: String,
    name: String,
    app: String,
    pid: u32,
    expected_sha256: Option<String>,
}

#[derive(Clone, Serialize)]
struct ProgressPayload {
    phase: String,
    percent: u32,
    detail: String,
}

/// Validate URL: must be HTTPS and from a trusted GitHub domain
fn validate_download_url(url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "Invalid download URL".to_string())?;
    if parsed.scheme() != "https" {
        return Err("Download URL must use HTTPS".to_string());
    }
    let host = parsed.host_str().unwrap_or("");
    let trusted = host == "github.com"
        || host.ends_with(".github.com")
        || host == "objects.githubusercontent.com"
        || host.ends_with(".githubusercontent.com");
    if !trusted {
        return Err("Download URL must point to a GitHub domain".to_string());
    }
    Ok(())
}

/// Sanitize file name: strip to basename, replace dangerous chars
fn sanitize_file_name(name: &str) -> String {
    let basename = Path::new(name)
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

/// Validate that the app path looks like the Activity executable
fn validate_app_path(app: &str) -> Result<(), String> {
    let path = Path::new(app);
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !file_name.starts_with("activity") || !file_name.ends_with(".exe") {
        return Err("Invalid app path: must be an Activity executable".to_string());
    }
    let app_str = app.replace('\\', "/");
    if app_str.contains("..") {
        return Err("Invalid app path: path traversal detected".to_string());
    }
    Ok(())
}

/// Validate SHA-256 hex string format
fn validate_sha256_format(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

fn parse_args() -> Option<UpdateArgs> {
    let args: Vec<String> = std::env::args().collect();
    let mut url = None;
    let mut name = None;
    let mut app = None;
    let mut pid = None;
    let mut sha256 = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" if i + 1 < args.len() => {
                url = Some(args[i + 1].clone());
                i += 2;
            }
            "--name" if i + 1 < args.len() => {
                name = Some(args[i + 1].clone());
                i += 2;
            }
            "--app" if i + 1 < args.len() => {
                app = Some(args[i + 1].clone());
                i += 2;
            }
            "--pid" if i + 1 < args.len() => {
                pid = args[i + 1].parse().ok();
                i += 2;
            }
            "--sha256" if i + 1 < args.len() => {
                let h = args[i + 1].trim().to_lowercase();
                if validate_sha256_format(&h) {
                    sha256 = Some(h);
                }
                i += 2;
            }
            _ => i += 1,
        }
    }

    let raw_url = url?;
    let raw_app = app?;
    let raw_pid = pid?;

    if validate_download_url(&raw_url).is_err() {
        return None;
    }
    if validate_app_path(&raw_app).is_err() {
        return None;
    }

    Some(UpdateArgs {
        url: raw_url,
        name: sanitize_file_name(&name.unwrap_or_default()),
        app: raw_app,
        pid: raw_pid,
        expected_sha256: sha256,
    })
}

#[tauri::command]
fn get_update_args() -> Result<UpdateArgs, String> {
    UPDATE_ARGS
        .get()
        .cloned()
        .ok_or_else(|| "No update arguments provided".to_string())
}

#[tauri::command]
async fn start_update(app: AppHandle) -> Result<(), String> {
    let args = UPDATE_ARGS
        .get()
        .cloned()
        .ok_or("No update arguments")?;

    // Phase 1: Wait for main app to exit
    emit_progress(&app, "waiting", 0, "Fermeture d'Activity...");
    wait_for_pid(args.pid).await;
    emit_progress(&app, "waiting", 100, "Activity fermé.");

    // Phase 2: Download
    emit_progress(&app, "downloading", 0, "Début du téléchargement...");
    let installer_path = download_installer(&app, &args.url, &args.name, args.expected_sha256.as_deref()).await?;

    // Phase 3: Install
    emit_progress(&app, "installing", 0, "Installation en cours...");
    run_installer(&installer_path).await?;
    emit_progress(&app, "installing", 100, "Installation terminée.");

    // Phase 4: Relaunch
    emit_progress(&app, "relaunching", 100, "Lancement d'Activity...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Re-validate app path before execution (defense in depth)
    validate_app_path(&args.app)?;
    let app_path = Path::new(&args.app);
    if !app_path.exists() {
        return Err("App executable not found after update".to_string());
    }

    std::process::Command::new(app_path)
        .spawn()
        .map_err(|e| format!("Failed to relaunch: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    app.exit(0);

    Ok(())
}

fn emit_progress(app: &AppHandle, phase: &str, percent: u32, detail: &str) {
    let _ = app.emit(
        "update-progress",
        ProgressPayload {
            phase: phase.to_string(),
            percent,
            detail: detail.to_string(),
        },
    );
}

async fn wait_for_pid(pid: u32) {
    loop {
        if !is_process_running(pid) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}

#[cfg(target_os = "windows")]
fn is_process_running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        ok != 0 && exit_code == STILL_ACTIVE as u32
    }
}

#[cfg(not(target_os = "windows"))]
fn is_process_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

async fn download_installer(
    app: &AppHandle,
    url: &str,
    name: &str,
    expected_sha256: Option<&str>,
) -> Result<PathBuf, String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    // Re-validate URL before downloading (defense in depth)
    validate_download_url(url)?;

    let client = reqwest::Client::builder()
        .user_agent("Activity-Updater/1.0")
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    // Validate final URL after redirects
    let final_url = response.url();
    if final_url.scheme() != "https" {
        return Err("Redirect to non-HTTPS URL is not allowed".to_string());
    }
    let final_host = final_url.host_str().unwrap_or("");
    let trusted = final_host == "github.com"
        || final_host.ends_with(".github.com")
        || final_host == "objects.githubusercontent.com"
        || final_host.ends_with(".githubusercontent.com");
    if !trusted {
        return Err("Redirect to untrusted domain is not allowed".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("Download returned status {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    let file_name = if name.is_empty() {
        sanitize_file_name(url.split('/').last().unwrap_or("update.exe"))
    } else {
        sanitize_file_name(name)
    };

    let temp_dir = std::env::temp_dir().join("activity-updater");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Cannot create temp dir: {}", e))?;
    let dest = temp_dir.join(&file_name);

    // Verify dest stays within temp_dir
    let canonical_dir = temp_dir
        .canonicalize()
        .map_err(|e| format!("Path error: {}", e))?;
    let canonical_parent = dest.parent()
        .ok_or("Invalid dest path")?
        .canonicalize()
        .map_err(|e| format!("Path error: {}", e))?;
    if !canonical_parent.starts_with(&canonical_dir) {
        return Err("Path traversal detected".to_string());
    }

    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| format!("Cannot create file: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_percent: u32 = 0;
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {}", e))?;
        downloaded += chunk.len() as u64;
        if downloaded > MAX_DOWNLOAD_SIZE {
            drop(file);
            let _ = tokio::fs::remove_file(&dest).await;
            return Err("Download too large (max 50 MB)".to_string());
        }

        // Hash the chunk as we download
        hasher.update(&chunk);

        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;

        let percent = if total_size > 0 {
            ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u32
        } else {
            0
        };

        if percent != last_percent {
            last_percent = percent;
            emit_progress(
                app,
                "downloading",
                percent,
                &format!("Téléchargement... {}%", percent),
            );
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    // Verify SHA-256 integrity
    if let Some(expected) = expected_sha256 {
        let computed = format!("{:x}", hasher.finalize());
        if computed != expected {
            let _ = tokio::fs::remove_file(&dest).await;
            return Err(format!(
                "Intégrité du fichier invalide!\nAttendu: {}\nObtenu: {}",
                expected, computed
            ));
        }
        emit_progress(app, "downloading", 100, "Intégrité vérifiée ✓");
    }

    emit_progress(app, "downloading", 100, "Téléchargement terminé.");
    Ok(dest)
}

async fn run_installer(path: &Path) -> Result<(), String> {
    // Validate installer is in our temp directory
    let temp_dir = std::env::temp_dir().join("activity-updater");
    if let (Ok(canon_dir), Ok(canon_path)) = (temp_dir.canonicalize(), path.canonicalize()) {
        if !canon_path.starts_with(&canon_dir) {
            return Err("Installer path outside temp directory".to_string());
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext != "exe" && ext != "msi" {
        return Err("Unsupported installer type".to_string());
    }

    let child = if ext == "msi" {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new("msiexec.exe")
                .args(["/i", &path.to_string_lossy(), "/qn", "/norestart"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err("MSI install only supported on Windows".to_string());
        }
    } else {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(path)
                .args(["/S"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Err("EXE install only supported on Windows".to_string());
        }
    };

    let mut process = child.map_err(|e| format!("Failed to run installer: {}", e))?;
    let status = tokio::task::spawn_blocking(move || process.wait())
        .await
        .map_err(|e| format!("Wait error: {}", e))?
        .map_err(|e| format!("Installer error: {}", e))?;

    if !status.success() {
        return Err(format!("Installer exited with code {:?}", status.code()));
    }

    Ok(())
}

pub fn run() {
    if let Some(args) = parse_args() {
        let _ = UPDATE_ARGS.set(args);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_update_args, start_update])
        .run(tauri::generate_context!())
        .expect("error while running updater");
}
