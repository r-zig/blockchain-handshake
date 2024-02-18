use super::{bitcoin_peer::BitcoinPeer, bitcoin_peer_discovery::BitcoinPeerDiscovery};
use crate::protocols::peer::Peer;

pub struct BitcoinPeerFactory;

impl BitcoinPeerFactory {
    pub fn new_peer() -> impl Peer {
        let peer_discovery = BitcoinPeerDiscovery::new();
        BitcoinPeer::new(peer_discovery)
    }
}
