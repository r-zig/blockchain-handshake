use bytes::BytesMut;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::Encoder;

use crate::bitcoin::{
    bitcoin_connection_info::BitcoinConnectionInfo,
    handshake::CHANNEL_NOT_INITIALIZED_ERROR,
    messages::{commands::Command, HeaderCodec, HeaderMessage},
};

use super::{
    await_version::AwaitVersion,
    connection_protocol::{AdvanceStateResult, BitcoinHandshakeError},
};

const VERACK_CHECKSUM: [u8; 4] = [0x5D, 0xF6, 0xE0, 0xE2];

#[derive(Debug)]
pub(super) struct SendVerAck {
    pub(super) channel: Option<TcpStream>,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl SendVerAck {
    // The verack message is sent in reply to version. This message consists of only a message header with the command string "verack".
    pub(super) async fn execute(&mut self) -> AdvanceStateResult {
        // prepare the header
        let mut header_buffer = BytesMut::new();
        let mut header_codec = HeaderCodec {};
        let header_message =
            HeaderMessage::new_without_payload(Command::VersionAck, VERACK_CHECKSUM);
        header_codec
            .encode(header_message, &mut header_buffer)
            .map_err(|e| BitcoinHandshakeError::ProtocolError(e.to_string()))?;

        if let Some(mut channel) = self.channel.take() {
            let result = channel
                .write_all(&header_buffer)
                .await
                .map_err(|e| BitcoinHandshakeError::ProtocolError(e.to_string()));

            // return the channel back
            self.channel = Some(channel);
            result
        } else {
            panic!("{}", CHANNEL_NOT_INITIALIZED_ERROR);
        }
    }
}

impl From<AwaitVersion> for SendVerAck {
    fn from(value: AwaitVersion) -> Self {
        SendVerAck {
            channel: value.channel,
            connection_info: value.connection_info,
        }
    }
}
