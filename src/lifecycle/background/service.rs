//! System service management utilities.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background) + Platform (Native).

#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::path::Path;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct SERVICE_STATUS {
    pub dwServiceType: u32,
    pub dwCurrentState: u32,
    pub dwControlsAccepted: u32,
    pub dwWin32ExitCode: u32,
    pub dwServiceSpecificExitCode: u32,
    pub dwCheckPoint: u32,
    pub dwWaitHint: u32,
}

#[cfg(target_os = "linux")]
enum InitSystem {
    Systemd,
    OpenRc,
    None,
}

#[cfg(target_os = "linux")]
fn detect_init_system() -> InitSystem {
    // 1. Check if PID 1 is systemd, or if /run/systemd/system exists
    if Path::new("/run/systemd/system").exists() {
        return InitSystem::Systemd;
    }
    if let Ok(comm) = std::fs::read_to_string("/proc/1/comm") {
        if comm.trim() == "systemd" {
            return InitSystem::Systemd;
        }
    }
    // 2. Check if /run/openrc exists
    if Path::new("/run/openrc").exists() {
        return InitSystem::OpenRc;
    }
    InitSystem::None
}

/// Query current state (RUNNING, STOPPED, etc.) of a specific service.
pub fn query_service_status(service_name: &str) -> String {
    #[cfg(all(target_os = "windows", feature = "service"))]
    unsafe {
        let scm = windows_sys::Win32::System::Services::OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::System::Services::SC_MANAGER_CONNECT,
        );
        if !scm.is_null() {
            let service_name_w: Vec<u16> = service_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let svc = windows_sys::Win32::System::Services::OpenServiceW(
                scm,
                service_name_w.as_ptr(),
                windows_sys::Win32::System::Services::SERVICE_QUERY_STATUS,
            );
            if !svc.is_null() {
                let mut status = windows_sys::Win32::System::Services::SERVICE_STATUS {
                    dwServiceType: 0,
                    dwCurrentState: 0,
                    dwControlsAccepted: 0,
                    dwWin32ExitCode: 0,
                    dwServiceSpecificExitCode: 0,
                    dwCheckPoint: 0,
                    dwWaitHint: 0,
                };
                let ok = windows_sys::Win32::System::Services::QueryServiceStatus(
                    svc,
                    &mut status as *mut windows_sys::Win32::System::Services::SERVICE_STATUS,
                );
                windows_sys::Win32::System::Services::CloseServiceHandle(svc);
                windows_sys::Win32::System::Services::CloseServiceHandle(scm);
                if ok != 0 {
                    return match status.dwCurrentState {
                        windows_sys::Win32::System::Services::SERVICE_STOPPED => "STOPPED".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_START_PENDING => "START_PENDING".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_STOP_PENDING => "STOP_PENDING".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_RUNNING => "RUNNING".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_CONTINUE_PENDING => "CONTINUE_PENDING".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_PAUSE_PENDING => "PAUSE_PENDING".to_string(),
                        windows_sys::Win32::System::Services::SERVICE_PAUSED => "PAUSED".to_string(),
                        _ => "UNKNOWN".to_string(),
                    };
                }
            } else {
                windows_sys::Win32::System::Services::CloseServiceHandle(scm);
            }
        }
        "NOT_FOUND".to_string()
    }

    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        let _ = service_name;
        "NOT_FOUND".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        match detect_init_system() {
            InitSystem::Systemd => {
                if let Ok(output) = Command::new("systemctl")
                    .args(&["is-active", service_name])
                    .output()
                {
                    let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if state == "active" {
                        return "RUNNING".to_string();
                    } else if state == "inactive" || state == "failed" {
                        return "STOPPED".to_string();
                    }
                }
            }
            InitSystem::OpenRc => {
                if let Ok(output) = Command::new("rc-service")
                    .args(&[service_name, "status"])
                    .output()
                {
                    let status_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
                    if status_str.contains("started") || status_str.contains("running") {
                        return "RUNNING".to_string();
                    } else if status_str.contains("stopped") {
                        return "STOPPED".to_string();
                    }
                }
            }
            InitSystem::None => {}
        }
        "NOT_FOUND".to_string()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = service_name;
        "Running".to_string()
    }
}

