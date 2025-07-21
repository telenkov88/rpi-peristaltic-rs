use crate::types::{Packet, BUFFER_SIZE};
use crc::Crc;

pub const COMMS_CRC: Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);

pub fn serialize_packet(packet: &Packet) -> [u8; BUFFER_SIZE] {
    let mut buf = [0u8; BUFFER_SIZE];

    let used = postcard::to_slice_cobs(packet, &mut buf[1..61])
        .expect("Serialization error")
        .len();

    buf[0] = used as u8;

    let crc = COMMS_CRC.checksum(&buf[0..used + 1]);

    buf[62] = (crc >> 8) as u8;
    buf[63] = crc as u8;

    buf
}

pub fn deserialize_with_crc(buf: &[u8]) -> Option<Packet> {
    if buf.len() != BUFFER_SIZE {
        return None;
    }

    // Read the length of COBS data from the first byte
    let cobs_len = buf[0] as usize;

    if cobs_len == 0 || cobs_len > 60 {
        // Max 60 bytes for COBS data
        return None;
    }

    // Extract CRC from the last 2 bytes
    let crc_received = ((buf[62] as u16) << 8) | buf[63] as u16;

    // Calculate CRC on length byte + COBS data
    let crc_calculated = COMMS_CRC.checksum(&buf[0..cobs_len + 1]);

    if crc_received != crc_calculated {
        return None;
    }

    // Decode the COBS data
    let mut packet_buf = [0u8; BUFFER_SIZE];
    let decode_report = cobs::decode(&buf[1..cobs_len + 1], &mut packet_buf).ok()?;
    let len = decode_report.frame_size();

    postcard::from_bytes(&packet_buf[0..len]).ok()
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

        // Corrupt the CRC
        serialized[62] = 0xFF;
        serialized[63] = 0xFF;

        assert!(deserialize_with_crc(&serialized).is_none());
    }
}
