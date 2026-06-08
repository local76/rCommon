//! Generic background worker traits and sample implementations.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background).

use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

/// Events emitted by background worker threads to notify the TUI main loop.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Reports fractional progress (0.0 to 1.0) of a background task.
    Progress(f64),
    /// Signals successful completion with a return message.
    Success(String),
    /// Signals failure with an error message string.
    Error(String),
}

/// A trait representing a background worker task.
pub trait Worker: Send + 'static {
    /// Executes the worker task, sending progress and completion events back to the channel.
    fn run(self: Box<Self>, tx: Sender<WorkerEvent>);
}

/// A sample worker that simulates progress.
pub struct SampleWorker {
    pub steps: usize,
    pub step_delay_ms: u64,
}

impl Worker for SampleWorker {
    fn run(self: Box<Self>, tx: Sender<WorkerEvent>) {
        thread::spawn(move || {
            for i in 1..=self.steps {
                thread::sleep(Duration::from_millis(self.step_delay_ms));
                let progress = i as f64 / self.steps as f64;
                let _ = tx.send(WorkerEvent::Progress(progress));
            }
            thread::sleep(Duration::from_millis(100));
            let _ = tx.send(WorkerEvent::Success(
                "Background task completed successfully!".to_string(),
            ));
        });
    }
}

/// Helper to spawn a sample mock background worker task.
pub fn spawn_background_task(tx: Sender<WorkerEvent>) {
    let worker = SampleWorker {
        steps: 20,
        step_delay_ms: 150,
    };
    Box::new(worker).run(tx);
}
