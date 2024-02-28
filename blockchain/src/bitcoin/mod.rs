use std::net::SocketAddr;

use clap::Parser;

pub mod bitcoin_connection_info;
pub mod bitcoin_factory;
pub mod bitcoin_peer;
mod bitcoin_peer_discovery;
mod handshake;
mod messages;

#[derive(Debug, Parser)]
#[clap(long_about = "Bitcoin own configuration")]
pub struct BitcoinConfiguration {
    #[clap(
        long = "remote-address",
        short = 'A',
        env = "DISCOVER_REMOTE_PEER_ADDRESS"
    )]
    pub discover_remote_peer_address: SocketAddr,

    #[clap(
        long,
        short = 'U',
        env = "USER_AGENT",
        default_value = "RZ Bitcoin client"
    )]
    pub user_agent: String,
}
