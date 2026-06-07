/// Ensures only one instance of the TUI application is active at any time.
pub struct SingleInstanceGuard {
    #[allow(dead_code)]
    handle: *mut std::ffi::c_void,
}

impl SingleInstanceGuard {
    pub fn try_new() -> Result<Self, String> {
        #[cfg(target_os = "windows")]
        {
            let exe_name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                .unwrap_or_else(|| "rtem".to_string());
            let mutex_name = format!("Local\\{}_SingleInstanceMutex_2026\0", exe_name);
            let name: Vec<u16> = mutex_name.encode_utf16().collect();
            let handle = unsafe {
                windows_sys::Win32::System::Threading::CreateMutexW(
                    std::ptr::null(),
                    1,
                    name.as_ptr(),
                )
            };
            if handle.is_null() {
                return Err("Failed to create single-instance mutex.".to_string());
            }

            let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            if err == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
                unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
                return Err("Another instance of this application is already running.".to_string());
            }

            Ok(SingleInstanceGuard { handle })
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::net::UnixListener;
            let exe_name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                .unwrap_or_else(|| "rtem".to_string());
            let socket_path = format!("/tmp/{}_single_instance.sock", exe_name);
            
            // Try to remove stale socket file
            let _ = std::fs::remove_file(&socket_path);
            
            match UnixListener::bind(&socket_path) {
                Ok(listener) => {
                    let ptr = Box::into_raw(Box::new(listener)) as *mut std::ffi::c_void;
                    Ok(SingleInstanceGuard { handle: ptr })
                }
                Err(_) => Err("Another instance of this application is already running.".to_string()),
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Ok(SingleInstanceGuard { handle: std::ptr::null_mut() })
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if !self.handle.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
        #[cfg(target_os = "linux")]
        if !self.handle.is_null() {
            unsafe {
                let _listener = Box::from_raw(self.handle as *mut std::os::unix::net::UnixListener);
                let exe_name = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "rtem".to_string());
                let socket_path = format!("/tmp/{}_single_instance.sock", exe_name);
                let _ = std::fs::remove_file(socket_path);
            }
        }
    }
}
