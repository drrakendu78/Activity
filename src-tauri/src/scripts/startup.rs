use tauri::command;

#[cfg(target_os = "windows")]
use auto_launch::AutoLaunch;

#[command]
pub fn enable_auto_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "Discord RPC Manager";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Could not get executable path: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Invalid path")?,
            &["--minimized"],
        );

        auto_launch
            .enable()
            .map_err(|e| format!("Failed to enable auto startup: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto startup is only supported on Windows".to_string())
    }
}

#[command]
pub fn disable_auto_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "Discord RPC Manager";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Could not get executable path: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Invalid path")?,
            &["--minimized"],
        );

        auto_launch
            .disable()
            .map_err(|e| format!("Failed to disable auto startup: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto startup is only supported on Windows".to_string())
    }
}

#[command]
pub fn is_auto_startup_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let app_name = "Discord RPC Manager";
        let app_path = std::env::current_exe()
            .map_err(|e| format!("Could not get executable path: {}", e))?;

        let auto_launch = AutoLaunch::new(
            app_name,
            app_path.to_str().ok_or("Invalid path")?,
            &["--minimized"],
        );

        auto_launch
            .is_enabled()
            .map_err(|e| format!("Failed to check auto startup: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(false)
    }
}
