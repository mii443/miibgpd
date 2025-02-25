use std::io;
use std::result::Result::Ok;

use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
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

    pub async fn get_message(&mut self) -> Option<Message> {
        self.read_data_from_tcp_connection().await;

        let buffer = self.split_buffer_at_message_separator()?;

        Message::try_from(buffer).ok()
    }

    async fn read_data_from_tcp_connection(&mut self) {
        loop {
            let mut buf: Vec<u8> = vec![];

            match self.connection.try_read_buf(&mut buf) {
                Ok(0) => {}
                Ok(n) => self.buffer.put(&buf[..]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => panic!(
                    "An error occured while reading data from TCP connection: {:?}",
                    e
                ),
            }
        }
    }

    fn split_buffer_at_message_separator(&mut self) -> Option<BytesMut> {
        let index = self.get_index_of_message_separator().ok()?;

        if self.buffer.len() < index {
            return None;
        }

        Some(self.buffer.split_to(index))
    }

    fn get_index_of_message_separator(&self) -> Result<usize> {
        let minimum_message_length = 19;

        if self.buffer.len() < 19 {
            return Err(anyhow::anyhow!(
                "Message length is too short: {}",
                self.buffer.len()
            ));
        }

        Ok(u16::from_be_bytes([self.buffer[16], self.buffer[17]]) as usize)
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
