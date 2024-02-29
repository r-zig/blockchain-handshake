use thiserror::Error;
use tokio::net::TcpStream;

use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

use super::{
    await_version::AwaitVersion, await_version_ack::AwaitVerAck, connecting::Connecting,
    disconnected::Disconnected, established::Established, send_version::SendVersion,
    send_version_ack::SendVerAck,
};

#[allow(private_interfaces)]
#[derive(Debug)]
pub enum BitcoinConnectionStates {
    Disconnected(Disconnected),
    Connecting(Connecting),
    SendVersion(SendVersion),
    AwaitVersion(AwaitVersion),
    SendVerAck(SendVerAck),
    AwaitVerAck(AwaitVerAck),
    // when the handshake established successfully, let the consumer take and use the inner opened tcp stream channel
    Established(Established),
    Failed(BitcoinHandshakeError),
}

#[derive(Error, Debug, Clone)]
pub enum BitcoinHandshakeError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid response received: {0}")]
    InvalidResponse(String),

    #[error("Connection timed out")]
    Timeout,

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

impl From<std::io::Error> for BitcoinHandshakeError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            tokio::io::ErrorKind::TimedOut => BitcoinHandshakeError::Timeout,
            tokio::io::ErrorKind::ConnectionRefused => {
                BitcoinHandshakeError::ConnectionFailed(value.to_string())
            }
            _ => BitcoinHandshakeError::ConnectionFailed(value.to_string()),
        }
    }
}

pub(super) type AdvanceStateResult = Result<(), BitcoinHandshakeError>;

// implement the connection protocol of bitcoin client (the handshake)
#[derive(Debug)]
pub struct BitcoinConnectionProtocol {
    state: BitcoinConnectionStates,
    connection_info: BitcoinConnectionInfo,
}

impl BitcoinConnectionProtocol {
    pub fn new(connection_info: BitcoinConnectionInfo) -> Self {
        BitcoinConnectionProtocol {
            connection_info: connection_info.clone(),
            state: BitcoinConnectionStates::Disconnected(Disconnected { connection_info }),
        }
    }

    #[tracing::instrument(level = "debug")]
    pub(crate) async fn advance(&mut self) -> Result<Option<TcpStream>, BitcoinHandshakeError> {
        let state = std::mem::replace(
            &mut self.state,
            BitcoinConnectionStates::Disconnected(Disconnected {
                connection_info: self.connection_info.clone(),
            }),
        );
        let result = match state {
            BitcoinConnectionStates::Disconnected(d) => self.handle_disconnect_state(d),
            BitcoinConnectionStates::Connecting(c) => self.handle_connecting_state(c).await,
            BitcoinConnectionStates::SendVersion(s) => self.handle_send_version(s).await,
            BitcoinConnectionStates::AwaitVersion(a) => self.handle_await_version(a).await,
            BitcoinConnectionStates::SendVerAck(s) => self.handle_send_version_ack(s).await,
            BitcoinConnectionStates::AwaitVerAck(a) => self.handle_await_version_ack(a).await,
            // Connection is established, nothing more to do
            BitcoinConnectionStates::Established(e) => return Ok(Some(e.channel)),
            BitcoinConnectionStates::Failed(e) => return Err(e),
        };
        if let Err(e) = result {
            self.state = BitcoinConnectionStates::Failed(e.clone());
            return Err(e);
        }
        Ok(None)
    }

    // Handle the "Disconnected" state by moving to the connecting state
    fn handle_disconnect_state(&mut self, disconnected: Disconnected) -> AdvanceStateResult {
        self.state = BitcoinConnectionStates::Connecting(disconnected.into());
        Ok(())
    }

    // Handle the "Connecting" state by waiting to establish a TCP connection. and advance to the next state
    async fn handle_connecting_state(&mut self, mut connecting: Connecting) -> AdvanceStateResult {
        match connecting.execute().await {
            Ok(_) => {
                self.state = BitcoinConnectionStates::SendVersion(connecting.into());
                Ok(())
            }
            Err(e) => Err(BitcoinHandshakeError::ConnectionFailed(format!(
                "Failed connecting to {}, reason: {}",
                self.connection_info.public_address.to_string(),
                e
            ))),
        }
    }

    async fn handle_send_version(&mut self, mut send_version: SendVersion) -> AdvanceStateResult {
        match send_version.execute().await {
            Ok(_) => {
                self.state = BitcoinConnectionStates::AwaitVersion(send_version.into());
                Ok(())
            }
            Err(e) => Err(BitcoinHandshakeError::ProtocolError(format!(
                "Failed sending version to {}, reason: {}",
                self.connection_info.public_address.to_string(),
                e
            ))),
        }
    }

    async fn handle_await_version(
        &mut self,
        mut await_version: AwaitVersion,
    ) -> AdvanceStateResult {
        match await_version.execute().await {
            Ok(_) => {
                self.state = BitcoinConnectionStates::SendVerAck(await_version.into());
                Ok(())
            }
            Err(e) => Err(BitcoinHandshakeError::InvalidResponse(format!(
                "Failed receiving version from {}, reason: {}",
                self.connection_info.public_address.to_string(),
                e
            ))),
        }
    }

    async fn handle_send_version_ack(
        &mut self,
        mut send_version_ack: SendVerAck,
    ) -> AdvanceStateResult {
        match send_version_ack.execute().await {
            Ok(_) => {
                self.state = BitcoinConnectionStates::AwaitVerAck(send_version_ack.into());
                Ok(())
            }
            Err(e) => Err(BitcoinHandshakeError::ProtocolError(format!(
                "Failed to send version ack to {}, reason: {}",
                self.connection_info.public_address.to_string(),
                e
            ))),
        }
    }

    async fn handle_await_version_ack(
        &mut self,
        mut await_version_ack: AwaitVerAck,
    ) -> AdvanceStateResult {
        match await_version_ack.execute().await {
            Ok(_) => {
                self.state = BitcoinConnectionStates::Established(await_version_ack.into());
                Ok(())
            }
            Err(e) => Err(BitcoinHandshakeError::InvalidResponse(format!(
                "Failed to receive verack from {}, reason: {}",
                self.connection_info.public_address.to_string(),
                e
            ))),
        }
    }

    pub async fn connect(
        &mut self,
    ) -> Result<(TcpStream, BitcoinConnectionInfo), BitcoinHandshakeError> {
        let mut connection_protocol = BitcoinConnectionProtocol::new(self.connection_info.clone());
        loop {
            _ = match connection_protocol.state {
                BitcoinConnectionStates::Established(established) => {
                    return Ok((established.channel, established.connection_info));
                }
                BitcoinConnectionStates::Failed(e) => return Err(e),
                _ => connection_protocol.advance().await,
            }
        }
    }
}
