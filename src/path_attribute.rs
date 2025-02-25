use crate::bgp_type::AutonomousSystemNumber;

use std::collections::BTreeSet;
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PathAttribute {
    Origin(Origin),
    AsPath(AsPath),
    NextHop(Ipv4Addr),
    DontKnow(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Origin {
    Igp,
    Egp,
    Incomplete,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AsPath {
    AsSequence(Vec<AutonomousSystemNumber>),
    AsSet(BTreeSet<AutonomousSystemNumber>),
}

impl PathAttribute {
    pub fn bytes_len(&self) -> usize {
        let path_attribute_value_length = match self {
            PathAttribute::Origin(_) => 1,
            PathAttribute::AsPath(a) => a.bytes_len(),
            PathAttribute::NextHop(_) => 4,
            PathAttribute::DontKnow(v) => v.len(),
        };

        let length = path_attribute_value_length + 2;

        if path_attribute_value_length > 255 {
            length + 2
        } else {
            length + 1
        }
    }
}

impl AsPath {
    fn bytes_len(&self) -> usize {
        let as_bytes_length = match self {
            AsPath::AsSequence(as_sequence) => 2 * as_sequence.len(),
            AsPath::AsSet(as_set) => 2 * as_set.len(),
        };

        2 + as_bytes_length
    }
}
