//! Background Processes lifecycle.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background).
//!
//! Part of Execution State (Lifecycle).
//! Silent running, services, daemons, no UI focus needed.

// Background code moved here (services, logging, notifications, clipboard for non-UI use).

#[cfg(feature = "service")]
pub mod service;
#[cfg(feature = "event-log")]
pub mod event_log;
#[cfg(feature = "event-log")]
pub mod file_log;
#[cfg(feature = "notification")]
pub mod notification;
#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod daemon;  // Power/priority for daemons.
pub mod worker;

// Re-exports
pub use worker::{WorkerEvent, Worker, SampleWorker, spawn_background_task};
#[cfg(feature = "service")]
pub use service::{
    SERVICE_STATUS, query_service_status, query_windows_service_status, has_admin_privileges, start_service, stop_service, restart_service
};
#[cfg(feature = "event-log")]
pub use event_log::{
    log_system_event, log_windows_event, EVENTLOG_SUCCESS, EVENTLOG_ERROR_TYPE,
    EVENTLOG_WARNING_TYPE, EVENTLOG_INFORMATION_TYPE, EVENTLOG_AUDIT_SUCCESS,
    EVENTLOG_AUDIT_FAILURE, EVENT_ID_USER_ACTION,
};
#[cfg(feature = "event-log")]
pub use file_log::{
    log_message, get_appdata_log_path, set_event_log_enabled, is_event_log_enabled,
    set_event_source,
};
#[cfg(feature = "notification")]
pub use notification::{show_toast_notification, show_toast_notification_with_id};
#[cfg(feature = "clipboard")]
pub use clipboard::copy_text_to_clipboard;
pub use daemon::{
    get_sleep_prevention_count, ProcessPriority, set_process_priority, set_low_priority, set_idle_priority, PowerRequest, set_thread_execution_state, prevent_system_sleep, BackgroundPowerGuard, background_power_guard, DaemonConfig, DaemonPriority, DaemonService,
};