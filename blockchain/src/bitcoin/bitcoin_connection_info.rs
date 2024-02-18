use std::net::SocketAddr;

use crate::protocols::connection_info::ConnectionInfo;

#[derive(Clone, Debug)]
pub struct BitcoinConnectionInfo {
    // The public address that the peer listen on to incoming connections
    pub public_address: SocketAddr,
}

impl ConnectionInfo for BitcoinConnectionInfo {}
