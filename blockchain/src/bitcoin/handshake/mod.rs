mod await_version;
mod await_version_ack;
mod connecting;
mod connection_protocol;
mod disconnected;
mod established;
mod send_version;
mod send_version_ack;
pub use connection_protocol::BitcoinConnectionProtocol;

const CHANNEL_NOT_INITIALIZED_ERROR: &str = "channel TcpStream must be initialized";
