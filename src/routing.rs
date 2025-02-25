use std::{
    net::IpAddr,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use futures::TryStreamExt;
use rtnetlink::{new_connection, IpVersion};
use tracing::info;

use crate::{config::Config, error::ConfigParseError};

#[derive(Debug, PartialEq, Eq, Clone)]
struct LocRib;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Ipv4Network(ipnetwork::Ipv4Network);

impl Deref for Ipv4Network {
    type Target = ipnetwork::Ipv4Network;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ipv4Network {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ipnetwork::Ipv4Network> for Ipv4Network {
    fn from(ip_network: ipnetwork::Ipv4Network) -> Self {
        Self(ip_network)
    }
}

impl FromStr for Ipv4Network {
    type Err = ConfigParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let network = s
            .parse::<ipnetwork::Ipv4Network>()
            .context(format!("cannot parse Ipv4Network: {:?}", s))?;

        Ok(Self(network))
    }
}

impl From<&Ipv4Network> for BytesMut {
    fn from(network: &Ipv4Network) -> Self {
        let prefix = network.prefix();

        let n = network.network().octets();
        let network_bytes = match prefix {
            0 => vec![],
            1..=8 => n[0..1].into(),
            9..=16 => n[0..2].into(),
            17..=24 => n[0..3].into(),
            25..=32 => n[0..4].into(),
            _ => panic!("Invalid prefix length: {:?}", prefix),
        };

        let mut bytes = BytesMut::new();

        bytes.put_u8(prefix);
        bytes.put(&network_bytes[..]);

        bytes
    }
}

impl Ipv4Network {
    pub fn bytes_len(&self) -> usize {
        match self.prefix() {
            0 => 1,
            1..=8 => 2,
            9..=16 => 3,
            17..=24 => 4,
            25..=32 => 5,
            _ => panic!("Invalid prefix length: {:?}", self.prefix()),
        }
    }
}

impl LocRib {
    pub async fn new(config: &Config) -> Result<Self> {
        todo!();
    }

    async fn lookup_kernel_routing_table(
        network_address: Ipv4Network,
    ) -> Result<(Vec<Ipv4Network>)> {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        let mut routes = handle.route().get(IpVersion::V4).execute();

        let mut results = vec![];
        while let Some(route) = routes.try_next().await? {
            let destination = if let Some((IpAddr::V4(addr), prefix)) = route.destination_prefix() {
                ipnetwork::Ipv4Network::new(addr, prefix)?.into()
            } else {
                continue;
            };

            if destination != network_address {
                continue;
            }

            results.push(destination);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::routing::LocRib;

    #[tokio::test]
    async fn locrib_can_lookup_kernel_routing_table() {
        let network = ipnetwork::Ipv4Network::new("192.168.0.0".parse().unwrap(), 16)
            .unwrap()
            .into();
        let routes = LocRib::lookup_kernel_routing_table(network).await.unwrap();
        let expected = vec![network];
        assert_eq!(routes, expected);
    }
}
