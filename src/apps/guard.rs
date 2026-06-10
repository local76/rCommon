//! Single-instance application guard helpers.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

#[cfg(target_os = "windows")]
type MutexHandle = windows_sys::Win32::Foundation::HANDLE;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int;
}

#[derive(Debug)]
enum SingleInstanceHandle {
    #[cfg(target_os = "windows")]
    Windows(MutexHandle),
    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    Unix(std::fs::File),
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    None,
}

/// Ensures only one instance of the TUI application is active at any time.
pub struct SingleInstanceGuard {
    #[allow(dead_code)]
    handle: SingleInstanceHandle,
}

fn get_exe_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "libraryApp".to_string())
}

impl SingleInstanceGuard {
    pub fn try_new() -> crate::core::error::Result<Self> {
        #[cfg(target_os = "windows")]
        {
            let exe_name = get_exe_name();
            let mutex_name = format!("Local\\{}_SingleInstanceMutex", exe_name);
            let name: Vec<u16> = mutex_name.encode_utf16().chain(std::iter::once(0)).collect();
            let handle = unsafe {
                windows_sys::Win32::System::Threading::CreateMutexW(
                    std::ptr::null(),
                    1,
                    name.as_ptr(),
                )
            };
            if handle.is_null() {
                return Err(crate::core::error::LibraryError::Guard("Failed to create single-instance mutex.".to_string()));
            }

            let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            if err == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
                unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
                return Err(crate::core::error::LibraryError::Guard("Another instance of this application is already running.".to_string()));
            }

            Ok(SingleInstanceGuard { handle: SingleInstanceHandle::Windows(handle) })
        }
        #[cfg(target_os = "linux")]
        {
            use std::fs::OpenOptions;
            use std::os::unix::io::AsRawFd;
            let exe_name = get_exe_name();
            let socket_path = format!("/tmp/{}_single_instance.sock", exe_name);

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&socket_path)
                .map_err(|e| crate::core::error::LibraryError::Guard(format!("Failed to open lock file: {}", e)))?;

            // Call flock(fd, LOCK_EX | LOCK_NB)
            // LOCK_EX = 2, LOCK_NB = 4
            let res = unsafe { flock(file.as_raw_fd(), 6) };
            if res < 0 {
                return Err(crate::core::error::LibraryError::Guard(
                    "Another instance of this application is already running.".to_string()
                ));
            }

            // Write our PID to the lock file
            use std::io::Write;
            let mut f = file;
            let _ = f.set_len(0);
            let _ = writeln!(f, "{}", std::process::id());
            let _ = f.flush();

            Ok(SingleInstanceGuard { handle: SingleInstanceHandle::Unix(f) })
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Ok(SingleInstanceGuard { handle: SingleInstanceHandle::None })
        }
    }

    /// Try to acquire the single-instance lock. On failure, log the error to the
    /// `file_log`, print to stderr, and exit the process with status 1.
    /// Returns the guard on success.
    ///
    /// `app_label` is used purely for the diagnostic log message.
    #[allow(unused_variables)]
    pub fn try_new_or_exit(app_label: &str) -> Self {
        match Self::try_new() {
            Ok(g) => g,
            Err(e) => {
                #[cfg(feature = "event-log")]
                crate::apps::file_log::log_message(
                    "ERROR",
                    &format!("SingleInstanceGuard blocked launch of {}: {}", app_label, e),
                );
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

impl Drop for SingleInstanceGuard {
    #[allow(irrefutable_let_patterns)]
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if let SingleInstanceHandle::Windows(h) = self.handle {
            if !h.is_null() {
                unsafe {
                    windows_sys::Win32::Foundation::CloseHandle(h);
                }
            }
        }
        #[cfg(target_os = "linux")]
        if let SingleInstanceHandle::Unix(_) = &self.handle {
            let exe_name = get_exe_name();
            let socket_path = format!("/tmp/{}_single_instance.sock", exe_name);
            let _ = std::fs::remove_file(socket_path);
        }
    }
}
