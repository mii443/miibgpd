use std::net::Ipv4Addr;

use bytes::BytesMut;

use crate::{bgp_type::AutonomousSystemNumber, error::ConvertBytesToBgpMessageError};

use super::{
    header::{Header, MessageType},
    open::OpenMessage,
};

#[derive(Debug)]
pub enum Message {
    Open(OpenMessage),
}

impl TryFrom<BytesMut> for Message {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        let header_bytes_length = 19;

        if bytes.len() < header_bytes_length {
            return Err(ConvertBytesToBgpMessageError::from(anyhow::anyhow!(
                "Message length is too short: {}",
                bytes.len()
            )));
        }

        let header = Header::try_from(BytesMut::from(&bytes[0..header_bytes_length]))?;

        match header.type_ {
            MessageType::Open => {
                let open_message = OpenMessage::try_from(bytes)?;
                Ok(Self::Open(open_message))
            }
        }
    }
}

impl From<Message> for BytesMut {
    fn from(message: Message) -> Self {
        match message {
            Message::Open(open) => open.into(),
        }
    }
}

impl Message {
    pub fn new_open(my_as_number: AutonomousSystemNumber, my_ip_addr: Ipv4Addr) -> Self {
        let open_message = OpenMessage::new(my_as_number, my_ip_addr);
        Self::Open(open_message)
    }
}
