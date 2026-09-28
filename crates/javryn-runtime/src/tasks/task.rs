//! Task and operation state machine models for Javryn V0.6.

use boa_engine::JsObject;
use javryn_core::RuntimeError;

pub use crate::workers::id::{OperationId, TaskId};
use crate::workers::{JsMessage, WorkerId};

/// Task lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Created,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    /// Validates if transition from `self` to `target` state is valid.
    pub fn can_transition_to(&self, target: TaskStatus) -> bool {
        matches!(
            (self, target),
            (TaskStatus::Created, TaskStatus::Queued)
                | (TaskStatus::Queued, TaskStatus::Running)
                | (TaskStatus::Queued, TaskStatus::Cancelled)
                | (TaskStatus::Running, TaskStatus::Completed)
                | (TaskStatus::Running, TaskStatus::Failed)
                | (TaskStatus::Running, TaskStatus::Cancelled)
                | (TaskStatus::Running, TaskStatus::Queued)
        )
    }

    /// Attempts to transition task status to target, returning error on illegal transition.
    pub fn transition_to(
        &mut self,
        task_id: TaskId,
        target: TaskStatus,
    ) -> Result<(), RuntimeError> {
        if self.can_transition_to(target) {
            *self = target;
            Ok(())
        } else {
            Err(RuntimeError::InvalidTaskStateTransition {
                task_id: task_id.0,
                from: format!("{:?}", self),
                to: format!("{:?}", target),
            })
        }
    }
}

/// Operation lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    Created,
    Queued,
    Running,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
}

impl OperationStatus {
    /// Validates if transition from `self` to `target` state is valid.
    pub fn can_transition_to(&self, target: OperationStatus) -> bool {
        matches!(
            (self, target),
            (OperationStatus::Created, OperationStatus::Queued)
                | (OperationStatus::Queued, OperationStatus::Running)
                | (OperationStatus::Queued, OperationStatus::Cancelled)
                | (OperationStatus::Running, OperationStatus::Completed)
                | (OperationStatus::Running, OperationStatus::Failed)
                | (OperationStatus::Running, OperationStatus::Cancelling)
                | (OperationStatus::Cancelling, OperationStatus::Cancelled)
                | (OperationStatus::Cancelling, OperationStatus::Failed)
        )
    }

    /// Attempts to transition operation status to target, returning error on illegal transition.
    pub fn transition_to(
        &mut self,
        op_id: OperationId,
        target: OperationStatus,
    ) -> Result<(), RuntimeError> {
        if self.can_transition_to(target) {
            *self = target;
            Ok(())
        } else {
            Err(RuntimeError::InvalidOperationStateTransition {
                operation_id: op_id.0,
                from: format!("{:?}", self),
                to: format!("{:?}", target),
            })
        }
    }
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
    pub payload_bytes: usize,
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
    pub status: OperationStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_transitions() {
        let mut status = TaskStatus::Created;
        assert!(status.transition_to(TaskId(1), TaskStatus::Queued).is_ok());
        assert!(status.transition_to(TaskId(1), TaskStatus::Running).is_ok());
        assert!(
            status
                .transition_to(TaskId(1), TaskStatus::Completed)
                .is_ok()
        );

        // Invalid transition: Completed -> Running
        assert!(
            status
                .transition_to(TaskId(1), TaskStatus::Running)
                .is_err()
        );
    }

    #[test]
    fn test_operation_status_transitions() {
        let mut status = OperationStatus::Created;
        assert!(
            status
                .transition_to(OperationId(1), OperationStatus::Queued)
                .is_ok()
        );
        assert!(
            status
                .transition_to(OperationId(1), OperationStatus::Running)
                .is_ok()
        );
        assert!(
            status
                .transition_to(OperationId(1), OperationStatus::Completed)
                .is_ok()
        );

        // Invalid transition: Completed -> Cancelled
        assert!(
            status
                .transition_to(OperationId(1), OperationStatus::Cancelled)
                .is_err()
        );
    }
}
