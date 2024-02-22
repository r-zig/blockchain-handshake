use bytes::{Buf, BufMut, BytesMut};

use std::{
    io::{self, ErrorKind},
    net::Ipv6Addr,
};
use tokio_util::codec::{Decoder, Encoder};

use super::types::{BitcoinIpAddr, CompactSize};

/// Represents a Bitcoin version message.
#[derive(Debug, Clone)]
pub(crate) struct VersionMessage {
    version: i32,
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

    pub fn verify_message(&self) -> Result<(), String> {
        // Version check: Example - only accept versions >= 70001 and <= 70015
        if self.version < 70001 || self.version > 70015 {
            return Err("Unsupported version".to_string());
        }

        // Services check: Ensure NODE_NETWORK is supported
        if self.services & 0x01 == 0 {
            return Err("Node must support NODE_NETWORK".to_string());
        }

        // Timestamp check: Example - allow a 90-minute skew from current time
        let current_timestamp = chrono::Utc::now().timestamp();
        if self.timestamp < current_timestamp - 5400 || self.timestamp > current_timestamp + 5400 {
            return Err("Timestamp is out of range".to_string());
        }

        // Address checks: For simplicity, ensuring they're not unspecified or loopback
        // Skipping detailed IP range validation for brevity
        self.addr_recv_ip
            .is_valid()
            .map_err(|e| e + " for addr_recv_ip")?;

        self.addr_trans_ip
            .is_valid()
            .map_err(|e| e + " for addr_trans_ip")?;

        // Port checks
        if self.addr_recv_port == 0 || self.addr_trans_port == 0 {
            return Err("Ports must be non-zero".to_string());
        }

        // User Agent check: Example - limit length to 256 characters
        if self.user_agent.len() > 256 {
            return Err("User agent too long".to_string());
        }

        // Start Height check
        if self.start_height < 0 {
            return Err("Start height must be non-negative".to_string());
        }

        // No need to explicitly check nonce and relay as their types ensure validity

        Ok(())
    }
}

pub(crate) struct VersionCodec;

impl Encoder<VersionMessage> for VersionCodec {
    type Error = std::io::Error;

    fn encode(&mut self, msg: VersionMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_i32_le(msg.version);
        dst.put_u64_le(msg.services);
        dst.put_i64_le(msg.timestamp);

        dst.put_u64_le(msg.addr_recv_services);
        // Assuming addr_recv_ip is serialized to 16 bytes
        dst.extend_from_slice(&msg.addr_recv_ip.to_bytes());
        dst.put_u16_le(msg.addr_recv_port);

        dst.put_u64_le(msg.addr_trans_services);
        // Assuming addr_trans_ip is serialized to 16 bytes
        dst.extend_from_slice(&msg.addr_trans_ip.to_bytes());
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
        let version = buf.get_i32_le();
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
        buf.advance(user_agent_size.value_as_usize());
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
