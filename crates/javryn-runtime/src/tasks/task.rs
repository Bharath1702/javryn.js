//! Parallel task and operation representations for Javryn V0.5.

use boa_engine::JsObject;

pub use crate::workers::id::{OperationId, TaskId};
use crate::workers::{JsMessage, WorkerId};

/// Lifecycle status of a parallel task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Represents an individual parallel item task.
#[derive(Debug)]
pub struct ParallelTask {
    pub task_id: TaskId,
    pub op_id: OperationId,
    pub input_index: usize,
    pub fn_source: String,
    pub arg: JsMessage,
    pub status: TaskStatus,
    pub assigned_worker: Option<WorkerId>,
}

/// Represents a multi-task parallel operation (`parallel.map`).
pub struct ParallelOperation {
    pub op_id: OperationId,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub results: Vec<Option<JsMessage>>,
    pub resolve_fn: JsObject,
    pub reject_fn: JsObject,
    pub error: Option<String>,
    pub is_cancelled: bool,
}
