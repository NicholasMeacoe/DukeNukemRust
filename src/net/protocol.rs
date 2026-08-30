#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetMode {
    #[default]
    SinglePlayer,
    Dukematch,
    Cooperative,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetPacket {
    Connect {
        player_id: u8,
        name: String,
        color_pal: u8,
    },
    InputSync {
        player_id: u8,
        gametic: u32,
        forward: i8,
        strafe: i8,
        yaw: i16,
        pitch: i16,
        actions: u16,
        weapon: u8,
    },
    PlayerStateSync {
        player_id: u8,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
        health: i16,
        armor: i16,
    },
    FragEvent {
        killer_id: u8,
        victim_id: u8,
        weapon_type: u8,
    },
    ChatMessage {
        sender_id: u8,
        message: String,
    },
}

impl NetPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            NetPacket::Connect { player_id, name, color_pal } => {
                buf.push(1); // Packet Type ID
                buf.push(*player_id);
                buf.push(*color_pal);
                let safe_name: String = name.chars().take(32).collect();
                let name_bytes = safe_name.as_bytes();
                buf.push(name_bytes.len() as u8);
                buf.extend_from_slice(name_bytes);
            }
            NetPacket::InputSync { player_id, gametic, forward, strafe, yaw, pitch, actions, weapon } => {
                buf.push(2);
                buf.push(*player_id);
                buf.extend_from_slice(&gametic.to_le_bytes());
                buf.push(*forward as u8);
                buf.push(*strafe as u8);
                buf.extend_from_slice(&yaw.to_le_bytes());
                buf.extend_from_slice(&pitch.to_le_bytes());
                buf.extend_from_slice(&actions.to_le_bytes());
                buf.push(*weapon);
            }
            NetPacket::PlayerStateSync { player_id, x, y, z, yaw, pitch, health, armor } => {
                buf.push(3);
                buf.push(*player_id);
                buf.extend_from_slice(&x.to_le_bytes());
                buf.extend_from_slice(&y.to_le_bytes());
                buf.extend_from_slice(&z.to_le_bytes());
                buf.extend_from_slice(&yaw.to_le_bytes());
                buf.extend_from_slice(&pitch.to_le_bytes());
                buf.extend_from_slice(&health.to_le_bytes());
                buf.extend_from_slice(&armor.to_le_bytes());
            }
            NetPacket::FragEvent { killer_id, victim_id, weapon_type } => {
                buf.push(4);
                buf.push(*killer_id);
                buf.push(*victim_id);
                buf.push(*weapon_type);
            }
            NetPacket::ChatMessage { sender_id, message } => {
                buf.push(5);
                buf.push(*sender_id);
                let safe_msg: String = message.chars().take(128).collect();
                let msg_bytes = safe_msg.as_bytes();
                buf.push(msg_bytes.len().min(255) as u8);
                buf.extend_from_slice(msg_bytes);
            }
        }
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.is_empty() {
            return Err("Packet data empty".to_string());
        }

        match data[0] {
            1 => {
                if data.len() < 4 { return Err("Invalid Connect packet".to_string()); }
                let player_id = data[1];
                let color_pal = data[2];
                let name_len = data[3] as usize;
                if data.len() < 4 + name_len { return Err("Truncated name in Connect".to_string()); }
                let name = String::from_utf8_lossy(&data[4..4 + name_len]).to_string();
                Ok(NetPacket::Connect { player_id, name, color_pal })
            }
            2 => {
                if data.len() < 15 { return Err("Invalid InputSync packet".to_string()); }
                let player_id = data[1];
                let gametic = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                let forward = data[6] as i8;
                let strafe = data[7] as i8;
                let yaw = i16::from_le_bytes([data[8], data[9]]);
                let pitch = i16::from_le_bytes([data[10], data[11]]);
                let actions = u16::from_le_bytes([data[12], data[13]]);
                let weapon = data[14];
                Ok(NetPacket::InputSync { player_id, gametic, forward, strafe, yaw, pitch, actions, weapon })
            }
            3 => {
                if data.len() < 26 { return Err("Invalid PlayerStateSync packet".to_string()); }
                let player_id = data[1];
                let x = f32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                let y = f32::from_le_bytes([data[6], data[7], data[8], data[9]]);
                let z = f32::from_le_bytes([data[10], data[11], data[12], data[13]]);
                let yaw = f32::from_le_bytes([data[14], data[15], data[16], data[17]]);
                let pitch = f32::from_le_bytes([data[18], data[19], data[20], data[21]]);
                let health = i16::from_le_bytes([data[22], data[23]]);
                let armor = i16::from_le_bytes([data[24], data[25]]);
                Ok(NetPacket::PlayerStateSync { player_id, x, y, z, yaw, pitch, health, armor })
            }
            4 => {
                if data.len() < 4 { return Err("Invalid FragEvent packet".to_string()); }
                Ok(NetPacket::FragEvent {
                    killer_id: data[1],
                    victim_id: data[2],
                    weapon_type: data[3],
                })
            }
            5 => {
                if data.len() < 3 { return Err("Invalid ChatMessage packet".to_string()); }
                let sender_id = data[1];
                let msg_len = data[2] as usize;
                if data.len() < 3 + msg_len { return Err("Truncated message".to_string()); }
                let message = String::from_utf8_lossy(&data[3..3 + msg_len]).to_string();
                Ok(NetPacket::ChatMessage { sender_id, message })
            }
            _ => Err(format!("Unknown packet type {}", data[0])),
        }
    }
}
