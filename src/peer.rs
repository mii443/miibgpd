use tracing::info;

use crate::{
    config::Config, connection::Connection, event::Event, event_queue::EventQueue,
    packets::message::Message, state::State,
};

#[derive(Debug)]
pub struct Peer {
    state: State,
    event_queue: EventQueue,
    tcp_connection: Option<Connection>,
    config: Config,
}

impl Peer {
    pub fn new(config: Config) -> Self {
        let state = State::Idle;
        let event_queue = EventQueue::new();

        Self {
            state,
            event_queue,
            tcp_connection: None,
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

        if let Some(connection) = &mut self.tcp_connection {
            if let Some(message) = connection.get_message().await {
                info!("message received, message={:?}", message);
                self.handle_message(message).await;
            }
        }
    }

    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::Open(open) => {
                self.event_queue.enqueue(Event::BgpOpen(open));
            }
            Message::Keepalive(keepalive) => {
                self.event_queue.enqueue(Event::KeepaliveMsg(keepalive));
            }
            Message::Update(update) => {
                self.event_queue.enqueue(Event::UpdateMsg(update));
            }
        }
    }

    async fn handle_event(&mut self, event: Event) {
        let current_state = self.state.clone();

        match &self.state {
            State::Idle => match event {
                Event::ManualStart => {
                    self.tcp_connection = Connection::connect(&self.config).await.ok();
                    if self.tcp_connection.is_some() {
                        self.event_queue.enqueue(Event::TcpConnectionConfirmed);
                    } else {
                        panic!("cannot establish TCP connection: {:?}", self.config);
                    }
                    self.state = State::Connect;
                }
                _ => {}
            },
            State::Connect => match event {
                Event::TcpConnectionConfirmed => {
                    self.tcp_connection
                        .as_mut()
                        .expect("tcp-connection is None")
                        .send(Message::new_open(
                            self.config.local_as,
                            self.config.local_ip,
                        ))
                        .await;
                    self.state = State::OpenSent;
                }
                _ => {}
            },
            State::OpenSent => match event {
                Event::BgpOpen(open) => {
                    self.tcp_connection
                        .as_mut()
                        .expect("tcp-connection is None")
                        .send(Message::new_keepalive())
                        .await;
                    self.state = State::OpenConfirm;
                }
                _ => {}
            },
            State::OpenConfirm => match event {
                Event::KeepaliveMsg(keepalive) => {
                    self.state = State::Established;
                }
                _ => {}
            },
            _ => {}
        }

        if self.state != current_state {
            info!(
                "state transitioned, from={:?}, to={:?}",
                current_state, self.state
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::config::Config;
    use crate::peer::Peer;
    use crate::state::State;

    #[tokio::test]
    async fn peer_can_transition_to_open_established_state() {
        let config: Config = "64512 127.0.0.1 64513 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();

        tokio::spawn(async move {
            let remote_config: Config = "64513 127.0.0.2 64512 127.0.0.1 passive".parse().unwrap();
            let mut remote_peer = Peer::new(remote_config);
            remote_peer.start();

            let max_step = 50;
            for _ in 0..max_step {
                remote_peer.next().await;
                if remote_peer.state == State::Established {
                    break;
                }
                tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
            }
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        let max_step = 50;
        for _ in 0..max_step {
            peer.next().await;
            if peer.state == State::Established {
                break;
            }
            tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
        }

        assert_eq!(peer.state, State::Established);
    }

    #[tokio::test]
    async fn peer_can_transition_to_open_confirm_state() {
        let config: Config = "64512 127.0.0.1 64513 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();

        tokio::spawn(async move {
            let remote_config: Config = "64513 127.0.0.2 64512 127.0.0.1 passive".parse().unwrap();
            let mut remote_peer = Peer::new(remote_config);
            remote_peer.start();

            let max_step = 50;
            for _ in 0..max_step {
                remote_peer.next().await;
                if remote_peer.state == State::OpenConfirm {
                    break;
                }
                tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
            }
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        let max_step = 50;
        for _ in 0..max_step {
            peer.next().await;
            if peer.state == State::OpenConfirm {
                break;
            }
            tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
        }

        assert_eq!(peer.state, State::OpenConfirm);
    }

    #[tokio::test]
    async fn peer_can_transition_to_connect_state() {
        let config: Config = "64512 127.0.0.1 65413 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();

        tokio::spawn(async move {
            let remote_config = "64513 127.0.0.2 64512 127.0.0.1 passive".parse().unwrap();
            let mut remote_peer = Peer::new(remote_config);
            remote_peer.start();
            remote_peer.next().await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        peer.next().await;
        assert_eq!(peer.state, State::Connect);
    }

    #[tokio::test]
    async fn peer_can_transition_to_open_sent_state() {
        let config: Config = "64512 127.0.0.1 65413 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();

        tokio::spawn(async move {
            let remote_config = "64513 127.0.0.2 65412 127.0.0.1 passive".parse().unwrap();
            let mut remote_peer = Peer::new(remote_config);
            remote_peer.start();
            remote_peer.next().await;
            remote_peer.next().await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        peer.next().await;
        peer.next().await;
        assert_eq!(peer.state, State::OpenSent);
    }
}
