//! Task priority model for Javryn V0.7 Intelligent Scheduler.

/// Priority level assigned to a parallel task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TaskPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
}
