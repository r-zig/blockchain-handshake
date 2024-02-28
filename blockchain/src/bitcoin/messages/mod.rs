mod header;
mod verack;
mod version;

pub(crate) use header::{HeaderCodec, HeaderMessage};
// pub(crate) use verack::VerackMessage;
pub(crate) use version::VersionCodec;
pub(crate) use version::VersionMessage;

pub mod commands {
    use strum::Display;
    use strum::EnumIter;
    use strum::EnumString;

    #[derive(Debug, PartialEq, EnumString, Display, EnumIter)]
    #[strum(serialize_all = "lowercase")]
    pub(crate) enum Command {
        Version,
        VerAck,
        Addr,
        Inv,
        GetData,
        NotFound,
        GetBlocks,
        GetHeaders,
        Tx,
        Block,
        Headers,
        GetAddr,
        MemPool,
        CheckOrder,
        SubmitOrder,
        Reply,
        Ping,
        Pong,
        Reject,
        FilterLoad,
        FilterAdd,
        FilterClear,
        MerkleBlock,
        Alert,
        SendHeaders,
        FeeFilter,
        SendCmpct,
        CmpctBlock,
        GetBlockTxn,
        BlockTxn,
    }

    impl Command {
        pub fn encode(&self) -> Result<[u8; 12], String> {
            let command = self.to_string();
            // Ensure the command is not longer than the fixed length.
            if command.len() > 12 {
                return Err("Command too long".to_owned());
            }
            let mut buf = [0u8; 12]; // Initialize a buffer with null bytes.
            for (i, &byte) in command.as_bytes().iter().enumerate() {
                if !byte.is_ascii() {
                    return Err("Command contains non-ASCII characters".to_owned());
                }
                buf[i] = byte; // Copy the command bytes into the buffer.
            }

            // The rest of the buffer is already filled with null bytes.
            Ok(buf)
        }

        /// Decodes a lowercase string into an enum variant, if valid.
        pub fn decode(buf: [u8; 12]) -> Result<Self, String> {
            // Convert the byte array into a UTF-8 string slice, stopping at the first null byte.
            // This effectively trims the null padding added during encoding.
            let command_str = match std::str::from_utf8(&buf) {
                Ok(s) => s.split('\0').next().unwrap_or(""),
                Err(_) => return Err("Failed to convert array to string".to_owned()),
            };

            // Parse the trimmed string into a Command enum variant.
            command_str.parse::<Command>().map_err(|e| e.to_string())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::Command;
        use strum::IntoEnumIterator; // Required for iterating over enum variants

        #[test]
        fn test_enum_to_string() {
            assert_eq!(Command::Version.to_string(), "version");
            assert_eq!(Command::VerAck.to_string(), "verack");
        }

        #[test]
        fn test_string_to_enum() {
            assert_eq!("version".parse::<Command>().unwrap(), Command::Version);
            assert_eq!("verack".parse::<Command>().unwrap(), Command::VerAck);
        }

        #[test]
        fn test_encode_decode_all_commands() {
            for command in Command::iter() {
                let encoded = command.encode().unwrap();
                let decoded = Command::decode(encoded)
                    .expect(&format!("Decoding failed for command: {}", command));
                assert_eq!(command, decoded, "Failed on command: {:?}", command);
            }
        }

        #[test]
        fn test_decode_invalid_command() {
            let invalid_array = [b'x'; 12]; // Invalid command representation
            assert!(Command::decode(invalid_array).is_err());
        }
    }
}
pub mod types {

    use std::{
        io,
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
    };

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) struct BitcoinIpAddr([u8; 16]);

    impl BitcoinIpAddr {
        pub fn to_bytes(&self) -> [u8; 16] {
            self.0
        }

        pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
            if bytes.len() == 16 {
                // Check if the address is an IPv4-mapped IPv6 address
                if bytes.starts_with(&[0u8; 10]) && bytes[10] == 0xff && bytes[11] == 0xff {
                    let ipv4addr = Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]);
                    Ok(ipv4addr.into())
                } else {
                    let ipv6addr = Ipv6Addr::from([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13],
                        bytes[14], bytes[15],
                    ]);
                    Ok(ipv6addr.into())
                }
            } else {
                Err("Invalid byte length for BitcoinIpAddr")
            }
        }
    }
    impl From<Ipv4Addr> for BitcoinIpAddr {
        fn from(addr: Ipv4Addr) -> Self {
            let mut bytes = [0u8; 16];
            // Fill in the prefix for IPv4-mapped IPv6 addresses
            bytes[10] = 0xFF;
            bytes[11] = 0xFF;
            // Place the IPv4 address in the last 4 bytes
            bytes[12..].copy_from_slice(&addr.octets());
            BitcoinIpAddr(bytes)
        }
    }

    impl From<Ipv6Addr> for BitcoinIpAddr {
        fn from(addr: Ipv6Addr) -> Self {
            // Directly use the IPv6 address bytes
            BitcoinIpAddr(addr.octets())
        }
    }

    impl From<BitcoinIpAddr> for IpAddr {
        fn from(addr: BitcoinIpAddr) -> Self {
            if addr.0[..10] == [0u8; 10] && addr.0[10] == 0xFF && addr.0[11] == 0xFF {
                // Convert back to Ipv4Addr if it matches the IPv4-mapped IPv6 address format
                IpAddr::V4(Ipv4Addr::new(
                    addr.0[12], addr.0[13], addr.0[14], addr.0[15],
                ))
            } else {
                // Otherwise, treat it as an Ipv6Addr
                IpAddr::V6(Ipv6Addr::from(addr.0))
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub(crate) struct CompactSize(u64);

    impl CompactSize {
        /// Creates a `CompactSize` instance based on the length of the input.
        ///
        /// This function is generic over any type `T` that implements `AsRef<[u8]>`,
        /// allowing it to accept a wide variety of types that can be represented as a byte slice.
        /// The primary use case for this function is when the length of the data (in bytes) is needed
        /// to construct a `CompactSize` object, and the actual content of the data is not relevant.
        ///
        /// # Examples
        ///
        /// ```
        /// let my_string = "Hello, world!";
        /// let compact_size_from_str = CompactSize::from_length(my_string);
        ///
        /// let my_bytes: Vec<u8> = vec![0, 1, 2, 3, 4, 5];
        /// let compact_size_from_bytes = CompactSize::from_length(&my_bytes);
        ///
        /// assert_eq!(compact_size_from_str, CompactSize(13)); // Length of "Hello, world!"
        /// assert_eq!(compact_size_from_bytes, CompactSize(6)); // Length of the byte vector
        /// ```
        ///
        /// This approach abstracts away the details of how the length is obtained,
        /// making the `CompactSize` struct flexible and easy to use with different types of data.
        ///
        /// # Parameters
        ///
        /// - `input`: An instance of any type `T` that can be referenced as a byte slice,
        /// including but not limited to `String`, `&str`, and `Vec<u8>`.
        ///
        /// # Returns
        ///
        /// Returns a `CompactSize` instance representing the length of the input data.
        pub(crate) fn from_length<T: AsRef<[u8]>>(input: T) -> Self {
            CompactSize(input.as_ref().len() as u64)
        }

        /// Returns the numeric value represented by this `CompactSize` as `usize`.
        pub fn value_as_usize(&self) -> usize {
            self.0 as usize
        }

        /// Decodes a `CompactSize` from a byte slice, returning the `CompactSize` and the number of bytes read.
        pub fn decode(buf: &[u8]) -> Result<(Self, usize), io::Error> {
            if buf.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Buffer is empty",
                ));
            }

            let first_byte = buf[0];
            match first_byte {
                0..=252 => {
                    // The value is the first byte itself.
                    Ok((CompactSize(first_byte as u64), 1))
                }
                253 => {
                    // The next 2 bytes are the value.
                    if buf.len() < 3 {
                        Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Not enough bytes for fd",
                        ))
                    } else {
                        let value = u16::from_le_bytes([buf[1], buf[2]]) as u64;
                        Ok((CompactSize(value), 3))
                    }
                }
                254 => {
                    // The next 4 bytes are the value.
                    if buf.len() < 5 {
                        Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Not enough bytes for fe",
                        ))
                    } else {
                        let value = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as u64;
                        Ok((CompactSize(value), 5))
                    }
                }
                255 => {
                    // The next 8 bytes are the value.
                    if buf.len() < 9 {
                        Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Not enough bytes for ff",
                        ))
                    } else {
                        let value = u64::from_le_bytes([
                            buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8],
                        ]);
                        Ok((CompactSize(value), 9))
                    }
                }
            }
        }
    }

    impl From<CompactSize> for Vec<u8> {
        fn from(value: CompactSize) -> Self {
            let value = value.0;
            match value {
                0..=252 => vec![value as u8],
                253..=0xffff => {
                    let mut vec = Vec::with_capacity(3);
                    vec.push(253);
                    vec.extend_from_slice(&(value as u16).to_le_bytes());
                    vec
                }
                0x10000..=0xffffffff => {
                    let mut vec = Vec::with_capacity(5);
                    vec.push(254);
                    vec.extend_from_slice(&(value as u32).to_le_bytes());
                    vec
                }
                _ => {
                    let mut vec = Vec::with_capacity(9);
                    vec.push(255);
                    vec.extend_from_slice(&value.to_le_bytes());
                    vec
                }
            }
        }
    }

    impl TryFrom<Vec<u8>> for CompactSize {
        type Error = &'static str;

        fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
            match bytes.get(0) {
                Some(&first_byte) if first_byte < 253 => Ok(CompactSize(first_byte as u64)),
                Some(&253) => {
                    let value_bytes = bytes.get(1..3).ok_or("Not enough bytes for u16")?;
                    Ok(CompactSize(u16::from_le_bytes(
                        value_bytes.try_into().map_err(|_| "Conversion error")?,
                    ) as u64))
                }
                Some(&254) => {
                    let value_bytes = bytes.get(1..5).ok_or("Not enough bytes for u32")?;
                    Ok(CompactSize(u32::from_le_bytes(
                        value_bytes.try_into().map_err(|_| "Conversion error")?,
                    ) as u64))
                }
                Some(&255) => {
                    let value_bytes = bytes.get(1..9).ok_or("Not enough bytes for u64")?;
                    Ok(CompactSize(u64::from_le_bytes(
                        value_bytes.try_into().map_err(|_| "Conversion error")?,
                    )))
                }
                _ => Err("Invalid format or empty vector"),
            }
        }
    }
}
