mod await_version;
mod await_version_ack;
mod connecting;
mod connection_protocol;
mod disconnected;
mod established;
mod send_version;
mod send_version_ack;
pub use connection_protocol::BitcoinConnectionProtocol;
use structopt::StructOpt;

const CHANNEL_NOT_INITIALIZED_ERROR: &str = "channel TcpStream must be initialized";

#[derive(Debug, StructOpt)]
#[structopt(name = "Bitcoin own configuration")]
pub struct BitcoinOwnConfiguration {
    #[structopt(long, env = "USER_AGENT")]
    pub user_agent: String,
}
