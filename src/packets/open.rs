use std::net::Ipv4Addr;

use anyhow::Context;
use bytes::{BufMut, BytesMut};

use crate::{
    bgp_type::{AutonomousSystemNumber, HoldTime, Version},
    error::ConvertBytesToBgpMessageError,
};

use super::header::{Header, MessageType};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct OpenMessage {
    header: Header,
    version: Version,
    my_as_number: AutonomousSystemNumber,
    hold_time: HoldTime,
    bgp_identifier: Ipv4Addr,
    optional_parameters_length: u8,
    optional_parameters: BytesMut,
}

impl OpenMessage {
    pub fn new(my_as_number: AutonomousSystemNumber, my_ip_addr: Ipv4Addr) -> Self {
        let header = Header::new(29, MessageType::Open);
        Self {
            header,
            version: Version::new(),
            my_as_number,
            hold_time: HoldTime::new(),
            bgp_identifier: my_ip_addr,
            optional_parameters_length: 0,
            optional_parameters: BytesMut::new(),
        }
    }
}

impl TryFrom<BytesMut> for OpenMessage {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        let header = Header::try_from(BytesMut::from(&bytes[0..19]))?;
        let version: Version = bytes[19].try_into()?;

        let my_as_number = AutonomousSystemNumber::from(u16::from_be_bytes(
            bytes[20..22]
                .try_into()
                .context(format!("cannot parse AS number: {:?}", &bytes[20..22]))?,
        ));

        let hold_time = HoldTime::from(u16::from_be_bytes(
            bytes[22..24]
                .try_into()
                .context(format!("cannot parse HoldTime: {:?}", &bytes[22..24]))?,
        ));

        let bgp_identifier: [u8; 4] = bytes[24..28]
            .try_into()
            .context(format!("cannot parse BGP Identifier: {:?}", &bytes[24..28]))?;
        let bgp_identifier = Ipv4Addr::from(bgp_identifier);
        let optional_parameters_length = bytes[28];
        let optional_parameters = BytesMut::from(&bytes[29..]);

        Ok(Self {
            header,
            version,
            my_as_number,
            hold_time,
            bgp_identifier,
            optional_parameters_length,
            optional_parameters,
        })
    }
}

impl From<OpenMessage> for BytesMut {
    fn from(message: OpenMessage) -> BytesMut {
        let mut bytes = BytesMut::new();
        let header_bytes: &BytesMut = &message.header.into();
        bytes.put(&header_bytes[..]);
        bytes.put_u8(message.version.into());
        bytes.put_u16(message.my_as_number.into());
        bytes.put_u16(message.hold_time.into());
        bytes.put(&message.bgp_identifier.octets()[..]);
        bytes.put_u8(message.optional_parameters_length);
        bytes.put(&message.optional_parameters[..]);

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_bytes_to_open_message_and_open_message_to_bytes() {
        let open_message = OpenMessage::new(64512.into(), "127.0.0.1".parse().unwrap());
        let open_message_bytes: BytesMut = open_message.clone().into();
        let open_message2: OpenMessage = open_message_bytes.try_into().unwrap();

        assert_eq!(open_message, open_message2);
    }
}
