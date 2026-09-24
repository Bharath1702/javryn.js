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

    /// Shuts down all active worker threads managed by this runtime.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        tracing::debug!(
            count = self.workers.len(),
            "shutting down all managed workers"
        );
        for (id, mut handle) in self.workers.drain() {
            if let Err(e) = handle.terminate() {
                tracing::warn!(worker_id = id.0, error = %e, "error terminating worker during shutdown");
            }
        }
        Ok(())
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
