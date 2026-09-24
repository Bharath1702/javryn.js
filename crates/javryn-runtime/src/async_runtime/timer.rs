//! Timer queue and TimerId abstractions for the Javryn async runtime.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use boa_engine::JsValue;

/// An opaque unique identifier for scheduled host timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimerId(pub u64);

/// A scheduled timer entry in the event loop queue.
#[derive(Clone)]
pub struct TimerEntry {
    /// Unique identifier for this timer.
    pub id: TimerId,
    /// Absolute instant when this timer is due to fire.
    pub due_time: Instant,
    /// The requested delay duration.
    pub delay: Duration,
    /// Whether this timer is recurring (`setInterval`).
    pub is_interval: bool,
    /// The JavaScript callback function to invoke.
    pub callback: JsValue,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.due_time == other.due_time
    }
}

impl Eq for TimerEntry {}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a min-heap (earliest due_time on top).
        other
            .due_time
            .cmp(&self.due_time)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A priority queue of scheduled timers ordered by earliest due time.
#[derive(Default)]
pub struct TimerQueue {
    heap: BinaryHeap<TimerEntry>,
    next_id: u64,
}

impl TimerQueue {
    /// Creates a new empty [`TimerQueue`].
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            next_id: 1,
        }
    }

    /// Schedules a single-shot timer (`setTimeout`).
    pub fn set_timeout(&mut self, callback: JsValue, delay: Duration) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id += 1;

        let due_time = Instant::now() + delay;
        self.heap.push(TimerEntry {
            id,
            due_time,
            delay,
            is_interval: false,
            callback,
        });

        id
    }

    /// Schedules a recurring interval timer (`setInterval`).
    pub fn set_interval(&mut self, callback: JsValue, interval: Duration) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id += 1;

        let due_time = Instant::now() + interval;
        self.heap.push(TimerEntry {
            id,
            due_time,
            delay: interval,
            is_interval: true,
            callback,
        });

        id
    }

    /// Cancels and removes a timer by its [`TimerId`].
    pub fn cancel(&mut self, id: TimerId) -> bool {
        let original_len = self.heap.len();
        self.heap.retain(|entry| entry.id != id);
        self.heap.len() < original_len
    }

    /// Returns the next deadline instant if any timers are scheduled.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.heap.peek().map(|entry| entry.due_time)
    }

    /// Returns `true` if there are active timers in the queue.
    pub fn has_pending(&self) -> bool {
        !self.heap.is_empty()
    }

    /// Returns the count of pending timers.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Pops the next ready timer if its `due_time` has arrived.
    pub fn pop_ready(&mut self, now: Instant) -> Option<TimerEntry> {
        if self.heap.peek().is_some_and(|p| p.due_time <= now) {
            return self.heap.pop();
        }
        None
    }

    /// Reschedules an interval timer after firing.
    pub fn reschedule_interval(&mut self, mut entry: TimerEntry) {
        if entry.is_interval {
            entry.due_time = Instant::now() + entry.delay;
            self.heap.push(entry);
        }
    }

    /// Clears all pending timers.
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_id_ordering() {
        let mut queue = TimerQueue::new();
        let id1 = queue.set_timeout(JsValue::undefined(), Duration::from_millis(100));
        let id2 = queue.set_timeout(JsValue::undefined(), Duration::from_millis(10));

        assert_eq!(queue.len(), 2);
        assert!(queue.has_pending());

        let ready = queue
            .pop_ready(Instant::now() + Duration::from_millis(20))
            .unwrap();
        assert_eq!(ready.id, id2);

        assert!(queue.cancel(id1));
        assert!(queue.is_empty());
    }
}
