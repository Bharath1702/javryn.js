//! Scheduler performance and diagnostics metrics for Javryn V0.7.

use super::priority::TaskPriority;
use crate::workers::WorkerId;
use std::time::Instant;

/// Detailed worker workload and utilization statistics.
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub worker_id: WorkerId,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub busy_duration_ms: u64,
    pub last_dispatch: Option<Instant>,
}

/// Runtime scheduler performance metrics.
#[derive(Debug, Clone, Default)]
pub struct SchedulerMetrics {
    pub total_dispatches: u64,
    pub total_completed: u64,
    pub queue_wait_total_ms: u64,
    pub queue_wait_max_ms: u64,
    pub starvation_boosts: u64,
    pub high_priority_dispatches: u64,
    pub normal_priority_dispatches: u64,
    pub low_priority_dispatches: u64,
}

impl SchedulerMetrics {
    pub fn record_dispatch(&mut self, wait_ms: u64, priority: TaskPriority) {
        self.total_dispatches += 1;
        self.queue_wait_total_ms += wait_ms;
        if wait_ms > self.queue_wait_max_ms {
            self.queue_wait_max_ms = wait_ms;
        }

        match priority {
            TaskPriority::High => self.high_priority_dispatches += 1,
            TaskPriority::Normal => self.normal_priority_dispatches += 1,
            TaskPriority::Low => self.low_priority_dispatches += 1,
        }
    }

    pub fn record_completion(&mut self) {
        self.total_completed += 1;
    }

    pub fn average_queue_wait_ms(&self) -> f64 {
        if self.total_dispatches == 0 {
            0.0
        } else {
            self.queue_wait_total_ms as f64 / self.total_dispatches as f64
        }
    }
}
