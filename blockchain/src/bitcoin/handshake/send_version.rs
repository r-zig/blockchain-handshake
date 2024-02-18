use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::connecting::Connecting;

use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

pub(super) struct SendVersion {
    pub(super) channel: TcpStream,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl SendVersion {
    // Initialize the state with the stream and the operation to send the version
    fn new(channel: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        SendVersion {
            channel,
            connection_info,
        }
    }

    pub(super) async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.channel.write_all(b"some bytes").await?;
        Ok(())
    }
}

impl From<Connecting> for SendVersion {
    fn from(value: Connecting) -> Self {
        SendVersion::new(
            value
                .channel
                .expect("channel TcpStream must be initialized"),
            value.connection_info,
        )
    }
}
