//! Scheduler policy, metrics, and load-aware task dispatching abstractions for Javryn V0.7.

pub mod metrics;
pub mod policy;
pub mod priority;

pub use metrics::{SchedulerMetrics, WorkerStats};
pub use policy::{LoadAwarePolicy, PriorityFifoPolicy, SchedulingPolicy};
pub use priority::TaskPriority;

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use crate::tasks::task::ParallelTask;
use crate::workers::WorkerId;

/// Central Intelligent Scheduler coordinating task selection, priority queues, and load balancing.
pub struct Scheduler {
    policy: Box<dyn SchedulingPolicy>,
    metrics: SchedulerMetrics,
    worker_stats: HashMap<WorkerId, WorkerStats>,
    starvation_boost_counter: usize,
}

impl Scheduler {
    /// Creates a new `Scheduler` with default `LoadAwarePolicy`.
    pub fn new() -> Self {
        Self {
            policy: Box::new(LoadAwarePolicy::new()),
            metrics: SchedulerMetrics::default(),
            worker_stats: HashMap::new(),
            starvation_boost_counter: 0,
        }
    }

    /// Sets a custom scheduling policy.
    pub fn set_policy(&mut self, policy: Box<dyn SchedulingPolicy>) {
        self.policy = policy;
    }

    /// Returns a reference to the scheduler metrics.
    pub fn metrics(&self) -> &SchedulerMetrics {
        &self.metrics
    }

    /// Returns worker statistics map.
    pub fn worker_stats(&self) -> &HashMap<WorkerId, WorkerStats> {
        &self.worker_stats
    }

    /// Registers a newly spawned worker with the scheduler.
    pub fn register_worker(&mut self, worker_id: WorkerId) {
        self.worker_stats
            .entry(worker_id)
            .or_insert_with(|| WorkerStats {
                worker_id,
                running_tasks: 0,
                completed_tasks: 0,
                busy_duration_ms: 0,
                last_dispatch: None,
            });
    }

    /// Removes a terminated worker from scheduler tracking.
    pub fn unregister_worker(&mut self, worker_id: WorkerId) {
        self.worker_stats.remove(&worker_id);
    }

    /// Selects the next runnable task from pending tasks queue according to active policy.
    pub fn select_task(&mut self, pending_tasks: &mut VecDeque<ParallelTask>) -> Option<usize> {
        if pending_tasks.is_empty() {
            return None;
        }

        // Apply starvation prevention aging check
        self.apply_aging_boosts(pending_tasks);

        self.policy.select_task(pending_tasks)
    }

    /// Selects the best target worker for dispatch according to active policy and worker load statistics.
    pub fn select_worker(&self, available_workers: &[WorkerId]) -> Option<WorkerId> {
        if available_workers.is_empty() {
            return None;
        }

        self.policy
            .select_worker(available_workers, &self.worker_stats)
    }

    /// Records task dispatch events for latency and metric tracking.
    pub fn record_dispatch(&mut self, worker_id: WorkerId, task: &ParallelTask) {
        let now = Instant::now();
        let wait_ms = now.duration_since(task.enqueue_time).as_millis() as u64;

        self.metrics.record_dispatch(wait_ms, task.priority);

        if let Some(stats) = self.worker_stats.get_mut(&worker_id) {
            stats.running_tasks += 1;
            stats.last_dispatch = Some(now);
        }
    }

    /// Records task completion events for latency and worker utilization tracking.
    pub fn record_completion(&mut self, worker_id: WorkerId, task: &ParallelTask) {
        let now = Instant::now();
        let exec_ms = now.duration_since(task.enqueue_time).as_millis() as u64;

        self.metrics.record_completion();

        if let Some(stats) = self.worker_stats.get_mut(&worker_id) {
            if stats.running_tasks > 0 {
                stats.running_tasks -= 1;
            }
            stats.completed_tasks += 1;
            stats.busy_duration_ms += exec_ms;
        }
    }

