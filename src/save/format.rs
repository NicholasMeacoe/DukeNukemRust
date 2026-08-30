#![allow(dead_code)]

use crate::player::types::{PlayerController, WeaponType};

pub const SAVEGAME_MAGIC: &[u8; 4] = b"DUKE";
pub const BYTEVERSION: u32 = 116;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveGameSnapshot {
    pub magic: [u8; 4],
    pub version: u32,
    pub title: String,
    pub timestamp: u64,
    pub episode: u8,
    pub level: u8,
    pub skill: u8,

    // Player State
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub velocity_y: f32,
    pub health: i32,
    pub max_health: i32,
    pub armor: i32,
    pub max_armor: i32,
    pub current_weapon: u8,
    pub pistol_mag: i32,
    pub devastator_alt_side: bool,

    // Weapons (12 total)
    pub weapons: [(i32, bool); 12], // (ammo, is_unlocked)

    // Inventory
    pub steroids_amount: i32,
    pub steroids_active: bool,
    pub medkit_amount: i32,
    pub nightvision_amount: i32,
    pub nightvision_active: bool,
    pub scuba_amount: i32,
    pub boots_amount: i32,
    pub holoduke_amount: i32,
    pub holoduke_active: bool,
    pub jetpack_amount: i32,
    pub jetpack_active: bool,
    pub air_supply: f32,

    // Keys
    pub has_blue_key: bool,
    pub has_red_key: bool,
    pub has_yellow_key: bool,

    // Level Stats
    pub kills_count: i32,
    pub secrets_found: i32,
    pub level_time_seconds: f32,
}

