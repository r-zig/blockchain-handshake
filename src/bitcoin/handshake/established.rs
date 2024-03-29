use tokio::net::TcpStream;

use super::{await_version_ack::AwaitVerAck, CHANNEL_NOT_INITIALIZED_ERROR};
use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

#[derive(Debug)]
pub(super) struct Established {
    pub(super) channel: TcpStream,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl Established {
    fn new(stream: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        Established {
            channel: stream,
            connection_info,
        }
    }
}

impl From<AwaitVerAck> for Established {
    fn from(value: AwaitVerAck) -> Self {
        Established::new(
            value.channel.expect(CHANNEL_NOT_INITIALIZED_ERROR),
            value.connection_info,
        )
    }
}
