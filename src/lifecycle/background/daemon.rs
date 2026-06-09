//! Daemon / background power and priority helpers.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background) + Platform (Native).
//!
//! Provides utilities for background daemons and tasks to yield CPU resources
//! (via niceness/priority level adjustments) or prevent system sleep during
//! critical long-running operations.
//!
//! # Examples
//!
//! Lowering process priority class for silent background work:
//! ```
//! use library::lifecycle::background::daemon::{self, ProcessPriority};
//!
//! // Yield CPU time slice to user-facing applications
//! daemon::set_process_priority(ProcessPriority::BelowNormal);
//! ```
//!
//! Keeping the CPU awake while executing a long background job:
//! ```
//! use library::lifecycle::background::daemon::BackgroundPowerGuard;
//!
//! // Acquire the guard; sleep prevention is active
//! let guard = BackgroundPowerGuard::acquire();
//!
//! // perform work ...
//!
//! drop(guard); // Normal sleep behaviors are restored
//! ```

use std::sync::atomic::{AtomicI32, Ordering};

static SLEEP_PREVENTION_COUNT: AtomicI32 = AtomicI32::new(0);

/// Query the current global count of active sleep prevention locks in the process.
pub fn get_sleep_prevention_count() -> i32 {
    SLEEP_PREVENTION_COUNT.load(Ordering::Relaxed)
}

#[cfg(all(windows, feature = "windows-sys"))]
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, SetPriorityClass, SetThreadPriority,
    BELOW_NORMAL_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, THREAD_PRIORITY_BELOW_NORMAL,
};

#[allow(unused_variables, dead_code)]
fn set_priority_values(win_class: u32, unix_nice: i32) {
    #[cfg(all(windows, feature = "windows-sys"))]
    unsafe {
        let process = GetCurrentProcess();
        let _ = SetPriorityClass(process, win_class);
        
        let thread = GetCurrentThread();
        let _ = SetThreadPriority(thread, THREAD_PRIORITY_BELOW_NORMAL);
    }

    #[cfg(unix)]
    unsafe {
        extern "C" {
            fn setpriority(which: i32, who: u32, priority: i32) -> i32;
        }
        let _ = setpriority(0, 0, unix_nice);
    }
}

/// Execution priority levels for background processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessPriority {
    /// Below normal priority level, yielding CPU to user-facing applications.
    BelowNormal,
    /// Idle priority level, running only when the CPU would otherwise be idle.
    Idle,
}

/// Set the priority level of the current process.
///
/// On Windows, sets the process priority class. On Unix, calls `setpriority` to increase niceness.
///
/// # Examples
///
/// ```
/// use library::lifecycle::background::daemon::{self, ProcessPriority};
///
/// daemon::set_process_priority(ProcessPriority::BelowNormal);
/// ```
pub fn set_process_priority(priority: ProcessPriority) {
    #[cfg(all(windows, feature = "windows-sys"))]
    {
        let win_class = match priority {
            ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            ProcessPriority::Idle => IDLE_PRIORITY_CLASS,
        };
        let unix_nice = match priority {
            ProcessPriority::BelowNormal => 10,
            ProcessPriority::Idle => 19,
        };
        set_priority_values(win_class, unix_nice);
    }

    #[cfg(all(windows, not(feature = "windows-sys")))]
    {
        let _ = priority;
    }

    #[cfg(unix)]
    {
        let unix_nice = match priority {
            ProcessPriority::BelowNormal => 10,
            ProcessPriority::Idle => 19,
        };
        set_priority_values(0, unix_nice);
    }
}

/// Set below normal process priority class for background operation.
///
/// # Deprecated
/// Use `set_process_priority(ProcessPriority::BelowNormal)` instead.
#[deprecated(since = "1.9.25", note = "Use set_process_priority instead")]
#[allow(deprecated)]
pub fn set_low_priority() {
    set_process_priority(ProcessPriority::BelowNormal);
}

