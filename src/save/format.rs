#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::PlayerController;
use crate::combat::types::{EnemyActor, Projectile};
use crate::scripting::ConActor;
use crate::interactivity::types::{
    InteractiveSwitch, SectorEffectorComponent, ItemPickup, CrackWall, BreakableGlass, 
    ExplodingBarrel, WaterFountain, ToiletProp, ViewscreenProp, SecurityCamera, 
    NukeExitSwitch, MasterSwitch
};
use crate::interactivity::props_extended::{DancerProp, SecurityCameraMonitor};

pub const SAVEGAME_MAGIC: &[u8; 4] = b"DUKE";
pub const BYTEVERSION: u32 = 117; // Increment version for new format

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl From<Transform> for SavedTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl Into<Transform> for SavedTransform {
    fn into(self) -> Transform {
        Transform {
            translation: Vec3::from(self.translation),
            rotation: Quat::from_array(self.rotation),
            scale: Vec3::from(self.scale),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveGameSnapshot {
    pub magic: [u8; 4],
    pub version: u32,
    pub title: String,
    pub timestamp: u64,
    pub episode: u8,
    pub level: u8,
    pub skill: u8,
    pub kills_count: i32,
    pub secrets_found: i32,
    pub level_time_seconds: f32,

    pub player: Option<(SavedTransform, PlayerController)>,
    
    // Dynamic Entities
    pub enemies: Vec<(SavedTransform, EnemyActor, ConActor)>,
    pub items: Vec<(SavedTransform, ItemPickup)>,
    pub projectiles: Vec<(SavedTransform, Projectile)>,
    
    // Interactivity
    pub sector_effectors: Vec<(SavedTransform, SectorEffectorComponent)>,
    pub switches: Vec<(SavedTransform, InteractiveSwitch)>,
    pub crack_walls: Vec<(SavedTransform, CrackWall)>,
    pub glass_panes: Vec<(SavedTransform, BreakableGlass)>,
    pub barrels: Vec<(SavedTransform, ExplodingBarrel)>,
    
    // Props
    pub fountains: Vec<(SavedTransform, WaterFountain)>,
    pub toilets: Vec<(SavedTransform, ToiletProp)>,
    pub dancers: Vec<(SavedTransform, DancerProp)>,
    pub viewscreens: Vec<(SavedTransform, ViewscreenProp)>,
    pub cameras: Vec<(SavedTransform, SecurityCamera)>,
    pub camera_monitors: Vec<(SavedTransform, SecurityCameraMonitor)>,
    
    pub nuke_switches: Vec<(SavedTransform, NukeExitSwitch)>,
    pub master_switches: Vec<(SavedTransform, MasterSwitch)>,
}

impl SaveGameSnapshot {
    pub fn new(
        title: &str,
        episode: u8,
        level: u8,
        skill: u8,
        kills: i32,
        secrets: i32,
        time_secs: f32,
    ) -> Self {
        Self {
            magic: *SAVEGAME_MAGIC,
            version: BYTEVERSION,
            title: title.chars().take(19).collect(),
            timestamp: 0,
            episode,
            level,
            skill,
            kills_count: kills,
            secrets_found: secrets,
            level_time_seconds: time_secs,
            player: None,
            enemies: Vec::new(),
            items: Vec::new(),
            projectiles: Vec::new(),
            sector_effectors: Vec::new(),
            switches: Vec::new(),
            crack_walls: Vec::new(),
            glass_panes: Vec::new(),
            barrels: Vec::new(),
            fountains: Vec::new(),
            toilets: Vec::new(),
            dancers: Vec::new(),
            viewscreens: Vec::new(),
            cameras: Vec::new(),
            camera_monitors: Vec::new(),
            nuke_switches: Vec::new(),
            master_switches: Vec::new(),
        }
    }
}
