//! System event logging utilities.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background) + Platform (Native).

pub const EVENTLOG_SUCCESS: u16 = 0x0000;
pub const EVENTLOG_ERROR_TYPE: u16 = 0x0001;
pub const EVENTLOG_WARNING_TYPE: u16 = 0x0002;
pub const EVENTLOG_INFORMATION_TYPE: u16 = 0x0004;
pub const EVENTLOG_AUDIT_SUCCESS: u16 = 0x0008;
pub const EVENTLOG_AUDIT_FAILURE: u16 = 0x0010;

pub const EVENT_ID_USER_ACTION: u32 = 1000;

/// Write a record directly to the native Windows Event Log under Application (on Windows)
/// or the Syslog daemon socket (on Linux).
pub fn log_system_event(source_name: &str, event_type: u16, event_id: u32, message: &str) {
    #[cfg(all(target_os = "windows", feature = "event-log"))]
    unsafe {
        let source_w: Vec<u16> = source_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let handle = windows_sys::Win32::System::EventLog::RegisterEventSourceW(
            std::ptr::null(),
            source_w.as_ptr(),
        );
        if !handle.is_null() {
            let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
            let strings: [*const u16; 1] = [message_w.as_ptr()];

            windows_sys::Win32::System::EventLog::ReportEventW(
                handle,
                event_type,
                0, // category
                event_id,
                std::ptr::null_mut(), // user sid
                1,                    // num strings
                0,                    // data size
                strings.as_ptr(),
                std::ptr::null_mut(), // raw data
            );
            windows_sys::Win32::System::EventLog::DeregisterEventSource(handle);
        }
    }
    #[cfg(all(target_os = "windows", not(feature = "event-log")))]
    {
        let _ = (source_name, event_type, event_id, message);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::net::UnixDatagram;
        if let Ok(socket) = UnixDatagram::unbound() {
            if socket.connect("/dev/log").is_ok() {
                let pri = match event_type {
                    1 => 11, // Facility user (1) * 8 + Severity err (3) = 11
                    2 => 12, // Facility user (1) * 8 + Severity warning (4) = 12
                    _ => 14, // Facility user (1) * 8 + Severity info (6) = 14
                };
                let log_msg = format!(
                    "<{}>{}[{}]: [ID {}] {}",
                    pri,
                    source_name,
                    std::process::id(),
                    event_id,
                    message
                );
                let _ = socket.send(log_msg.as_bytes());
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (source_name, event_type, event_id, message);
    }
}

/// Legacy wrapper for log_system_event.
#[deprecated(since = "1.0.0", note = "Use log_system_event instead")]
pub fn log_windows_event(source_name: &str, event_type: u16, event_id: u32, message: &str) {
    log_system_event(source_name, event_type, event_id, message);
}
