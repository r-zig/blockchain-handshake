use crate::bitcoin::bitcoin_connection_info::BitcoinConnectionInfo;

#[derive(Debug)]
pub(super) struct Disconnected {
    pub(super) connection_info: BitcoinConnectionInfo,
}
