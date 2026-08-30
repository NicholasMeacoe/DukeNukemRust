#![allow(dead_code)]

pub const DEMO_BYTEVERSION: u8 = 116;

pub mod demo_bits {
    pub const JUMP: u32        = 1 << 0;
    pub const CROUCH: u32      = 1 << 1;
    pub const FIRE: u32        = 1 << 2;
    pub const AIM_UP: u32      = 1 << 3;
    pub const AIM_DOWN: u32    = 1 << 4;
    pub const RUN: u32         = 1 << 5;
    pub const LOOK_LEFT: u32   = 1 << 6;
    pub const LOOK_RIGHT: u32  = 1 << 7;
    pub const STEROIDS: u32    = 1 << 12;
    pub const LOOK_UP: u32     = 1 << 13;
    pub const LOOK_DOWN: u32   = 1 << 14;
    pub const NIGHTVISION: u32 = 1 << 15;
    pub const MEDKIT: u32      = 1 << 16;
    pub const CENTER_VIEW: u32 = 1 << 18;
    pub const QUICK_KICK: u32  = 1 << 22;
    pub const HOLODUKE: u32    = 1 << 24;
    pub const JETPACK: u32     = 1 << 25;
    pub const TURN_AROUND: u32 = 1 << 28;
    pub const OPEN: u32        = 1 << 29;
    pub const INVENTORY: u32   = 1 << 30;
    pub const ESCAPE: u32      = 1 << 31;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameticInput {
    pub avel: i8,   // Yaw rotation velocity delta (-127..127)
    pub horz: i8,   // Pitch rotation delta (-127..127)
    pub fvel: i16,  // Forward / Backward movement velocity
    pub svel: i16,  // Strafe Left / Right movement velocity
    pub bits: u32,  // Action bitflags (see demo_bits)
}

impl GameticInput {
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        buf[0] = self.avel as u8;
        buf[1] = self.horz as u8;
        buf[2..4].copy_from_slice(&self.fvel.to_le_bytes());
        buf[4..6].copy_from_slice(&self.svel.to_le_bytes());
        buf[6..10].copy_from_slice(&self.bits.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < 10 {
            return Err("Input buffer too short");
        }
        let avel = buf[0] as i8;
        let horz = buf[1] as i8;
        let fvel = i16::from_le_bytes(buf[2..4].try_into().unwrap());
        let svel = i16::from_le_bytes(buf[4..6].try_into().unwrap());
        let bits = u32::from_le_bytes(buf[6..10].try_into().unwrap());
        Ok(Self { avel, horz, fvel, svel, bits })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoHeader {
    pub total_tics: u32,
    pub version: u8,
    pub episode: u8,
    pub level: u8,
    pub skill: u8,
}

impl DemoHeader {
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.total_tics.to_le_bytes());
        buf[4] = self.version;
        buf[5] = self.episode;
        buf[6] = self.level;
        buf[7] = self.skill;
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < 8 {
            return Err("Demo header too short");
        }
        let total_tics = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let version = buf[4];
        let episode = buf[5];
        let level = buf[6];
        let skill = buf[7];
        Ok(Self { total_tics, version, episode, level, skill })
    }
}
