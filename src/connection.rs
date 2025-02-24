use anyhow::{Context, Ok, Result};
use bytes::BytesMut;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::info;

use crate::{
    config::{Config, Mode},
    error::CreateConnectionError,
    packets::message::Message,
};

const BGP_PORT: u16 = 179;

#[derive(Debug)]
pub struct Connection {
    connection: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub async fn connect(config: &Config) -> Result<Self> {
        let connection = match config.mode {
            Mode::Active => Self::connect_to_remote_peer(config).await,
            Mode::Passive => Self::wait_connection_from_remote_peer(config).await,
        }?;

        let buffer = BytesMut::with_capacity(1500);

        Ok(Self { connection, buffer })
    }

    pub async fn send(&mut self, message: Message) {
        let bytes: BytesMut = message.into();
        self.connection.write_all(&bytes[..]).await;
    }

    async fn connect_to_remote_peer(config: &Config) -> Result<TcpStream> {
        info!(
            "connecting to remote peer, remote-ip={:?}, bgp-port={}",
            config.remote_ip, BGP_PORT
        );
        TcpStream::connect((config.remote_ip, BGP_PORT))
            .await
            .context(format!(
                "cannot connect to remote peer, remote-ip={:?}, bgp-port={}",
                config.remote_ip, BGP_PORT
            ))
    }

    async fn wait_connection_from_remote_peer(config: &Config) -> Result<TcpStream> {
        info!(
            "waiting connection from remote peer, local-ip={:?}, bgp-port={}",
            config.local_ip, BGP_PORT
        );
        let listener = TcpListener::bind((config.local_ip, BGP_PORT)).await?;

        Ok(listener.accept().await?.0)
    }
}
