// shared/src/types.rs
use serde::{Deserialize, Serialize};

pub const MAX_ENCODERS: usize = 8;

pub const BUFFER_SIZE: usize = 64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SensorDataPacket {
    pub seq: u32,
    pub encoders: [i32; MAX_ENCODERS],
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResetCommand {
    pub encoder_id: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Packet {
    SensorData(SensorDataPacket),
    Reset(ResetCommand),
    Ping { timestamp: u32 },
    Pong { timestamp: u32 },
}

impl SensorDataPacket {
    pub fn new(seq: u32, encoders: [i32; MAX_ENCODERS]) -> Self {
        Self { seq, encoders }
    }

    pub fn total_movement(&self) -> i32 {
        self.encoders.iter().map(|&x| x.abs()).sum()
    }

    pub fn has_movement(&self, previous: &SensorDataPacket) -> bool {
        self.encoders
            .iter()
            .zip(previous.encoders.iter())
            .any(|(curr, prev)| curr != prev)
    }
}

impl ResetCommand {
    pub fn single(encoder_id: u8) -> Self {
        Self { encoder_id }
    }

    pub fn all() -> Self {
        Self { encoder_id: 255 }
    }

    pub fn resets_all(&self) -> bool {
        self.encoder_id == 255
    }
}
