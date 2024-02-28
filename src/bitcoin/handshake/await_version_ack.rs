use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tracing::{debug, error, warn};

use super::{
    connection_protocol::AdvanceStateResult, send_version_ack::SendVerAck,
    CHANNEL_NOT_INITIALIZED_ERROR,
};
use crate::bitcoin::{
    bitcoin_connection_info::BitcoinConnectionInfo,
    handshake::connection_protocol::BitcoinHandshakeError,
    messages::{commands::Command, HeaderCodec},
};

const MAX_READ_HEADER_ATTEMPTS: i8 = 3;
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
            for _ in 1..=MAX_READ_HEADER_ATTEMPTS {
                let result = framed.next().await;
                match result {
                    Some(Ok(header_message)) => {
                        // return the channel back
                        self.channel = Some(framed.into_inner());
                        debug!(
                            "Receive verack successfully from {:?}",
                            self.connection_info
                        );
                        if header_message.command != Command::VerAck {
                            return Err(BitcoinHandshakeError::ProtocolError(format!("Invalid header. header must be of verack command. the current header: {:?}",header_message)));
                        }
                        return Ok(());
                    }
                    Some(Err(e)) => {
                        // return the channel back
                        self.channel = Some(framed.into_inner());
                        error!(
                            "failed to receive verack message from {:?}, reason: {:?}",
                            self.connection_info, e
                        );
                        return Err(BitcoinHandshakeError::ProtocolError(e.to_string()));
                    }
                    None => {
                        warn!(
                            "did not receive verack from {:?}, continue reading",
                            self.connection_info
                        );
                        continue;
                    }
                }
            }
            return Err(BitcoinHandshakeError::ProtocolError(format!("Failed to read the verack header and stop after pass the max retries of: {} times", MAX_READ_HEADER_ATTEMPTS)));
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
