//! Async runtime and event loop module for Javryn.

pub mod event_loop;
pub mod timer;

pub use event_loop::EventLoop;
pub use timer::{TimerEntry, TimerId, TimerQueue};
