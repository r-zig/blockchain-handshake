use std::io::{self, Error, ErrorKind};

use bytes::{Buf, BufMut, BytesMut};
use sha2::{Digest, Sha256};
use tokio_util::codec::{Decoder, Encoder};
use tracing::error;

use super::commands::Command;

const MAINNET_MAGIC: u32 = 0xD9B4BEF9;
const TESTNET_MAGIC: u32 = 0x0709110B;
const HEADER_LENGTH: usize = 24;

// message header for all messages type
#[derive(Debug)]
pub(crate) struct HeaderMessage {
    magic: u32,
    pub command: Command, // should convert into [u8; 12],
    payload_length: u32,
    checksum: [u8; 4],
}

impl HeaderMessage {
    pub fn new(command: Command, payload_buffer: &BytesMut) -> Self {
        let checksum = Self::compute_checksum(payload_buffer);
        let payload_length = payload_buffer.len() as u32;
        HeaderMessage {
            command,
            payload_length,
            magic: Self::get_magic(),
            checksum,
        }
    }

    pub fn new_without_payload(command: Command, checksum: [u8; 4]) -> Self {
        match command {
            Command::VerAck => {}
            _ => error!(
                "command {} must initialize with payload, use the other new() fn",
                command
            ),
        }
        HeaderMessage {
            command,
            payload_length: 0,
            magic: Self::get_magic(),
            checksum,
        }
    }

    fn compute_checksum(payload: &[u8]) -> [u8; 4] {
        // First SHA-256 hash
        let hash1 = Sha256::digest(payload);
        // Second SHA-256 hash
        let hash2 = Sha256::digest(&hash1);

        // Extract the first 4 bytes as the checksum
        let checksum = &hash2[0..4];
        let mut result = [0u8; 4];
        result.copy_from_slice(checksum);
        result
    }

    fn get_magic() -> u32 {
        MAINNET_MAGIC
        // todo!("read from configuration, decide if mainnet or testnet");
        // TESTNET_MAGIC
    }
}

pub(crate) struct HeaderCodec;

impl Decoder for HeaderCodec {
    type Item = HeaderMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_LENGTH {
            // Size of the header
            return Ok(None);
        }

        let magic = src.get_u32_le();
        let mut command = [0u8; 12];
        src.copy_to_slice(&mut command);
        let payload_length = src.get_u32_le();
        let mut checksum = [0u8; 4];
        src.copy_to_slice(&mut checksum);

        if magic != MAINNET_MAGIC && magic != TESTNET_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic number"));
        }
        Ok(Some(HeaderMessage {
            magic,
            command: Command::decode(command)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid command"))?,
            payload_length,
            checksum,
        }))
    }
}

impl Encoder<HeaderMessage> for HeaderCodec {
    type Error = io::Error;

    fn encode(&mut self, item: HeaderMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.magic != MAINNET_MAGIC && item.magic != TESTNET_MAGIC {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid magic number for encoding",
            ));
        }
        dst.put_u32_le(item.magic);
        let command_buf = &item
            .command
            .encode()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        dst.extend_from_slice(command_buf);
        dst.put_u32_le(item.payload_length);
        dst.extend_from_slice(&item.checksum);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_valid_header() {
        let mut codec = HeaderCodec;
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&MAINNET_MAGIC.to_le_bytes());
        buf.extend_from_slice(b"version\x00\x00\x00\x00\x00");
        buf.extend_from_slice(&100u32.to_le_bytes());
        buf.extend_from_slice(&[0xAB; 4]);

        match codec.decode(&mut buf) {
            Ok(Some(header)) => {
                assert_eq!(header.magic, MAINNET_MAGIC);
            }
            Ok(None) => panic!("Failed to decode a valid header, buffer to short"),
            Err(e) => panic!("Failed to decode a valid header {:?}", e),
        }
    }

    #[test]
    fn decode_invalid_magic() {
        let mut codec = HeaderCodec;
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&0x12345678u32.to_le_bytes());
        buf.extend_from_slice(b"version\x00\x00\x00\x00\x00");
        buf.extend_from_slice(&100u32.to_le_bytes());
        buf.extend_from_slice(&[0xAB; 4]);

        match codec.decode(&mut buf) {
            Err(e) => {
                assert_eq!(e.kind(), ErrorKind::InvalidData);
            }
            _ => panic!("Expected an error due to invalid magic number"),
        }
    }
}
