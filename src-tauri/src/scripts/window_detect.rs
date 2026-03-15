use serde::Serialize;
use tauri::command;

#[derive(Serialize, Clone, Debug)]
pub struct ForegroundApp {
    pub exe_name: String,
    pub window_title: String,
    pub exe_path: String,
}

#[cfg(target_os = "windows")]
pub fn get_foreground_app_internal() -> Option<ForegroundApp> {
    use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return None;
        }

        // Get window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = if title_len > 0 {
            String::from_utf16_lossy(&title_buf[..title_len as usize])
        } else {
            String::new()
        };

        // Get process ID
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        // Open process and get exe path
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        match process {
            Ok(handle) => {
                let mut exe_buf = [0u16; MAX_PATH as usize];
                let mut size = exe_buf.len() as u32;
                let result = QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR(exe_buf.as_mut_ptr()),
                    &mut size,
                );
                let _ = CloseHandle(handle);

                if result.is_ok() && size > 0 {
                    let exe_path = String::from_utf16_lossy(&exe_buf[..size as usize]);
                    let exe_name = std::path::Path::new(&exe_path)
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default()
                        .to_lowercase();

                    Some(ForegroundApp {
                        exe_name,
                        window_title,
                        exe_path,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_app_internal() -> Option<ForegroundApp> {
    None
}

#[command]
pub fn get_active_window() -> Result<Option<ForegroundApp>, String> {
    Ok(get_foreground_app_internal())
}
