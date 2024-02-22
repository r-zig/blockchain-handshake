use super::{
    connecting::Connecting, connection_protocol::BitcoinHandshakeError,
    CHANNEL_NOT_INITIALIZED_ERROR,
};
use bytes::BytesMut;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::Encoder;

use crate::bitcoin::{
    bitcoin_connection_info::BitcoinConnectionInfo,
    messages::{commands::Command, HeaderCodec, HeaderMessage, VersionCodec, VersionMessage},
};

#[derive(Debug)]
pub(super) struct SendVersion {
    pub(super) channel: Option<TcpStream>,
    pub(super) connection_info: BitcoinConnectionInfo,
    user_agent: String,
}

impl SendVersion {
    // Initialize the state with the stream and the operation to send the version
    fn new(channel: TcpStream, connection_info: BitcoinConnectionInfo, user_agent: String) -> Self {
        SendVersion {
            channel: Some(channel),
            connection_info,
            user_agent,
        }
    }

    pub(super) async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // store the payload in temporary buffer, so we can take the payload length
        let mut payload_buffer = BytesMut::new();
        let mut payload_codec = VersionCodec {};
        let payload_message = VersionMessage::new(&self.user_agent, 0);
        payload_codec
            .encode(payload_message, &mut payload_buffer)
            .map_err(|e| BitcoinHandshakeError::ProtocolError(e.to_string()))?;

        // prepare the header
        let mut header_buffer = BytesMut::new();
        let mut header_codec = HeaderCodec {};
        let header_message = HeaderMessage::new(Command::Version, &payload_buffer);
        header_codec
            .encode(header_message, &mut header_buffer)
            .map_err(|e| BitcoinHandshakeError::ProtocolError(e.to_string()))?;

        if let Some(mut channel) = self.channel.take() {
            let result = async {
                channel.write_all(&header_buffer).await?;
                channel.write_all(&payload_buffer).await?;
                Ok(())
            }
            .await;

            // return the channel back
            self.channel = Some(channel);
            result
        } else {
            panic!("{}", CHANNEL_NOT_INITIALIZED_ERROR);
        }
    }
}

impl From<Connecting> for SendVersion {
    fn from(value: Connecting) -> Self {
        SendVersion::new(
            value.channel.expect(CHANNEL_NOT_INITIALIZED_ERROR),
            value.connection_info,
            "My agent - todo take from configuration".to_owned(),
        )
    }
}
