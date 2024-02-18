use tokio::{io::AsyncReadExt, net::TcpStream};

use super::{connection_protocol::AdvanceStateResult, send_version::SendVersion};

use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

pub(super) struct AwaitVersion {
    pub(super) channel: TcpStream,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl AwaitVersion {
    // Initialize the state with the stream and the operation to await the version
    fn new(stream: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        AwaitVersion {
            channel: stream,
            connection_info,
        }
    }

    pub(crate) async fn execute(&mut self) -> AdvanceStateResult {
        self.channel.read_f32().await?;
        Ok(())
    }
}

impl From<SendVersion> for AwaitVersion {
    fn from(value: SendVersion) -> Self {
        AwaitVersion::new(value.channel, value.connection_info)
    }
}
