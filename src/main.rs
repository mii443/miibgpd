use std::{env, str::FromStr, time::Duration};

use miibgpd::{config::Config, peer::Peer};
use tokio::time::sleep;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("miibgpd started");

    let config = env::args().skip(1).fold("".to_owned(), |mut acc, s| {
        acc += &(s.to_owned() + " ");
        acc
    });
    let config = config.trim_end();
    let configs = vec![Config::from_str(&config).unwrap()];

    let mut peers: Vec<Peer> = configs.into_iter().map(Peer::new).collect();
    for peer in &mut peers {
        peer.start();
    }

    let mut handles = vec![];
    for mut peer in peers {
        let handle = tokio::spawn(async move {
            loop {
                peer.next().await;
                sleep(Duration::from_millis(10)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
