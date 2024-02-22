use std::sync::Arc;

use futures::StreamExt;
use tokio::{net::TcpStream, sync::Mutex};

use crate::protocols::{
    peer::{LocalPeer, Peer, PeerState},
    peer_discovery::PeerDiscovery,
};

use super::{
    bitcoin_connection_info::BitcoinConnectionInfo, bitcoin_peer_discovery::BitcoinPeerDiscovery,
    handshake::BitcoinConnectionProtocol,
};

pub struct BitcoinPeer {
    peer_state: Option<PeerState>,
    peer_discovery: BitcoinPeerDiscovery,
    connected_peers: Arc<Mutex<Vec<RemotePeer>>>,
}

impl BitcoinPeer {
    pub fn new(peer_discovery: BitcoinPeerDiscovery) -> Self {
        BitcoinPeer {
            peer_state: None,
            peer_discovery,
            connected_peers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl LocalPeer for BitcoinPeer {
    type Discovery = BitcoinPeerDiscovery;
}

impl Peer for BitcoinPeer {
    fn get_state(&self) -> Option<&PeerState> {
        self.peer_state.as_ref()
    }

    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let connected_peers = self.connected_peers.clone();
        let peers_stream = self.peer_discovery.discover_peers().await;
        if peers_stream
            .map(RemotePeer::new)
            .then(move |mut peer| {
                let connected_peers = connected_peers.clone();
                async move {
                    // try to connect to the peer, if succeed - store this peer for later use
                    match peer.connect().await {
                        Ok(_) => {
                            let mut connected_peers = connected_peers.lock().await;
                            connected_peers.push(peer);
                            true
                        }
                        Err(_) => false,
                    }
                }
            })
            // in this version we connect only to one peer
            .any(|result| async move { result })
            .await
        {
            Ok(())
        } else {
            Err("Unable to connect to at least one peer".into())
        }
    }
}

struct RemotePeer {
    // The underlined tcp stream that communicate with the other
    channel: Option<TcpStream>,
    // The connection information used to connect to the remote peer
    connection_info: BitcoinConnectionInfo,
    peer_state: Option<PeerState>,
}

impl RemotePeer {
    fn new(connection_info: BitcoinConnectionInfo) -> Self {
        RemotePeer {
            channel: None,
            peer_state: None,
            connection_info,
        }
    }
}
impl Peer for RemotePeer {
    fn get_state(&self) -> Option<&PeerState> {
        self.peer_state.as_ref()
    }

    // try to connect to the remote peer with the information from the connection_info
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection_protocol = BitcoinConnectionProtocol::new(self.connection_info.clone());
        match connection_protocol.connect().await {
            Ok(value) => {
                self.channel = Some(value.0);
                self.peer_state = Some(PeerState::Authenticated);
                self.connection_info = value.1;
                Ok(())
            }
            Err(e) => {
                tracing::error! {%e};
                Err(Box::new(e))
            }
        }
    }
}
