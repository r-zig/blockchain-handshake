use futures::{stream, Stream};
use std::pin::Pin;

use crate::protocols::peer_discovery::PeerDiscovery;

use super::{bitcoin_connection_info::BitcoinConnectionInfo, BitcoinConfiguration};

pub struct BitcoinPeerDiscovery {
    config: BitcoinConfiguration,
}

impl BitcoinPeerDiscovery {
    pub fn new(config: BitcoinConfiguration) -> Self {
        BitcoinPeerDiscovery { config }
    }
}

impl PeerDiscovery for BitcoinPeerDiscovery {
    type Info = BitcoinConnectionInfo;
    async fn discover_peers(&self) -> Pin<Box<dyn Stream<Item = Self::Info> + Send + Sync>> {
        let connection_info = BitcoinConnectionInfo {
            public_address: self.config.discover_remote_peer_address,
            version: None,
        };
        let stream = stream::once(async move { connection_info });
        // require since Once itself does not implement Unpin
        let pinned_stream = Box::pin(stream);
        pinned_stream
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use futures::StreamExt;
    use std::net::SocketAddr;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_discover_single_peer() {
        let expected_remote_peer_address = "127.0.0.1:8333";
        // Set up the environment variable
        std::env::set_var("DISCOVER_REMOTE_PEER_ADDRESS", expected_remote_peer_address);

        // Create a BitcoinPeerDiscovery instance
        let discovery = BitcoinPeerDiscovery::new(BitcoinConfiguration::parse());

        // Run the async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut peers_stream = discovery.discover_peers().await;
            if let Some(peer_connection_info) = peers_stream.next().await {
                assert_eq!(
                    peer_connection_info.public_address,
                    SocketAddr::from_str(expected_remote_peer_address).unwrap()
                );
            } else {
                panic!("No peers discovered");
            }
        });
    }
}
