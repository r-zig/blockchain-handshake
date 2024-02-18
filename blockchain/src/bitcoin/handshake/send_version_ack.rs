use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

use super::{await_version::AwaitVersion, connection_protocol::AdvanceStateResult};

pub(super) struct SendVerAck {
    pub(super) channel: TcpStream,
    pub(super) connection_info: BitcoinConnectionInfo,
}

impl SendVerAck {
    pub(super) async fn execute(&mut self) -> AdvanceStateResult {
        self.channel.write_f32(1.0).await;
        Ok(())
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
