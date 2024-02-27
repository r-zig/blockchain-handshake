use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tracing::{debug, error};

use super::{
    connection_protocol::AdvanceStateResult, send_version::SendVersion,
    CHANNEL_NOT_INITIALIZED_ERROR,
};

use crate::bitcoin::{
    bitcoin_connection_info::BitcoinConnectionInfo,
    handshake::connection_protocol::BitcoinHandshakeError, messages::VersionCodec,
};

#[derive(Debug)]
pub(super) struct AwaitVersion {
    pub(super) channel: Option<TcpStream>,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl AwaitVersion {
    // Initialize the state with the stream and the operation to await the version
    fn new(channel: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        AwaitVersion {
            channel: Some(channel),
            connection_info,
        }
    }

    pub(crate) async fn execute(&mut self) -> AdvanceStateResult {
        if let Some(channel) = self.channel.take() {
            let mut framed = Framed::new(channel, VersionCodec);
            let result = framed.next().await;

            // return the channel back
            self.channel = Some(framed.into_inner());
            match result {
                Some(r) => match r {
                    Ok(version_message) => {
                        debug!(
                            "Receive version successfully from {:?}, version message: {:?}",
                            self.connection_info, version_message
                        );
                        // version_message
                        //     .verify_message()
                        //     .map_err(|e| BitcoinHandshakeError::InvalidResponse(e))?;
                        // assign the version message to the connection info, for later use when communicating with this peer
                        self.connection_info.version = Some(version_message);
                        return Ok(());
                    }
                    Err(e) => {
                        error!("failed to receive version from {:?}", self.connection_info);
                        return Err(BitcoinHandshakeError::ProtocolError(e.to_string()));
                    }
                },
                None => {
                    error!("failed to receive version from {:?}", self.connection_info);
                    return Err(BitcoinHandshakeError::ProtocolError(
                        "Cannot receive version".to_owned(),
                    ));
                }
            }
        } else {
            panic!("{}", CHANNEL_NOT_INITIALIZED_ERROR);
        }
    }
}

impl From<SendVersion> for AwaitVersion {
    fn from(value: SendVersion) -> Self {
        AwaitVersion::new(
            value.channel.expect(CHANNEL_NOT_INITIALIZED_ERROR),
            value.connection_info,
        )
    }
}