/// Set extreme idle/background priority class for low importance daemons.
///
/// # Deprecated
/// Use `set_process_priority(ProcessPriority::Idle)` instead.
#[deprecated(since = "1.9.25", note = "Use set_process_priority instead")]
#[allow(deprecated)]
pub fn set_idle_priority() {
    set_process_priority(ProcessPriority::Idle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerRequest {
    /// Allow the system and display to sleep normally.
    AllowSleep,
    /// Keep the system awake (prevent standby) but allow the display to turn off.
    KeepSystemAwake,
    /// Keep both the system and the display awake.
    KeepSystemAndDisplayAwake,
}

/// Sets the thread execution state to prevent or allow sleep.
/// Exposes fine-grained control over system vs display sleep behavior on Windows.
pub fn set_thread_execution_state(request: PowerRequest) {
    #[cfg(all(windows, feature = "windows-sys"))]
    {
        use windows_sys::Win32::System::Power::{
            SetThreadExecutionState, ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_DISPLAY_REQUIRED,
            ES_SYSTEM_REQUIRED,
        };
        let flags = match request {
            PowerRequest::AllowSleep => ES_CONTINUOUS,
            PowerRequest::KeepSystemAwake => ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED,
            PowerRequest::KeepSystemAndDisplayAwake => {
                ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED | ES_AWAYMODE_REQUIRED
            }
        };
        unsafe {
            SetThreadExecutionState(flags);
        }
    }
    #[cfg(any(not(windows), not(feature = "windows-sys")))]
    {
        let _ = request;
    }
}

/// Power management for background tasks (prevent system sleep but allow display sleep).
///
/// Useful for background daemons that must keep running during a long operation (e.g. sync, download)
/// without keeping the display turned on.
///
/// # Examples
///
/// ```
/// use library::lifecycle::background::daemon;
///
/// // Keep the system awake during a critical section
/// daemon::prevent_system_sleep(true);
///
/// // Allow sleep again
/// daemon::prevent_system_sleep(false);
/// ```
pub fn prevent_system_sleep(prevent: bool) {
    if prevent {
        SLEEP_PREVENTION_COUNT.fetch_add(1, Ordering::Relaxed);
        set_thread_execution_state(PowerRequest::KeepSystemAwake);
    } else {
        SLEEP_PREVENTION_COUNT.fetch_sub(1, Ordering::Relaxed);
        set_thread_execution_state(PowerRequest::AllowSleep);
    }
}

/// A structured guard that keeps the system awake during its lifecycle.
/// Releases the sleep prevention when dropped.
///
/// This uses RAII semantics to automatically call `prevent_system_sleep(false)` when
/// the guard instance is dropped.
///
/// # Examples
///
/// ```
/// use library::lifecycle::background::daemon::BackgroundPowerGuard;
///
/// {
///     let _guard = BackgroundPowerGuard::acquire();
///     // System is guaranteed to stay awake during this block
/// } // _guard goes out of scope here; normal sleep timeouts are restored automatically
/// ```
pub struct BackgroundPowerGuard {
    active: bool,
}

impl BackgroundPowerGuard {
    /// Attempts to acquire a sleep prevention guard.
    pub fn acquire() -> Self {
        prevent_system_sleep(true);
        Self { active: true }
    }
}

impl Drop for BackgroundPowerGuard {
    fn drop(&mut self) {
        if self.active {
            prevent_system_sleep(false);
        }
    }
}

/// Backward compatibility power guard helper function.
pub fn background_power_guard(active: bool) {
    prevent_system_sleep(active);
}

/// Configuration for bootstrapping a background daemon service.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// The unique name of the daemon (used for socket/pipe naming and single instance checks).
    pub name: String,
    /// The desired execution priority class.
    pub priority: DaemonPriority,
    /// Whether to enforce single instance execution.
    pub single_instance: bool,
    /// Whether to automatically prevent system sleep.
    pub prevent_sleep: bool,
}

impl DaemonConfig {
    /// Creates a default configuration with the specified daemon name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            priority: DaemonPriority::Low,
            single_instance: true,
            prevent_sleep: false,
        }
    }
}

