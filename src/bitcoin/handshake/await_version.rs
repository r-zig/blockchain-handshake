use bytes::BytesMut;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_util::codec::Decoder;
use tracing::{debug, error};

use super::{
    connection_protocol::AdvanceStateResult, send_version::SendVersion,
    CHANNEL_NOT_INITIALIZED_ERROR,
};

use crate::{
    bitcoin::{
        bitcoin_connection_info::BitcoinConnectionInfo,
        handshake::connection_protocol::BitcoinHandshakeError,
        messages::{
            commands::Command, sha2_checksum, HeaderCodec, HeaderMessage, VersionCodec,
            VersionMessage,
        },
    },
    HEADER_LENGTH,
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
        if let Some(mut channel) = self.channel.take() {
            let header = match read_header(&mut channel).await {
                Ok(header) => header,
                Err(e) => {
                    // return the channel back
                    self.channel = Some(channel);
                    return Err(e);
                }
            };

            // header verification
            if header.command != Command::Version {
                let error_str = format!(
                    "Received invalid header command {:?}, expected version command",
                    header.command
                );
                error!(error_str);
                return Err(BitcoinHandshakeError::InvalidResponse(error_str));
            }

            // read the version
            let version_result = read_version(&mut channel, header).await;
            // return the channel back on both cases Err and Ok
            self.channel = Some(channel);

            // If Ok - assign the version
            version_result.and_then(|version| {
                debug!("version accepted. details: {:?}", version);
                self.connection_info.version = Some(version);
                Ok(())
            })
        } else {
            panic!("{}", CHANNEL_NOT_INITIALIZED_ERROR);
        }
    }
}

async fn read_header(channel: &mut TcpStream) -> Result<HeaderMessage, BitcoinHandshakeError> {
    let mut codec = HeaderCodec {};
    let mut buffer = BytesMut::with_capacity(HEADER_LENGTH);
    buffer.resize(HEADER_LENGTH, 0);
    channel.read_exact(&mut buffer).await?;
    let header = codec
        .decode(&mut buffer)
        .map_err(|e| {
            BitcoinHandshakeError::ProtocolError(format!("Failed receive header. error: {:?}", e))
        })
        .and_then(|opt| {
            opt.ok_or(BitcoinHandshakeError::ProtocolError(
                "Cannot receive header".to_owned(),
            ))
        })?;

    return Ok(header);
}

async fn read_version(
    channel: &mut TcpStream,
    header: HeaderMessage,
) -> Result<VersionMessage, BitcoinHandshakeError> {
    let buffer_size: usize = header.payload_length.try_into().unwrap();
    let mut codec = VersionCodec {};
    let mut buffer = BytesMut::with_capacity(buffer_size);
    buffer.resize(buffer_size, 0);
    channel.read_exact(&mut buffer).await?;

    // verify the checksum
    let computed_checksum = sha2_checksum(&buffer);
    if computed_checksum != header.checksum {
        return Err(BitcoinHandshakeError::InvalidResponse(
                                            format!("Invalid checksum. The received header checksum: {:?} not equal to the computed actual payload checksum: {:?}",header.checksum,computed_checksum),
                                        ));
    }

    codec
        .decode(&mut buffer)
        .map_err(|e| {
            BitcoinHandshakeError::ProtocolError(format!("Failed receive version. error: {:?}", e))
        })
        .and_then(|opt| {
            opt.ok_or(BitcoinHandshakeError::ProtocolError(
                "Cannot receive version".to_owned(),
            ))
        })
}

impl From<SendVersion> for AwaitVersion {
    fn from(value: SendVersion) -> Self {
        AwaitVersion::new(
            value.channel.expect(CHANNEL_NOT_INITIALIZED_ERROR),
            value.connection_info,
        )
    }
}
