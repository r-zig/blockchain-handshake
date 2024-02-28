use super::{
    bitcoin_peer::BitcoinPeer, bitcoin_peer_discovery::BitcoinPeerDiscovery, BitcoinConfiguration,
};
use crate::protocols::peer::Peer;

pub struct BitcoinPeerFactory;

impl BitcoinPeerFactory {
    pub fn new_peer(config: BitcoinConfiguration) -> impl Peer {
        let peer_discovery = BitcoinPeerDiscovery::new(config);
        BitcoinPeer::new(peer_discovery)
    }
}
