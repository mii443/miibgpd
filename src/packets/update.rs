use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::bgp_type::AutonomousSystemNumber;
use crate::error::ConvertBytesToBgpMessageError;
use crate::path_attribute::{AsPath, Origin, PathAttribute};
use crate::routing::Ipv4Network;
use bytes::BytesMut;

use super::header::Header;

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
        todo!();
    }
}

impl From<UpdateMessage> for BytesMut {
    fn from(update: UpdateMessage) -> Self {
        todo!();
    }
}

impl TryFrom<BytesMut> for UpdateMessage {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        todo!();
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
            vec!["10.100.220.0.24".parse().unwrap()],
            vec![],
        );

        let update_message_bytes: BytesMut = update_message.clone().into();
        let update_message2: UpdateMessage = update_message_bytes.try_into().unwrap();
        assert_eq!(update_message, update_message2);
    }
}
