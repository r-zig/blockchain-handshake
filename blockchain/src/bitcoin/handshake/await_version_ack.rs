use tokio::{io::AsyncReadExt, net::TcpStream};

use super::{connection_protocol::AdvanceStateResult, send_version_ack::SendVerAck};
use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

#[derive(Debug)]
pub(super) struct AwaitVerAck {
    pub(super) channel: TcpStream,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl AwaitVerAck {
    // Initialize the state with the stream and the operation to await the version ack
    fn new(stream: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        AwaitVerAck {
            channel: stream,
            connection_info,
        }
    }

    pub(super) async fn execute(&mut self) -> AdvanceStateResult {
        self.channel.read_f32().await;
        Ok(())
    }
}

impl From<SendVerAck> for AwaitVerAck {
    fn from(value: SendVerAck) -> Self {
        AwaitVerAck::new(value.channel, value.connection_info)
    }
}
