use std::net::SocketAddr;

use crate::protocols::connection_info::ConnectionInfo;

use super::messages::VersionMessage;

#[derive(Clone, Debug)]

pub struct BitcoinConnectionInfo {
    // The public address that the peer listen on to incoming connections
    pub public_address: SocketAddr,

    pub version: Option<VersionMessage>,
}

impl ConnectionInfo for BitcoinConnectionInfo {}
