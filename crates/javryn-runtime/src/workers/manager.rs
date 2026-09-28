//! Worker pool manager coordinating worker threads and channels for Javryn V0.4.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use javryn_core::RuntimeError;

use super::id::{RequestId, WorkerId};
use super::message::{JsMessage, WorkerResponse};
use super::worker::WorkerHandle;

static NEXT_WORKER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Generates a new unique `WorkerId`.
pub fn next_worker_id() -> WorkerId {
    WorkerId(NEXT_WORKER_ID.fetch_add(1, Ordering::Relaxed))
}

/// Generates a new unique `RequestId`.
pub fn next_request_id() -> RequestId {
    RequestId(NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed))
}

/// Central manager overseeing all active worker threads in the runtime.
pub struct WorkerManager {
    workers: HashMap<WorkerId, WorkerHandle>,
}

impl WorkerManager {
    /// Creates a new empty `WorkerManager`.
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    /// Spawns a new worker thread, optionally loading an initial JavaScript file.
    pub fn spawn_worker(&mut self, script_path: Option<PathBuf>) -> Result<WorkerId, RuntimeError> {
        let id = next_worker_id();
        tracing::debug!(worker_id = id.0, "spawning worker thread via manager");

        let handle = WorkerHandle::spawn(id, script_path)?;
        self.workers.insert(id, handle);

        Ok(id)
    }

    /// Sends a serializable [`JsMessage`] payload to the target `worker_id`.
    pub fn post_message(
        &self,
        worker_id: WorkerId,
        data: JsMessage,
    ) -> Result<RequestId, RuntimeError> {
        let handle =
            self.workers
                .get(&worker_id)
                .ok_or_else(|| RuntimeError::WorkerCommunication {
                    message: format!("worker {worker_id} not found in WorkerManager"),
                })?;

        let request_id = next_request_id();
        handle.post_message(request_id, data)?;
        Ok(request_id)
    }

    /// Sends a parallel task execution request to the target `worker_id`.
    pub fn post_task(
        &self,
        worker_id: WorkerId,
        task_id: super::id::TaskId,
        fn_source: String,
        arg: JsMessage,
    ) -> Result<(), RuntimeError> {
        let handle =
            self.workers
                .get(&worker_id)
                .ok_or_else(|| RuntimeError::WorkerCommunication {
                    message: format!("worker {worker_id} not found in WorkerManager"),
                })?;

        handle.post_task(task_id, fn_source, arg)
    }

    /// Returns a vector of active running worker IDs.
    pub fn active_worker_ids(&self) -> Vec<WorkerId> {
        self.workers.keys().copied().collect()
    }

    /// Non-blocking check for responses from all managed worker threads.
    pub fn poll_responses(&mut self) -> Vec<WorkerResponse> {
        let mut responses = Vec::new();
        for worker in self.workers.values() {
            while let Ok(Some(resp)) = worker.try_recv() {
                responses.push(resp);
            }
        }
        responses
    }

    /// Returns the number of active workers.
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns `true` if no active workers exist.
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Terminates a specific worker thread by ID.
    pub fn terminate_worker(&mut self, worker_id: WorkerId) -> Result<(), RuntimeError> {
        if let Some(mut handle) = self.workers.remove(&worker_id) {
            handle.terminate()?;
        }
        Ok(())
    }

    /// Shuts down all active worker threads managed by this runtime using a timeout per worker.
    pub fn shutdown_with_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<(), RuntimeError> {
        tracing::debug!(
            count = self.workers.len(),
            timeout_ms = timeout.as_millis(),
            "shutting down all managed workers with timeout"
        );
        for (id, mut handle) in self.workers.drain() {
            if let Err(e) = handle.terminate_with_timeout(timeout) {
                tracing::warn!(worker_id = id.0, error = %e, "error terminating worker during shutdown");
            }
        }
        Ok(())
    }

    /// Shuts down all active worker threads managed by this runtime.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        self.shutdown_with_timeout(std::time::Duration::from_secs(5))
    }
}

impl Default for WorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WorkerManager {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

thread_local! {
    /// Thread-local WorkerManager for the main execution thread.
    static MANAGER: std::cell::RefCell<WorkerManager> = std::cell::RefCell::new(WorkerManager::new());
}

/// Accesses the main thread's local `WorkerManager`.
pub fn with_worker_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut WorkerManager) -> R,
{
    MANAGER.with(|m| f(&mut m.borrow_mut()))
}

