use tracing::info;

use crate::{config::Config, event::Event, event_queue::EventQueue, state::State};

#[derive(Debug)]
pub struct Peer {
    state: State,
    event_queue: EventQueue,
    config: Config,
}

impl Peer {
    pub fn new(config: Config) -> Self {
        let state = State::Idle;
        let event_queue = EventQueue::new();

        Self {
            state,
            event_queue,
            config,
        }
    }

    pub fn start(&mut self) {
        info!("peer started");
        self.event_queue.enqueue(Event::ManualStart);
    }

    pub async fn next(&mut self) {
        if let Some(event) = self.event_queue.dequeue() {
            info!("event occurred, event={:?}", event);
            self.handle_event(event).await;
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match &self.state {
            State::Idle => match event {
                Event::ManualStart => {
                    self.state = State::Connect;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::peer::Peer;
    use crate::state::State;

    #[tokio::test]
    async fn peer_can_transition_to_connect_state() {
        let config: Config = "64512 127.0.0.1 65413 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();
        peer.next().await;
        assert_eq!(peer.state, State::Connect);
    }
}
