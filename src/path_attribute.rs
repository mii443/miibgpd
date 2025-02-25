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
