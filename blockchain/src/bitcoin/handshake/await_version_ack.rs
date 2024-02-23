use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tracing::{debug, error};

use super::{
    connection_protocol::AdvanceStateResult, send_version_ack::SendVerAck,
    CHANNEL_NOT_INITIALIZED_ERROR,
};
use crate::bitcoin::{
    bitcoin_connection_info::BitcoinConnectionInfo,
    handshake::connection_protocol::BitcoinHandshakeError,
    messages::{commands::Command, HeaderCodec},
};

#[derive(Debug)]
pub(super) struct AwaitVerAck {
    pub(super) channel: Option<TcpStream>,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl AwaitVerAck {
    // Initialize the state with the stream and the operation to await the version ack
    fn new(stream: TcpStream, connection_info: BitcoinConnectionInfo) -> Self {
        AwaitVerAck {
            channel: Some(stream),
            connection_info,
        }
    }

    pub(super) async fn execute(&mut self) -> AdvanceStateResult {
        if let Some(channel) = self.channel.take() {
            let mut framed = Framed::new(channel, HeaderCodec);
            let result = framed.next().await;

            // return the channel back
            self.channel = Some(framed.into_inner());
            match result {
                Some(r) => match r {
                    Ok(header_message) => {
                        debug!(
                            "Receive verack successfully from {:?}",
                            self.connection_info
                        );
                        if header_message.command != Command::VersionAck {
                            return Err(BitcoinHandshakeError::ProtocolError(format!("Invalid header. header must be of verack command. the current header: {:?}",header_message)));
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        error!(
                            "failed to receive verack message from {:?}, reason: {:?}",
                            self.connection_info, e
                        );
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

impl From<SendVerAck> for AwaitVerAck {
    fn from(value: SendVerAck) -> Self {
        AwaitVerAck::new(
            value.channel.expect(CHANNEL_NOT_INITIALIZED_ERROR),
            value.connection_info,
        )
    }
}
