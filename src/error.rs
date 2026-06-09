//! Custom first-class error types for library APIs.
//!
//! Provides structured error classification to prevent panics and raw String error passing.

use std::fmt;
use std::error::Error;

/// The primary error type for all operations in the library library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum libraryError {
    /// Errors occurring during local IPC named pipe or socket operations.
    Ipc(String),
    /// Errors occurring during command-line argument parsing.
    Cli(String),
    /// Errors occurring during background service controls (start/stop/restart).
    Service(String),
    /// Errors occurring during lifecycle single instance lock acquisitions.
    Guard(String),
    /// Errors occurring during OpenRGB communication or protocol parsing.
    Rgb(String),
    /// Errors occurring during platform display/system queries.
    Platform(String),
    /// Errors occurring during text formatting or date/uptime calculations.
    Formatting(String),
}

impl fmt::Display for libraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipc(msg) => write!(f, "IPC error: {}", msg),
            Self::Cli(msg) => write!(f, "CLI error: {}", msg),
            Self::Service(msg) => write!(f, "Service error: {}", msg),
            Self::Guard(msg) => write!(f, "Lifecycle guard error: {}", msg),
            Self::Rgb(msg) => write!(f, "RGB protocol error: {}", msg),
            Self::Platform(msg) => write!(f, "Platform query error: {}", msg),
            Self::Formatting(msg) => write!(f, "Formatting error: {}", msg),
        }
    }
}

impl Error for libraryError {}

impl libraryError {
    /// Check if the error represents a transient IPC connection abort or invalid param
    /// that indicates host shutdown or client disconnection.
    pub fn is_ipc_termination(&self) -> bool {
        match self {
            Self::Ipc(msg) => {
                let lower = msg.to_lowercase();
                lower.contains("aborted") 
                    || lower.contains("invalid") 
                    || lower.contains("broken pipe") 
                    || lower.contains("connection reset")
                    || lower.contains("pipe is being closed")
            }
            _ => false,
        }
    }
}

/// A specialized `Result` type alias utilizing libraryError.
pub type Result<T> = std::result::Result<T, libraryError>;

/// Convenient Result type alias utilizing libraryError.
pub type libraryResult<T> = Result<T>;

// Conversion helpers from std::io::Error
impl From<std::io::Error> for libraryError {
    fn from(err: std::io::Error) -> Self {
        Self::Ipc(err.to_string())
    }
}