/// Execution priority levels for background daemons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonPriority {
    /// Normal priority class (standard scheduling).
    Normal,
    /// Below normal priority, yielding CPU to user-facing applications (recommended).
    Low,
    /// Idle priority, running only when the CPU is otherwise idle.
    Idle,
}

/// A controller/coordinator for running a background daemon service.
/// Tightly integrates power management, priority control, single-instance protection,
/// and headless IPC server hosting.
pub struct DaemonService {
    config: DaemonConfig,
    _power_guard: Option<BackgroundPowerGuard>,
    #[cfg(feature = "window")]
    _instance_guard: Option<crate::lifecycle::foreground::guard::SingleInstanceGuard>,
}

impl DaemonService {
    /// Bootstraps a daemon service according to the configuration.
    /// Performs single instance checks, sets the process priority, and configures power management.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use library::lifecycle::background::daemon::{DaemonService, DaemonConfig, DaemonPriority};
    ///
    /// let config = DaemonConfig {
    ///     name: "my_daemon".to_string(),
    ///     priority: DaemonPriority::Low,
    ///     single_instance: true,
    ///     prevent_sleep: true,
    /// };
    ///
    /// let daemon = DaemonService::bootstrap(config).expect("Failed to bootstrap daemon");
    /// ```
    pub fn bootstrap(config: DaemonConfig) -> crate::error::Result<Self> {
        #[cfg(feature = "window")]
        let instance_guard = if config.single_instance {
            Some(crate::lifecycle::foreground::guard::SingleInstanceGuard::try_new()?)
        } else {
            None
        };

        // Set process priority
        match config.priority {
            DaemonPriority::Normal => {}
            DaemonPriority::Low => set_process_priority(ProcessPriority::BelowNormal),
            DaemonPriority::Idle => set_process_priority(ProcessPriority::Idle),
        }

        // Setup power guard if requested
        let power_guard = if config.prevent_sleep {
            Some(BackgroundPowerGuard::acquire())
        } else {
            None
        };

        Ok(Self {
            config,
            _power_guard: power_guard,
            #[cfg(feature = "window")]
            _instance_guard: instance_guard,
        })
    }

    /// Returns the name of the daemon service.
    pub fn name(&self) -> &str {
        &self.config.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_background_power_guard_raii_behavior() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let initial_count = get_sleep_prevention_count();
        {
            let _guard = BackgroundPowerGuard::acquire();
            assert_eq!(get_sleep_prevention_count(), initial_count + 1);
        } // guard dropped here
        assert_eq!(get_sleep_prevention_count(), initial_count);
    }

    #[test]
    fn test_daemon_service_bootstrap() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let config = DaemonConfig {
            name: format!("test_daemon_{}", std::process::id()),
            priority: DaemonPriority::Low,
            single_instance: false, // Turn off in tests to avoid locks
            prevent_sleep: true,
        };
        let initial_count = get_sleep_prevention_count();
        let daemon = DaemonService::bootstrap(config).unwrap();
        assert_eq!(get_sleep_prevention_count(), initial_count + 1);
        drop(daemon);
        assert_eq!(get_sleep_prevention_count(), initial_count);
    }

    #[test]
    fn test_daemon_helpers() {
        let _lock = TEST_MUTEX.lock().unwrap();
        set_process_priority(ProcessPriority::BelowNormal);
        set_process_priority(ProcessPriority::Idle);

        #[allow(deprecated)]
        {
            set_low_priority();
            set_idle_priority();
        }

        prevent_system_sleep(true);
        prevent_system_sleep(false);

        {
            let _guard = BackgroundPowerGuard::acquire();
        }
    }
}

