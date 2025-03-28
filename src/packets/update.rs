use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::bgp_type::AutonomousSystemNumber;
use crate::error::ConvertBytesToBgpMessageError;
use crate::path_attribute::{AsPath, Origin, PathAttribute};
use crate::routing::Ipv4Network;
use anyhow::Context;
use bytes::{BufMut, BytesMut};

use super::header::{Header, MessageType};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct UpdateMessage {
    header: Header,
    pub withdrawn_routes: Vec<Ipv4Network>,
    withdrawn_routes_length: u16, // octets
    pub path_attributes: Arc<Vec<PathAttribute>>,
    path_attributes_length: u16, // octets
    pub network_layer_reachability_information: Vec<Ipv4Network>,
}

impl UpdateMessage {
    pub fn new(
        path_attributes: Arc<Vec<PathAttribute>>,
        network_layer_reachability_information: Vec<Ipv4Network>,
        withdrawn_routes: Vec<Ipv4Network>,
    ) -> Self {
        let path_attributes_length =
            path_attributes.iter().map(|p| p.bytes_len()).sum::<usize>() as u16;

        let network_layer_reachability_information_length = network_layer_reachability_information
            .iter()
            .map(|r| r.bytes_len())
            .sum::<usize>() as u16;

        let withdrawn_routes_length = withdrawn_routes
            .iter()
            .map(|w| w.bytes_len())
            .sum::<usize>() as u16;

        let header_minimum_length = 19u16;
        let header = Header::new(
            header_minimum_length
                + path_attributes_length
                + network_layer_reachability_information_length
                + withdrawn_routes_length
                + 4,
            MessageType::Update,
        );

        Self {
            header,
            withdrawn_routes,
            withdrawn_routes_length,
            path_attributes,
            path_attributes_length,
            network_layer_reachability_information,
        }
    }
}

impl From<UpdateMessage> for BytesMut {
    fn from(message: UpdateMessage) -> Self {
        let mut bytes = BytesMut::new();

        bytes.put::<BytesMut>(message.header.into());

        bytes.put_u16(message.withdrawn_routes_length);
        message
            .withdrawn_routes
            .iter()
            .for_each(|r| bytes.put::<BytesMut>(r.into()));

        bytes.put_u16(message.path_attributes_length);
        message
            .path_attributes
            .iter()
            .for_each(|r| bytes.put::<BytesMut>(r.into()));

        message
            .network_layer_reachability_information
            .iter()
            .for_each(|r| bytes.put::<BytesMut>(r.into()));

        bytes
    }
}

impl TryFrom<BytesMut> for UpdateMessage {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        let header = Header::try_from(BytesMut::from(&bytes[0..19]))?;

        let withdrawn_routes_length: u16 = u16::from_be_bytes(bytes[19..21].try_into().context(
            format!("cannot convert to withdrawn_routes_length: {:?}", &bytes),
        )?);
        let withdrawn_routes_end_index = 21 + withdrawn_routes_length as usize;
        let withdrawn_routes_bytes = &bytes[21..withdrawn_routes_end_index];
        let withdrawn_routes = Ipv4Network::from_u8_slice(withdrawn_routes_bytes)?;

        let path_attributes_start_index = withdrawn_routes_end_index + 2;
        let total_path_attribute_length = u16::from_be_bytes(
            bytes[withdrawn_routes_end_index..path_attributes_start_index]
                .try_into()
                .context(format!(
                    "cannot convert to total_path_attribute_length: {:?}",
                    &bytes
                ))?,
        );

        let path_attributes_bytes = &bytes[path_attributes_start_index
            ..path_attributes_start_index + total_path_attribute_length as usize];
        let path_attributes = Arc::new(PathAttribute::from_u8_slice(path_attributes_bytes)?);
        let network_layer_reachability_information_start_index =
            path_attributes_start_index + total_path_attribute_length as usize;
        let network_layer_reachability_information = Ipv4Network::from_u8_slice(
            &bytes[network_layer_reachability_information_start_index..],
        )?;

        Ok(Self {
            header,
            withdrawn_routes,
            withdrawn_routes_length,
            path_attributes,
            path_attributes_length: total_path_attribute_length,
            network_layer_reachability_information,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_bytes_to_update_message_and_update_message_to_bytes() {
        let some_as: AutonomousSystemNumber = 64513.into();
        let some_ip: Ipv4Addr = "10.0.100.3".parse().unwrap();

        let local_as: AutonomousSystemNumber = 64514.into();
        let local_ip: Ipv4Addr = "10.200.100.3".parse().unwrap();

        let update_message_path_attributes = Arc::new(vec![
            PathAttribute::Origin(Origin::Igp),
            PathAttribute::AsPath(AsPath::AsSequence(vec![some_as, local_as])),
            PathAttribute::NextHop(local_ip),
        ]);

        let update_message = UpdateMessage::new(
            update_message_path_attributes,
            vec!["10.100.220.0/24".parse().unwrap()],
            vec![],
        );

        let update_message_bytes: BytesMut = update_message.clone().into();
        let update_message2: UpdateMessage = update_message_bytes.try_into().unwrap();
        assert_eq!(update_message, update_message2);
    }
}
