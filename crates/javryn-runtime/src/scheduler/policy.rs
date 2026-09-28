//! Scheduling policies defining task and worker selection algorithms for Javryn V0.7.

use super::metrics::WorkerStats;
use crate::tasks::task::ParallelTask;
use crate::workers::WorkerId;
use std::collections::{HashMap, VecDeque};

/// Trait implemented by scheduling policy algorithms.
pub trait SchedulingPolicy: Send + Sync {
    /// Selects index of next task to dispatch from `pending_tasks`.
    fn select_task(&self, pending_tasks: &VecDeque<ParallelTask>) -> Option<usize>;

    /// Selects target `WorkerId` from candidate workers.
    fn select_worker(
        &self,
        available_workers: &[WorkerId],
        worker_stats: &HashMap<WorkerId, WorkerStats>,
    ) -> Option<WorkerId>;
}

/// Standard Priority + FIFO scheduling policy.
#[derive(Debug, Default)]
pub struct PriorityFifoPolicy;

impl PriorityFifoPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl SchedulingPolicy for PriorityFifoPolicy {
    fn select_task(&self, pending_tasks: &VecDeque<ParallelTask>) -> Option<usize> {
        if pending_tasks.is_empty() {
            return None;
        }

        // Find highest priority task, breaking ties with FIFO (earliest index)
        let mut best_idx = 0;
        let mut highest_priority = pending_tasks[0].priority;

        for (idx, task) in pending_tasks.iter().enumerate().skip(1) {
            if task.priority > highest_priority {
                highest_priority = task.priority;
                best_idx = idx;
            }
        }

        Some(best_idx)
    }

    fn select_worker(
        &self,
        available_workers: &[WorkerId],
        _worker_stats: &HashMap<WorkerId, WorkerStats>,
    ) -> Option<WorkerId> {
        available_workers.first().copied()
    }
}

/// Dynamic Load-Aware scheduling policy balancing workload across workers.
#[derive(Debug, Default)]
pub struct LoadAwarePolicy;

impl LoadAwarePolicy {
    pub fn new() -> Self {
        Self
    }
}

impl SchedulingPolicy for LoadAwarePolicy {
    fn select_task(&self, pending_tasks: &VecDeque<ParallelTask>) -> Option<usize> {
        if pending_tasks.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut highest_priority = pending_tasks[0].priority;

        for (idx, task) in pending_tasks.iter().enumerate().skip(1) {
            if task.priority > highest_priority {
                highest_priority = task.priority;
                best_idx = idx;
            }
        }

        Some(best_idx)
    }

    fn select_worker(
        &self,
        available_workers: &[WorkerId],
        worker_stats: &HashMap<WorkerId, WorkerStats>,
    ) -> Option<WorkerId> {
        if available_workers.is_empty() {
            return None;
        }

        // Select worker with lowest running tasks, or lowest total busy duration
        let mut best_worker = available_workers[0];
        let mut min_tasks = usize::MAX;
        let mut min_busy = u64::MAX;

        for &worker_id in available_workers {
            let stats = worker_stats.get(&worker_id);
            let running = stats.map(|s| s.running_tasks).unwrap_or(0);
            let busy = stats.map(|s| s.busy_duration_ms).unwrap_or(0);

            if running < min_tasks || (running == min_tasks && busy < min_busy) {
                min_tasks = running;
                min_busy = busy;
                best_worker = worker_id;
            }
        }

        Some(best_worker)
    }
}