/// Legacy wrapper for query_service_status.
#[deprecated(since = "1.0.0", note = "Use query_service_status instead")]
pub fn query_windows_service_status(service_name: &str) -> String {
    query_service_status(service_name)
}

/// Check if the current process has administrative/root privileges required for service mutations.
pub fn has_admin_privileges() -> bool {
    #[cfg(all(target_os = "windows", feature = "service"))]
    unsafe {
        let scm = windows_sys::Win32::System::Services::OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::System::Services::SC_MANAGER_CREATE_SERVICE,
        );
        if !scm.is_null() {
            windows_sys::Win32::System::Services::CloseServiceHandle(scm);
            true
        } else {
            false
        }
    }
    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        false
    }
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn getuid() -> u32;
        }
        unsafe { getuid() == 0 }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        true
    }
}

/// Start a system service natively on Windows or via detected systemd/OpenRC on Linux.
pub fn start_service(service_name: &str) -> crate::error::Result<()> {
    if !has_admin_privileges() {
        return Err(crate::error::RcommonError::Service("Privilege check failed: Administrator / root privileges are required to modify system services.".to_string()));
    }

    #[cfg(all(target_os = "windows", feature = "service"))]
    unsafe {
        let scm = windows_sys::Win32::System::Services::OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::System::Services::SC_MANAGER_CONNECT,
        );
        if scm.is_null() {
            return Err(crate::error::RcommonError::Service(format!("Failed to open SC Manager: OS error {}", std::io::Error::last_os_error())));
        }
        let service_name_w: Vec<u16> = service_name.encode_utf16().chain(std::iter::once(0)).collect();
        let svc = windows_sys::Win32::System::Services::OpenServiceW(
            scm,
            service_name_w.as_ptr(),
            windows_sys::Win32::System::Services::SERVICE_START,
        );
        if svc.is_null() {
            let err = std::io::Error::last_os_error();
            windows_sys::Win32::System::Services::CloseServiceHandle(scm);
            return Err(crate::error::RcommonError::Service(format!("Failed to open service: {}", err)));
        }
        let ok = windows_sys::Win32::System::Services::StartServiceW(svc, 0, std::ptr::null());
        windows_sys::Win32::System::Services::CloseServiceHandle(svc);
        windows_sys::Win32::System::Services::CloseServiceHandle(scm);
        if ok != 0 {
            Ok(())
        } else {
            Err(crate::error::RcommonError::Service(format!("Failed to start service: {}", std::io::Error::last_os_error())))
        }
    }

    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        let _ = service_name;
        Err(crate::error::RcommonError::Service("Service support is disabled (compiled without 'service' feature)".to_string()))
    }

    #[cfg(target_os = "linux")]
    {
        match detect_init_system() {
            InitSystem::Systemd => {
                let status = Command::new("systemctl")
                    .args(&["start", service_name])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("systemctl start failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute systemctl: {}", e))),
                }
            }
            InitSystem::OpenRc => {
                let status = Command::new("rc-service")
                    .args(&[service_name, "start"])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("rc-service start failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute rc-service: {}", e))),
                }
            }
            InitSystem::None => Err(crate::error::RcommonError::Service("No supported init system detected (systemd or openrc required)".to_string())),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = service_name;
        Ok(())
    }
}