impl SaveGameSnapshot {
    pub fn new(
        title: &str,
        episode: u8,
        level: u8,
        skill: u8,
        player_pos: (f32, f32, f32),
        player: &PlayerController,
        kills: i32,
        secrets: i32,
        time_secs: f32,
    ) -> Self {
        let mut weapons = [(0, false); 12];
        for (i, w) in player.weapons.iter().enumerate() {
            if i < 12 {
                weapons[i] = (w.ammo, w.is_unlocked);
            }
        }

        Self {
            magic: *SAVEGAME_MAGIC,
            version: BYTEVERSION,
            title: title.chars().take(19).collect(),
            timestamp: 0,
            episode,
            level,
            skill,
            pos_x: player_pos.0,
            pos_y: player_pos.1,
            pos_z: player_pos.2,
            yaw: player.yaw,
            pitch: player.pitch,
            velocity_y: player.velocity_y,
            health: player.health,
            max_health: player.max_health,
            armor: player.armor,
            max_armor: player.max_armor,
            current_weapon: player.current_weapon as u8,
            pistol_mag: player.pistol_mag,
            devastator_alt_side: player.devastator_alt_side,
            weapons,
            steroids_amount: player.inventory.steroids_amount,
            steroids_active: player.inventory.steroids_active,
            medkit_amount: player.inventory.medkit_amount,
            nightvision_amount: player.inventory.nightvision_amount,
            nightvision_active: player.inventory.nightvision_active,
            scuba_amount: player.inventory.scuba_amount,
            boots_amount: player.inventory.boots_amount,
            holoduke_amount: player.inventory.holoduke_amount,
            holoduke_active: player.inventory.holoduke_active,
            jetpack_amount: player.inventory.jetpack_amount,
            jetpack_active: player.inventory.jetpack_active,
            air_supply: player.inventory.air_supply,
            has_blue_key: player.has_blue_key,
            has_red_key: player.has_red_key,
            has_yellow_key: player.has_yellow_key,
            kills_count: kills,
            secrets_found: secrets,
            level_time_seconds: time_secs,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());

        // 20-byte fixed title buffer
        let mut title_buf = [0u8; 20];
        let title_bytes = self.title.as_bytes();
        let copy_len = title_bytes.len().min(19);
        title_buf[..copy_len].copy_from_slice(&title_bytes[..copy_len]);
        bytes.extend_from_slice(&title_buf);

        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.push(self.episode);
        bytes.push(self.level);
        bytes.push(self.skill);

        bytes.extend_from_slice(&self.pos_x.to_le_bytes());
        bytes.extend_from_slice(&self.pos_y.to_le_bytes());
        bytes.extend_from_slice(&self.pos_z.to_le_bytes());
        bytes.extend_from_slice(&self.yaw.to_le_bytes());
        bytes.extend_from_slice(&self.pitch.to_le_bytes());
        bytes.extend_from_slice(&self.velocity_y.to_le_bytes());

        bytes.extend_from_slice(&self.health.to_le_bytes());
        bytes.extend_from_slice(&self.max_health.to_le_bytes());
        bytes.extend_from_slice(&self.armor.to_le_bytes());
        bytes.extend_from_slice(&self.max_armor.to_le_bytes());
        bytes.push(self.current_weapon);
        bytes.extend_from_slice(&self.pistol_mag.to_le_bytes());
        bytes.push(if self.devastator_alt_side { 1 } else { 0 });

        for &(ammo, unlocked) in &self.weapons {
            bytes.extend_from_slice(&ammo.to_le_bytes());
            bytes.push(if unlocked { 1 } else { 0 });
        }

        bytes.extend_from_slice(&self.steroids_amount.to_le_bytes());
        bytes.push(if self.steroids_active { 1 } else { 0 });
        bytes.extend_from_slice(&self.medkit_amount.to_le_bytes());
        bytes.extend_from_slice(&self.nightvision_amount.to_le_bytes());
        bytes.push(if self.nightvision_active { 1 } else { 0 });
        bytes.extend_from_slice(&self.scuba_amount.to_le_bytes());
        bytes.extend_from_slice(&self.boots_amount.to_le_bytes());
        bytes.extend_from_slice(&self.holoduke_amount.to_le_bytes());
        bytes.push(if self.holoduke_active { 1 } else { 0 });
        bytes.extend_from_slice(&self.jetpack_amount.to_le_bytes());
        bytes.push(if self.jetpack_active { 1 } else { 0 });
        bytes.extend_from_slice(&self.air_supply.to_le_bytes());

        bytes.push(if self.has_blue_key { 1 } else { 0 });
        bytes.push(if self.has_red_key { 1 } else { 0 });
        bytes.push(if self.has_yellow_key { 1 } else { 0 });

        bytes.extend_from_slice(&self.kills_count.to_le_bytes());
        bytes.extend_from_slice(&self.secrets_found.to_le_bytes());
        bytes.extend_from_slice(&self.level_time_seconds.to_le_bytes());

        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 160 {
            return Err("Save file too small");
        }

        if &data[0..4] != SAVEGAME_MAGIC {
            return Err("Invalid savegame magic");
        }

        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version != BYTEVERSION {
            return Err("Savegame version mismatch");
        }

        let title_bytes = &data[8..28];
        let title_end = title_bytes.iter().position(|&b| b == 0).unwrap_or(20);
        let title = String::from_utf8_lossy(&title_bytes[..title_end]).to_string();

        let mut offset = 28;
        let timestamp = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let episode = data[offset]; offset += 1;
        let level = data[offset]; offset += 1;
        let skill = data[offset]; offset += 1;

        let pos_x = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let pos_y = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let pos_z = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let yaw = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let pitch = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let velocity_y = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;

        let health = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let max_health = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let armor = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let max_armor = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let current_weapon = data[offset]; offset += 1;
        let pistol_mag = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let devastator_alt_side = data[offset] != 0; offset += 1;

        let mut weapons = [(0, false); 12];
        for i in 0..12 {
            let ammo = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
            let unlocked = data[offset] != 0; offset += 1;
            weapons[i] = (ammo, unlocked);
        }

        let steroids_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let steroids_active = data[offset] != 0; offset += 1;
        let medkit_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let nightvision_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let nightvision_active = data[offset] != 0; offset += 1;
        let scuba_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let boots_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let holoduke_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let holoduke_active = data[offset] != 0; offset += 1;
        let jetpack_amount = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let jetpack_active = data[offset] != 0; offset += 1;
        let air_supply = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;

        let has_blue_key = data[offset] != 0; offset += 1;
        let has_red_key = data[offset] != 0; offset += 1;
        let has_yellow_key = data[offset] != 0; offset += 1;

        let kills_count = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let secrets_found = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap()); offset += 4;
        let level_time_seconds = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());

        Ok(Self {
            magic: *SAVEGAME_MAGIC,
            version,
            title,
            timestamp,
            episode,
            level,
            skill,
            pos_x,
            pos_y,
            pos_z,
            yaw,
            pitch,
            velocity_y,
            health,
            max_health,
            armor,
            max_armor,
            current_weapon,
            pistol_mag,
            devastator_alt_side,
            weapons,
            steroids_amount,
            steroids_active,
            medkit_amount,
            nightvision_amount,
            nightvision_active,
            scuba_amount,
            boots_amount,
            holoduke_amount,
            holoduke_active,
            jetpack_amount,
            jetpack_active,
            air_supply,
            has_blue_key,
            has_red_key,
            has_yellow_key,
            kills_count,
            secrets_found,
            level_time_seconds,
        })
    }

    pub fn apply_to_player(&self, player: &mut PlayerController) {
        player.yaw = self.yaw;
        player.pitch = self.pitch;
        player.velocity_y = self.velocity_y;
        player.health = self.health;
        player.max_health = self.max_health;
        player.armor = self.armor;
        player.max_armor = self.max_armor;
        player.current_weapon = match self.current_weapon {
            0 => WeaponType::Knee,
            1 => WeaponType::Pistol,
            2 => WeaponType::Shotgun,
            3 => WeaponType::Chaingun,
            4 => WeaponType::Rpg,
            5 => WeaponType::Pipebomb,
            6 => WeaponType::Shrinker,
            7 => WeaponType::Devastator,
            8 => WeaponType::Tripbomb,
            9 => WeaponType::Freezethrower,
            10 => WeaponType::HandRemote,
            11 => WeaponType::Expander,
            _ => WeaponType::Pistol,
        };
        player.pistol_mag = self.pistol_mag;
        player.devastator_alt_side = self.devastator_alt_side;

        for (i, &(ammo, unlocked)) in self.weapons.iter().enumerate() {
            if i < player.weapons.len() {
                player.weapons[i].ammo = ammo;
                player.weapons[i].is_unlocked = unlocked;
            }
        }

        player.inventory.steroids_amount = self.steroids_amount;
        player.inventory.steroids_active = self.steroids_active;
        player.inventory.medkit_amount = self.medkit_amount;
        player.inventory.nightvision_amount = self.nightvision_amount;
        player.inventory.nightvision_active = self.nightvision_active;
        player.inventory.scuba_amount = self.scuba_amount;
        player.inventory.boots_amount = self.boots_amount;
        player.inventory.holoduke_amount = self.holoduke_amount;
        player.inventory.holoduke_active = self.holoduke_active;
        player.inventory.jetpack_amount = self.jetpack_amount;
        player.inventory.jetpack_active = self.jetpack_active;
        player.inventory.air_supply = self.air_supply;

        player.has_blue_key = self.has_blue_key;
        player.has_red_key = self.has_red_key;
        player.has_yellow_key = self.has_yellow_key;
    }
}
