use crate::types::{Packet, BUFFER_SIZE};
use crc::Crc;

pub const COMMS_CRC: Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);
pub const SYNC_HEADER: [u8; 4] = [0xAA, 0x55, 0xAA, 0x55];
const SYNC_HEADER_LEN: usize = 4;
const LENGTH_FIELD_LEN: usize = 1;
const CRC_LEN: usize = 2;
const PAYLOAD_START: usize = SYNC_HEADER_LEN + LENGTH_FIELD_LEN;
const MAX_COBS_LEN: usize = BUFFER_SIZE - PAYLOAD_START - CRC_LEN;
const CRC_START: usize = BUFFER_SIZE - CRC_LEN;

pub fn serialize_packet(packet: &Packet) -> [u8; BUFFER_SIZE] {
    let mut buf = [0u8; BUFFER_SIZE];

    buf[..SYNC_HEADER_LEN].copy_from_slice(&SYNC_HEADER);

    let payload_range = PAYLOAD_START..PAYLOAD_START + MAX_COBS_LEN;
    let used = postcard::to_slice_cobs(packet, &mut buf[payload_range])
        .expect("Serialization error")
        .len();

    buf[SYNC_HEADER_LEN] = used as u8;

    let crc_range = ..PAYLOAD_START + used;
    let crc = COMMS_CRC.checksum(&buf[crc_range]);

    buf[CRC_START..].copy_from_slice(&crc.to_be_bytes());

    buf
}

pub fn deserialize_with_crc(buf: &[u8]) -> Option<Packet> {
    if buf.len() != BUFFER_SIZE {
        return None;
    }

    if buf[..SYNC_HEADER_LEN] != SYNC_HEADER {
        return None;
    }

    let cobs_len = buf[SYNC_HEADER_LEN] as usize;

    if cobs_len == 0 || cobs_len > MAX_COBS_LEN {
        return None;
    }

    let crc_received = u16::from_be_bytes([buf[CRC_START], buf[CRC_START + 1]]);

    let crc_calculated = COMMS_CRC.checksum(&buf[..PAYLOAD_START + cobs_len]);

    if crc_received != crc_calculated {
        return None;
    }

    // Decode the COBS data
    let mut packet_buf = [0u8; BUFFER_SIZE];
    let payload_end = PAYLOAD_START + cobs_len;
    let decode_report = cobs::decode(&buf[PAYLOAD_START..payload_end], &mut packet_buf).ok()?;
    let len = decode_report.frame_size();

    postcard::from_bytes(&packet_buf[..len]).ok()
}

pub fn create_sensor_packet(seq: u32, encoders: [i32; 8]) -> Packet {
    use crate::types::SensorDataPacket;
    Packet::SensorData(SensorDataPacket::new(seq, encoders))
}

pub fn create_reset_packet(encoder_id: u8) -> Packet {
    use crate::types::ResetCommand;
    Packet::Reset(ResetCommand { encoder_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_serialize_deserialize_sensor_data() {
        let original = SensorDataPacket::new(123, [1, -2, 3, -4, 5, -6, 7, -8]);
        let packet = Packet::SensorData(original.clone());

        let serialized = serialize_packet(&packet);
        let deserialized = deserialize_with_crc(&serialized).unwrap();

        match deserialized {
            Packet::SensorData(data) => {
                assert_eq!(data, original);
            }
            _ => panic!("Wrong packet type"),
        }
    }

    #[test]
    fn test_serialize_deserialize_reset_command() {
        let packet = Packet::Reset(ResetCommand::single(3));

        let serialized = serialize_packet(&packet);
        let deserialized = deserialize_with_crc(&serialized).unwrap();

        match deserialized {
            Packet::Reset(cmd) => {
                assert_eq!(cmd.encoder_id, 3);
            }
            _ => panic!("Wrong packet type"),
        }
    }

    #[test]
    fn test_invalid_crc() {
        let packet = Packet::Reset(ResetCommand::all());
        let mut serialized = serialize_packet(&packet);

        serialized[CRC_START] = !serialized[CRC_START];

        assert!(deserialize_with_crc(&serialized).is_none());
    }

    #[test]
    fn test_invalid_sync_header() {
        let mut serialized = serialize_packet(&Packet::Reset(ResetCommand::all()));

        // Corrupt sync header
        serialized[0] = 0x00;
        assert!(deserialize_with_crc(&serialized).is_none());
    }

    #[test]
    fn test_invalid_cobs_length_zero() {
        let mut serialized = serialize_packet(&Packet::Reset(ResetCommand::all()));

        // Set invalid COBS length
        serialized[SYNC_HEADER_LEN] = 0;
        assert!(deserialize_with_crc(&serialized).is_none());
    }

    #[test]
    fn test_invalid_cobs_length_overflow() {
        let mut serialized = serialize_packet(&Packet::Reset(ResetCommand::all()));

        // Set invalid COBS length
        serialized[SYNC_HEADER_LEN] = MAX_COBS_LEN as u8 + 1;
        assert!(deserialize_with_crc(&serialized).is_none());
    }

    #[test]
    fn test_corrupted_cobs_data() {
        let mut serialized = serialize_packet(&Packet::Reset(ResetCommand::all()));

        // Corrupt COBS data
        serialized[PAYLOAD_START] = !serialized[PAYLOAD_START];
        assert!(deserialize_with_crc(&serialized).is_none());
    }

    #[test]
    fn test_buffer_size_mismatch() {
        let small_buf = [0u8; BUFFER_SIZE - 1];
        assert!(deserialize_with_crc(&small_buf).is_none());
    }

    #[test]
    fn test_crc_calculation_range() {
        let packet = Packet::Reset(ResetCommand::all());
        let serialized = serialize_packet(&packet);
        let used_len = serialized[SYNC_HEADER_LEN] as usize;

        let crc_range_to_check = ..PAYLOAD_START + used_len;
        let expected_crc = COMMS_CRC.checksum(&serialized[crc_range_to_check]);

        let stored_crc = u16::from_be_bytes([serialized[CRC_START], serialized[CRC_START + 1]]);

        assert_eq!(stored_crc, expected_crc);
    }

    #[test]
    fn test_empty_payload() {
        // Should fail because cobs_len=0 is invalid
        let mut serialized = serialize_packet(&Packet::Reset(ResetCommand::all()));
        serialized[SYNC_HEADER_LEN] = 0; // Force zero length
        assert!(deserialize_with_crc(&serialized).is_none());
    }
}