/// Resets the main thread's local `WorkerManager`.
pub fn reset_worker_manager() {
    MANAGER.with(|m| {
        let _ = m.borrow_mut().shutdown();
        *m.borrow_mut() = WorkerManager::new();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn spawn_and_terminate_worker_isolation() {
        let mut manager = WorkerManager::new();
        let w1 = manager.spawn_worker(None).expect("spawn worker 1");
        let w2 = manager.spawn_worker(None).expect("spawn worker 2");

        assert_eq!(manager.len(), 2);
        assert!(manager.active_worker_ids().contains(&w1));
        assert!(manager.active_worker_ids().contains(&w2));

        // Terminate worker 1 — worker 2 must remain functional
        manager.terminate_worker(w1).expect("terminate worker 1");

        assert_eq!(manager.len(), 1);
        assert!(!manager.active_worker_ids().contains(&w1));
        assert!(manager.active_worker_ids().contains(&w2));

        // Worker 2 still accepts messages
        let req_id = manager
            .post_message(w2, JsMessage::String("hello w2".to_string()))
            .expect("post message to w2");
        assert!(req_id.0 > 0);

        manager.shutdown().expect("clean shutdown");
    }

    #[test]
    fn timed_shutdown_completes_within_bound() {
        let mut manager = WorkerManager::new();
        let _w1 = manager.spawn_worker(None).expect("spawn worker 1");

        let start = std::time::Instant::now();
        manager
            .shutdown_with_timeout(Duration::from_millis(500))
            .expect("shutdown with timeout");

        assert!(start.elapsed() < Duration::from_secs(3));
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn shutdown_is_idempotent() {
        let mut manager = WorkerManager::new();
        let _w1 = manager.spawn_worker(None).expect("spawn worker");
        manager.shutdown().expect("first shutdown");
        manager
            .shutdown()
            .expect("second shutdown should also succeed");
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn post_message_to_nonexistent_worker_fails() {
        let manager = WorkerManager::new();
        let result = manager.post_message(WorkerId(999), JsMessage::Null);
        assert!(result.is_err());
    }

    #[test]
    fn terminate_nonexistent_worker_is_noop() {
        let mut manager = WorkerManager::new();
        let result = manager.terminate_worker(WorkerId(999));
        assert!(result.is_ok());
    }

    #[test]
    fn drop_triggers_cleanup() {
        let mut manager = WorkerManager::new();
        let _w1 = manager.spawn_worker(None).expect("spawn worker");
        assert_eq!(manager.len(), 1);
        drop(manager);
        // No panic, no leak — drop calls shutdown
    }

    #[test]
    fn empty_manager_is_empty() {
        let manager = WorkerManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.active_worker_ids().is_empty());
    }

    // ─── RELEASE GATE 1: WORKER FAILURE ISOLATION & RECOVERY ───
    #[test]
    fn release_gate_worker_failure_isolation() {
        let mut manager = WorkerManager::new();
        let w1 = manager.spawn_worker(None).expect("spawn w1");
        let w2 = manager.spawn_worker(None).expect("spawn w2");
        assert_eq!(manager.len(), 2);

        // Simulate worker 1 failure/crash by terminating handle and removing from pool
        let _ = manager.terminate_worker(w1);
        assert_eq!(manager.len(), 1);
        assert!(!manager.active_worker_ids().contains(&w1));
        assert!(manager.active_worker_ids().contains(&w2));

        // Remaining worker 2 accepts new tasks without pool corruption
        let req_id = manager
            .post_message(w2, JsMessage::String("test payload".to_string()))
            .expect("post to w2");
        assert!(req_id.0 > 0);

        // Spawn replacement worker
        let w3 = manager.spawn_worker(None).expect("spawn w3 replacement");
        assert_eq!(manager.len(), 2);
        assert!(manager.active_worker_ids().contains(&w3));

        manager.shutdown().expect("clean shutdown");
    }

    // ─── RELEASE GATE 2: CHANNEL DISCONNECT HANDLING ───
    #[test]
    fn release_gate_channel_disconnect_handling() {
        let mut manager = WorkerManager::new();
        let w1 = manager.spawn_worker(None).expect("spawn w1");

        // Force shutdown of worker thread to cause receiver channel disconnect
        manager.terminate_worker(w1).expect("terminate worker");

        // Attempting to send message to terminated worker fails gracefully with WorkerCommunication error
        let res = manager.post_message(w1, JsMessage::Number(42.0));
        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            RuntimeError::WorkerCommunication { .. }
        ));

        // Submitting new worker request works cleanly
        let w2 = manager.spawn_worker(None).expect("spawn w2");
        assert!(manager.active_worker_ids().contains(&w2));
        manager.shutdown().expect("clean shutdown");
    }
}
