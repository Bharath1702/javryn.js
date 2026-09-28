pub mod host;
pub mod manager;
pub mod task;

pub use crate::workers::id::{OperationId, TaskId};
pub use host::register_parallel_api;
pub use manager::TaskManager;
pub use task::{ParallelOperation, ParallelTask, TaskStatus};
