use bytes::{Buf, BufMut, BytesMut};

use std::{
    io::{self, ErrorKind},
    net::Ipv6Addr,
};
use tokio_util::codec::{Decoder, Encoder};

use super::types::{BitcoinIpAddr, CompactSize};

/// Represents a Bitcoin version message.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VersionMessage {
    version: u32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    addr_recv_ip: BitcoinIpAddr,
    addr_recv_port: u16,
    addr_trans_services: u64,
    addr_trans_ip: BitcoinIpAddr,
    addr_trans_port: u16,
    nonce: u64,
    user_agent_bytes: CompactSize,
    user_agent: String,
    start_height: i32,
    relay: bool,
}

impl VersionMessage {
    /// Creates a new `VersionMessage`.
    pub fn new(user_agent: &str, start_height: i32) -> Self {
        let user_agent = user_agent.into();
        VersionMessage {
            version: 70015, // The protocol version
            services: 1,    // NODE_NETWORK
            timestamp: chrono::Utc::now().timestamp(),
            addr_recv_services: 1, // NODE_NETWORK
            addr_recv_ip: Ipv6Addr::UNSPECIFIED.into(),
            addr_recv_port: 0,
            addr_trans_services: 1, // NODE_NETWORK
            addr_trans_ip: Ipv6Addr::UNSPECIFIED.into(),
            addr_trans_port: 0,
            nonce: rand::random(),
            user_agent_bytes: CompactSize::from_length(&user_agent),
            user_agent,
            start_height,
            relay: false,
        }
    }
}

pub(crate) struct VersionCodec;

impl Encoder<VersionMessage> for VersionCodec {
    type Error = std::io::Error;

    fn encode(&mut self, msg: VersionMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32_le(msg.version);
        dst.put_u64_le(msg.services);
        dst.put_i64_le(msg.timestamp);

        dst.put_u64_le(msg.addr_recv_services);
        // Assuming addr_recv_ip is serialized to 16 bytes
        dst.extend_from_slice(&msg.addr_recv_ip.encode());
        dst.put_u16_le(msg.addr_recv_port);

        dst.put_u64_le(msg.addr_trans_services);
        // Assuming addr_trans_ip is serialized to 16 bytes
        dst.extend_from_slice(&msg.addr_trans_ip.encode());
        dst.put_u16_le(msg.addr_trans_port);

        dst.put_u64_le(msg.nonce);

        // User agent: encode its length as CompactSize first, then the string itself
        let user_agent_bytes = msg.user_agent.into_bytes();
        let user_agent_compact_size = CompactSize::from_length(&user_agent_bytes);
        dst.extend_from_slice(&Vec::<u8>::from(user_agent_compact_size));
        dst.extend_from_slice(&user_agent_bytes);

        dst.put_i32_le(msg.start_height);
        dst.put_u8(msg.relay as u8);

        Ok(())
    }
}

impl Decoder for VersionCodec {
    type Item = VersionMessage; // Define your inbound message type
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 86 {
            // Minimum size without user_agent
            return Ok(None); // Wait for more bytes
        }

        let mut buf = src.as_ref();
        let version = buf.get_u32_le();
        let services = buf.get_u64_le();
        let timestamp = buf.get_i64_le();

        let addr_recv_services = buf.get_u64_le();
        let addr_recv_ip = BitcoinIpAddr::try_from_bytes(&buf[0..16]).map_err(|_| {
            io::Error::new(ErrorKind::InvalidData, "Invalid bytes for addr_recv_ip")
        })?;

        buf.advance(16); // Skip over the bytes we just processed
        let addr_recv_port = buf.get_u16_le();

        let addr_trans_services = buf.get_u64_le();
        let addr_trans_ip = BitcoinIpAddr::try_from_bytes(&buf[0..16]).map_err(|_| {
            io::Error::new(ErrorKind::InvalidData, "Invalid bytes for addr_trans_ip")
        })?;
        buf.advance(16); // Skip over the bytes we just processed
        let addr_trans_port = buf.get_u16_le();

        let nonce = buf.get_u64_le();

        // For user_agent, we need to decode its CompactSize length first
        let (user_agent_size, bytes_used) = CompactSize::decode(&buf).map_err(|_| {
            io::Error::new(ErrorKind::InvalidData, "Invalid bytes for user_agent_size")
        })?;
        buf.advance(bytes_used); // Advance the buffer past the CompactSize bytes

        if buf.remaining() < user_agent_size.value_as_usize() + 4 + 1 {
            // Ensure enough bytes for user_agent, start_height, and relay
            return Ok(None);
        }

        let user_agent_bytes = buf.copy_to_bytes(user_agent_size.value_as_usize());
        let user_agent = String::from_utf8(user_agent_bytes.to_vec())
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8 for user_agent"))?;

        let start_height = buf.get_i32_le();
        let relay = buf.get_u8() != 0;

        Ok(Some(VersionMessage {
            version,
            services,
            timestamp,
            addr_recv_services,
            addr_recv_ip,
            addr_recv_port,
            addr_trans_services,
            addr_trans_ip,
            addr_trans_port,
            nonce,
            user_agent_bytes: CompactSize::from_length(&user_agent),
            user_agent,
            start_height,
            relay,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn encode_decode_version() -> Result<(), Box<dyn std::error::Error>> {
        let expected_message = VersionMessage::new("user_agent", 70);
        let mut codec = VersionCodec {};
        let mut bytes = BytesMut::new();
        codec.encode(expected_message.clone(), &mut bytes).unwrap();

        let decoded_message = codec.decode(&mut bytes).unwrap().unwrap();
        assert_eq!(expected_message, decoded_message);
        Ok(())
    }
}
