use std::{str::FromStr, time::Duration};

use miibgpd::{config::Config, peer::Peer};
use tokio::time::sleep;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("miibgpd started");

    let configs = vec![Config::from_str("64512 127.0.0.1 64513 127.0.0.2 active").unwrap()];

    let mut peers: Vec<Peer> = configs.into_iter().map(Peer::new).collect();
    for peer in &mut peers {
        peer.start();
    }

    let mut handles = vec![];
    for mut peer in peers {
        let handle = tokio::spawn(async move {
            loop {
                peer.next().await;
                sleep(Duration::from_secs_f32(0.1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