/// Stop a system service natively on Windows or via detected systemd/OpenRC on Linux.
pub fn stop_service(service_name: &str) -> crate::error::Result<()> {
    if !has_admin_privileges() {
        return Err(crate::error::RcommonError::Service("Privilege check failed: Administrator / root privileges are required to modify system services.".to_string()));
    }

    #[cfg(all(target_os = "windows", feature = "service"))]
    unsafe {
        let scm = windows_sys::Win32::System::Services::OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::System::Services::SC_MANAGER_CONNECT,
        );
        if scm.is_null() {
            return Err(crate::error::RcommonError::Service(format!("Failed to open SC Manager: OS error {}", std::io::Error::last_os_error())));
        }
        let service_name_w: Vec<u16> = service_name.encode_utf16().chain(std::iter::once(0)).collect();
        let svc = windows_sys::Win32::System::Services::OpenServiceW(
            scm,
            service_name_w.as_ptr(),
            windows_sys::Win32::System::Services::SERVICE_STOP,
        );
        if svc.is_null() {
            let err = std::io::Error::last_os_error();
            windows_sys::Win32::System::Services::CloseServiceHandle(scm);
            return Err(crate::error::RcommonError::Service(format!("Failed to open service: {}", err)));
        }
        let mut status = windows_sys::Win32::System::Services::SERVICE_STATUS {
            dwServiceType: 0,
            dwCurrentState: 0,
            dwControlsAccepted: 0,
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };
        let ok = windows_sys::Win32::System::Services::ControlService(
            svc,
            windows_sys::Win32::System::Services::SERVICE_CONTROL_STOP,
            &mut status,
        );
        windows_sys::Win32::System::Services::CloseServiceHandle(svc);
        windows_sys::Win32::System::Services::CloseServiceHandle(scm);
        if ok != 0 {
            Ok(())
        } else {
            Err(crate::error::RcommonError::Service(format!("Failed to stop service: {}", std::io::Error::last_os_error())))
        }
    }

    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        let _ = service_name;
        Err(crate::error::RcommonError::Service("Service support is disabled (compiled without 'service' feature)".to_string()))
    }

    #[cfg(target_os = "linux")]
    {
        match detect_init_system() {
            InitSystem::Systemd => {
                let status = Command::new("systemctl")
                    .args(&["stop", service_name])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("systemctl stop failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute systemctl: {}", e))),
                }
            }
            InitSystem::OpenRc => {
                let status = Command::new("rc-service")
                    .args(&[service_name, "stop"])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("rc-service stop failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute rc-service: {}", e))),
                }
            }
            InitSystem::None => Err(crate::error::RcommonError::Service("No supported init system detected (systemd or openrc required)".to_string())),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = service_name;
        Ok(())
    }
}

/// Restart a system service natively on Windows or via detected systemd/OpenRC on Linux.
pub fn restart_service(service_name: &str) -> crate::error::Result<()> {
    if !has_admin_privileges() {
        return Err(crate::error::RcommonError::Service("Privilege check failed: Administrator / root privileges are required to modify system services.".to_string()));
    }

    #[cfg(all(target_os = "windows", feature = "service"))]
    {
        stop_service(service_name)?;
        
        // Wait up to 5 seconds for the service to transition to STOPPED
        let mut stopped = false;
        for _ in 0..100 {
            if query_service_status(service_name) == "STOPPED" {
                stopped = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        if !stopped {
            return Err(crate::error::RcommonError::Service("Failed to restart service: service did not stop in time.".to_string()));
        }
        
        start_service(service_name)?;
        Ok(())
    }

    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        let _ = service_name;
        Err(crate::error::RcommonError::Service("Service support is disabled (compiled without 'service' feature)".to_string()))
    }

    #[cfg(target_os = "linux")]
    {
        match detect_init_system() {
            InitSystem::Systemd => {
                let status = Command::new("systemctl")
                    .args(&["restart", service_name])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("systemctl restart failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute systemctl: {}", e))),
                }
            }
            InitSystem::OpenRc => {
                let status = Command::new("rc-service")
                    .args(&[service_name, "restart"])
                    .status();
                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(crate::error::RcommonError::Service(format!("rc-service restart failed with status: {}", s))),
                    Err(e) => Err(crate::error::RcommonError::Service(format!("Failed to execute rc-service: {}", e))),
                }
            }
            InitSystem::None => Err(crate::error::RcommonError::Service("No supported init system detected (systemd or openrc required)".to_string())),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = service_name;
        Ok(())
    }
}
