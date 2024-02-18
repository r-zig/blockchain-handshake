use super::peer_discovery::PeerDiscovery;

// abstract trait for all peer types, regardless if they represent the local peer , or remote connected to - peer
pub trait Peer {
    fn get_state(&self) -> Option<&PeerState>;
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

// LocalPeer additional requirements over Peer
pub trait LocalPeer: Peer {
    type Discovery: PeerDiscovery;
}

// The general peer state without the specific protocol states
pub enum PeerState {
    Initialize,
    NetworkConnecting,
    NetworkConnected,
    Authenticated,
    Error(CommunicationError),
    Closed,
}

// Communication errors that can happened during connectivity phase
pub enum CommunicationError {
    NetworkError,
    ProtocolError,
}
