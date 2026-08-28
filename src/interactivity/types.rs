#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum EffectorKind {
    /// SE 0: Rotating / Pivot Door
    RotatingDoor {
        pivot: Vec2,
        orig_ang: f32,
        target_ang: f32,
        current_ang: f32,
        speed: f32,
        is_open: bool,
    },
    /// SE 15: Sliding Door
    SlidingDoor {
        orig_pos: Vec2,
        open_offset: Vec2,
        progress: f32, // 0.0 (closed) to 1.0 (fully open)
        speed: f32,
        auto_close_timer: Option<f32>,
        auto_close_delay: f32,
        is_open: bool,
    },
    /// SE 17 & 18: Elevator / Lift (Two-stop or Multi-stop)
    Elevator {
        orig_floor_z: i32,
        target_floor_z: i32,
        current_floor_z: i32,
        orig_ceil_z: i32,
        target_ceil_z: i32,
        current_ceil_z: i32,
        speed: i32,
        is_at_top: bool,
        auto_return_timer: Option<f32>,
    },
    /// SE 7: Underwater / Teleporter Conveyor
    UnderwaterTeleport {
        target_sector: usize,
    },
    /// SE 3: Light Strobe / Flicker
    LightStrobe {
        base_shade: i8,
        min_shade: i8,
        max_shade: i8,
        timer: f32,
        rate: f32,
    },
}

#[derive(Component, Debug, Clone)]
pub struct SectorEffectorComponent {
    pub sector_idx: usize,
    pub lotag: i16,
    pub hitag: i16,
    pub kind: EffectorKind,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchType {
    LightSwitch,
    SpaceDoorSwitch,
    DipSwitch,
    TechSwitch,
    PowerSwitch,
    LockSwitch,
    HandSwitch,
    PullSwitch,
    AlienSwitch,
    AccessSwitch { key_required: u8 }, // 1 = Blue, 2 = Red, 3 = Yellow
}

#[derive(Component, Debug, Clone)]
pub struct InteractiveSwitch {
    pub switch_type: SwitchType,
    pub on_tile: i16,
    pub off_tile: i16,
    pub is_on: bool,
    pub lotag: i16,
    pub hitag: i16,
    pub sound_id: i32,
    pub material_handle: Option<Handle<StandardMaterial>>,
}

#[derive(Component, Debug, Clone)]
pub struct WaterFountain {
    pub uses_left: i32,
    pub is_broken: bool,
    pub broken_tile: i16,
}

#[derive(Component, Debug, Clone)]
pub struct ToiletProp {
    pub is_broken: bool,
    pub broken_tile: i16,
    pub water_tile: i16,
    pub last_used_time: f32,
}

#[derive(Component, Debug, Clone)]
pub struct BreakableGlass {
    pub health: i32,
    pub is_broken: bool,
    pub wall_idx: Option<usize>,
    pub sector_idx: Option<usize>,
}

#[derive(Component, Debug, Clone)]
pub struct ExplodingBarrel {
    pub health: i32,
    pub damage_radius: f32,
    pub damage: i32,
    pub is_exploded: bool,
}

#[derive(Component, Debug, Clone)]
pub struct CrackWall {
    pub health: i32,
    pub stage: u8,
    pub lotag: i16,
    pub is_blown: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Touchplate {
    pub lotag: i16,
    pub hitag: i16,
    pub radius: f32,
    pub triggered: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Activator {
    pub lotag: i16,
    pub hitag: i16,
}

#[derive(Component, Debug, Clone)]
pub struct MasterSwitch {
    pub lotag: i16,
    pub hitag: i16,
}

#[derive(Event, Debug, Clone)]
pub struct ActivateTagEvent {
    pub lotag: i16,
}

#[derive(Event, Debug, Clone)]
pub struct InteractEvent {
    pub player_pos: Vec3,
    pub player_dir: Vec3,
}

#[derive(Event, Debug, Clone)]
pub struct ExplosionDamageEvent {
    pub origin: Vec3,
    pub radius: f32,
    pub damage: i32,
}

#[derive(Event, Debug, Clone)]
pub struct PlayerHealEvent {
    pub amount: i32,
}
