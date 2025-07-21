#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;
pub mod uart_protocol;

pub use types::*;
pub use uart_protocol::*;

pub const PACKET_SIZE: usize = 64;
pub const MAX_ENCODERS: usize = 8;

pub const PROTOCOL_VERSION: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_roundtrip() {
        let original = SensorDataPacket {
            seq: 42,
            encoders: [1, -2, 3, -4, 5, -6, 7, -8],
        };
        let packet = Packet::SensorData(original);

        let serialized = serialize_packet(&packet);
        let deserialized = deserialize_with_crc(&serialized).unwrap();

        match deserialized {
            Packet::SensorData(data) => {
                assert_eq!(data.seq, 42);
                assert_eq!(data.encoders, [1, -2, 3, -4, 5, -6, 7, -8]);
            }
            _ => panic!("Wrong packet type"),
        }
    }
}
