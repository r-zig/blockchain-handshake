use tokio::net::TcpStream;

use super::{
    connection_protocol::{AdvanceStateResult, BitcoinHandshakeError},
    disconnected::Disconnected,
};
use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

#[derive(Debug)]
pub(super) struct Connecting {
    pub(super) channel: Option<TcpStream>,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl Connecting {
    async fn connect(
        connection_info: BitcoinConnectionInfo,
    ) -> Result<TcpStream, BitcoinHandshakeError> {
        match TcpStream::connect(connection_info.public_address).await {
            Ok(stream) => Ok(stream),
            Err(e) => match e.kind() {
                tokio::io::ErrorKind::TimedOut => Err(BitcoinHandshakeError::Timeout),
                _ => Err(BitcoinHandshakeError::ConnectionFailed(e.to_string())),
            },
        }
    }

    pub(super) async fn execute(&mut self) -> AdvanceStateResult {
        self.channel = Some(Self::connect(self.connection_info.clone()).await?);
        Ok(())
    }
}
impl From<Disconnected> for Connecting {
    fn from(value: Disconnected) -> Self {
        Connecting {
            channel: None,
            connection_info: value.connection_info.clone(),
        }
    }
}