    /// Applies aging boost to tasks queued for a prolonged duration to prevent starvation.
    fn apply_aging_boosts(&mut self, pending_tasks: &mut VecDeque<ParallelTask>) {
        self.starvation_boost_counter += 1;
        if self.starvation_boost_counter.is_multiple_of(50) {
            for task in pending_tasks.iter_mut() {
                if task.priority != TaskPriority::High
                    && task.enqueue_time.elapsed().as_millis() > 500
                {
                    task.priority = TaskPriority::High;
                    self.metrics.starvation_boosts += 1;
                }
            }
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::task::TaskStatus;
    use crate::workers::JsMessage;
    use crate::workers::id::{OperationId, TaskId};

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        assert!(TaskPriority::High > TaskPriority::Low);
    }

    #[test]
    fn test_priority_fifo_policy_selects_highest_priority() {
        let policy = PriorityFifoPolicy::new();
        let mut tasks = VecDeque::new();

        tasks.push_back(ParallelTask {
            task_id: TaskId(1),
            op_id: OperationId(1),
            input_index: 0,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Normal,
            enqueue_time: Instant::now(),
        });
        tasks.push_back(ParallelTask {
            task_id: TaskId(2),
            op_id: OperationId(1),
            input_index: 1,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::High,
            enqueue_time: Instant::now(),
        });
        tasks.push_back(ParallelTask {
            task_id: TaskId(3),
            op_id: OperationId(1),
            input_index: 2,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Low,
            enqueue_time: Instant::now(),
        });

        assert_eq!(policy.select_task(&tasks), Some(1));
    }

    #[test]
    fn test_load_aware_policy_selects_least_loaded_worker() {
        let policy = LoadAwarePolicy::new();
        let workers = vec![WorkerId(1), WorkerId(2), WorkerId(3)];
        let mut stats = HashMap::new();

        stats.insert(
            WorkerId(1),
            WorkerStats {
                worker_id: WorkerId(1),
                running_tasks: 2,
                completed_tasks: 5,
                busy_duration_ms: 100,
                last_dispatch: None,
            },
        );
        stats.insert(
            WorkerId(2),
            WorkerStats {
                worker_id: WorkerId(2),
                running_tasks: 0,
                completed_tasks: 5,
                busy_duration_ms: 50,
                last_dispatch: None,
            },
        );
        stats.insert(
            WorkerId(3),
            WorkerStats {
                worker_id: WorkerId(3),
                running_tasks: 1,
                completed_tasks: 5,
                busy_duration_ms: 80,
                last_dispatch: None,
            },
        );

        assert_eq!(policy.select_worker(&workers, &stats), Some(WorkerId(2)));
    }

    #[test]
    fn test_scheduler_starvation_boost() {
        let mut scheduler = Scheduler::new();
        let mut tasks = VecDeque::new();

        let old_time = Instant::now() - std::time::Duration::from_millis(600);
        tasks.push_back(ParallelTask {
            task_id: TaskId(1),
            op_id: OperationId(1),
            input_index: 0,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Low,
            enqueue_time: old_time,
        });

        for _ in 0..50 {
            let _ = scheduler.select_task(&mut tasks);
        }

        assert_eq!(tasks[0].priority, TaskPriority::High);
        assert_eq!(scheduler.metrics().starvation_boosts, 1);
    }

    #[test]
    fn test_priority_dispatch_sequence() {
        let mut scheduler = Scheduler::new();
        let mut tasks = VecDeque::new();

        tasks.push_back(ParallelTask {
            task_id: TaskId(1),
            op_id: OperationId(1),
            input_index: 0,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Low,
            enqueue_time: Instant::now(),
        });
        tasks.push_back(ParallelTask {
            task_id: TaskId(2),
            op_id: OperationId(1),
            input_index: 1,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::High,
            enqueue_time: Instant::now(),
        });
        tasks.push_back(ParallelTask {
            task_id: TaskId(3),
            op_id: OperationId(1),
            input_index: 2,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Normal,
            enqueue_time: Instant::now(),
        });

        // 1st selection should yield High priority (index 1)
        let idx1 = scheduler.select_task(&mut tasks).unwrap();
        assert_eq!(idx1, 1);
        let t1 = tasks.remove(idx1).unwrap();
        assert_eq!(t1.priority, TaskPriority::High);

        // 2nd selection should yield Normal priority (index 1 in remaining list: Low, Normal)
        let idx2 = scheduler.select_task(&mut tasks).unwrap();
        assert_eq!(idx2, 1);
        let t2 = tasks.remove(idx2).unwrap();
        assert_eq!(t2.priority, TaskPriority::Normal);

        // 3rd selection should yield Low priority (index 0)
        let idx3 = scheduler.select_task(&mut tasks).unwrap();
        assert_eq!(idx3, 0);
        let t3 = tasks.remove(idx3).unwrap();
        assert_eq!(t3.priority, TaskPriority::Low);
    }

    #[test]
    fn test_metrics_arithmetic_and_cleanup() {
        let mut scheduler = Scheduler::new();
        let w1 = WorkerId(1);
        scheduler.register_worker(w1);

        let t1 = ParallelTask {
            task_id: TaskId(1),
            op_id: OperationId(1),
            input_index: 0,
            fn_source: String::new(),
            arg: JsMessage::Null,
            status: TaskStatus::Queued,
            assigned_worker: None,
            payload_bytes: 0,
            priority: TaskPriority::Normal,
            enqueue_time: Instant::now() - std::time::Duration::from_millis(50),
        };

        scheduler.record_dispatch(w1, &t1);
        assert_eq!(scheduler.metrics().total_dispatches, 1);
        assert!(scheduler.metrics().queue_wait_total_ms >= 50);

        scheduler.record_completion(w1, &t1);
        assert_eq!(scheduler.metrics().total_completed, 1);
        assert!(scheduler.metrics().queue_wait_max_ms >= 50);
        assert!(scheduler.metrics().average_queue_wait_ms() >= 50.0);

        scheduler.unregister_worker(w1);
        assert!(scheduler.worker_stats().get(&w1).is_none());
    }
}
