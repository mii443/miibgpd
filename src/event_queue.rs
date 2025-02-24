use std::collections::VecDeque;

use crate::event::Event;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EventQueue(VecDeque<Event>);

impl EventQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn enqueue(&mut self, event: Event) {
        self.0.push_front(event);
    }

    pub fn dequeue(&mut self) -> Option<Event> {
        self.0.pop_back()
    }
}
