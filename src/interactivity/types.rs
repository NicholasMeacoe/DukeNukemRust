use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        target_pos: Vec3,
        is_submerged: bool,
    },
    /// SE 3: Light Strobe / Flicker
    LightStrobe {
        base_shade: i8,
        min_shade: i8,
        max_shade: i8,
        timer: f32,
        rate: f32,
    },
    /// SE 12: Light Switch Operator
    LightSwitchOperator {
        is_on: bool,
        on_shade: i8,
        off_shade: i8,
    },
    /// SE 21: Drop Floor / Ceiling Crusher
    DropFloor {
        orig_floor_z: i32,
        target_floor_z: i32,
        current_floor_z: i32,
        speed: i32,
        is_dropped: bool,
    },
    /// SE 25: Rotating Engine / Gears
    RotatingEngine {
        pivot: Vec2,
        current_ang: f32,
        speed: f32,
    },
    /// SE 30: Two-Way Subway Train / Moving Sector
    SubwayTrain {
        stop_a: Vec2,
        stop_b: Vec2,
        current_pos: Vec2,
        progress: f32,
        speed: f32,
        moving_to_b: bool,
        pause_timer: f32,
    },
    /// SE 10: Auto-Close Door (Vertical door)
    AutoCloseDoor {
        orig_ceil_z: i32,
        open_ceil_z: i32,
        current_ceil_z: i32,
        speed: i32,
        auto_close_timer: Option<f32>,
        auto_close_delay: f32,
        is_open: bool,
    },
    /// SE 1: Pivot Rotating Sector
    PivotRotatingSector {
        pivot: Vec2,
        orig_ang: f32,
        target_ang: f32,
        current_ang: f32,
        speed: f32,
        is_open: bool,
    },
    /// SE 2 & 22: Earthquake Camera Shake
    Earthquake {
        intensity: f32,
        duration: f32,
        elapsed: f32,
        is_triggered: bool,
    },
    /// SE 4 & 5: Random Light Flicker / Light Buzz
    RandomFlicker {
        base_shade: i8,
        min_shade: i8,
        max_shade: i8,
        timer: f32,
        is_buzz: bool,
    },
    /// SE 11: Continuous Rotating Sector
    ContinuousRotation {
        pivot: Vec2,
        current_ang: f32,
        angular_speed: f32,
    },
    /// SE 12: Smooth Light Glow Gradient
    GlowGradient {
        min_shade: i8,
        max_shade: i8,
        current_shade: f32,
        rate: f32,
        increasing: bool,
    },
    /// SE 20: Stretch Ceiling
    StretchCeiling {
        orig_ceil_z: i32,
        target_ceil_z: i32,
        current_ceil_z: i32,
        speed: i32,
        is_stretched: bool,
    },
    /// SE 24: Conveyor Belt Floor Velocity
    ConveyorBelt { direction: Vec2, speed: f32 },
    /// SE 31 & 32: Crusher Sectors (Floor rise / Ceiling lower)
    CrusherSector {
        min_z: i32,
        max_z: i32,
        current_z: i32,
        speed: i32,
        crushing_ceiling: bool,
        moving_down: bool,
    },
    /// SE 36: Shooting Breakable Glass Pane
    ShootingGlassPane { health: i32, is_shattered: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SectorMeshPart {
    Floor,
    Ceiling,
    UpperWall,
    LowerWall,
    MiddleWall,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DynamicSectorMesh {
    pub sector_idx: usize,
    pub part: SectorMeshPart,
}

impl DynamicSectorMesh {
    pub fn new(sector_idx: usize, part: SectorMeshPart) -> Self {
        Self { sector_idx, part }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WaterSurface {
    pub sector_index: usize,
    pub elevation: f32,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WaterSplashParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SectorEffectorComponent {
    pub sector_idx: usize,
    pub lotag: i16,
    pub hitag: i16,
    pub kind: EffectorKind,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeycardPickup {
    pub key_type: u8, // 1 = Blue, 2 = Red, 3 = Yellow
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InteractiveSwitch {
    pub switch_type: SwitchType,
    pub on_tile: i16,
    pub off_tile: i16,
    pub is_on: bool,
    pub lotag: i16,
    pub hitag: i16,
    pub sound_id: i32,
    #[serde(skip)]
    pub material_handle: Option<Handle<StandardMaterial>>,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WaterFountain {
    pub uses_left: i32,
    pub is_broken: bool,
    pub broken_tile: i16,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToiletProp {
    pub is_broken: bool,
    pub broken_tile: i16,
    pub water_tile: i16,
    pub last_used_time: f32,
    pub cooldown_timer: f32,
}

impl Default for ToiletProp {
    fn default() -> Self {
        Self {
            is_broken: false,
            broken_tile: 970,
            water_tile: 971,
            last_used_time: 0.0,
            cooldown_timer: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BreakableGlass {
    pub health: i32,
    pub is_broken: bool,
    pub wall_idx: Option<usize>,
    pub sector_idx: Option<usize>,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplodingBarrel {
    pub health: i32,
    pub damage_radius: f32,
    pub damage: i32,
    pub is_exploded: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewscreenProp {
    pub camera_tag: i16,
    pub is_active: bool,
    pub is_broken: bool,
    pub broken_tile: i16,
    pub scanline_timer: f32,
}

impl Default for ViewscreenProp {
    fn default() -> Self {
        Self {
            camera_tag: 0,
            is_active: false,
            is_broken: false,
            broken_tile: 501, // VIEWSCREENBROKE
            scanline_timer: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityCamera {
    pub tag: i16,
    pub sweep_angle: f32,
    pub sweep_speed: f32,
    pub base_yaw: f32,
}

impl Default for SecurityCamera {
    fn default() -> Self {
        Self {
            tag: 0,
            sweep_angle: 0.0,
            sweep_speed: 1.0,
            base_yaw: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PickupKind {
    // Health
    SmallMedkit,    // +10 HP (up to 100) - Tile 51
    LargeMedkit,    // +30 HP (up to 100) - Tile 52
    PortableMedkit, // +100 Portable Medkit - Tile 53
    AtomicHealth,   // +50 HP (up to 200) - Tile 55
    ArmorVest,      // 100 Armor - Tile 56
    // Ammo
    PistolClip,    // +12 Pistol Ammo - Tile 40
    ShotgunBox,    // +10 Shotgun Ammo - Tile 49
    ChaingunBox,   // +50 Chaingun Ammo - Tile 44
    RpgRocket,     // +5 Rockets - Tile 47
    PipebombBox,   // +5 Pipebombs - Tile 48
    ShrinkerAmmo,  // +5 Shrinker - Tile 42
    DevastatorBox, // +15 Devastator - Tile 45
    FreezeAmmo,    // +25 Freeze - Tile 46
    ExpanderAmmo,  // +20 Expander - Tile 45
    // Inventory Items
    Steroids,           // +400 Steroids - Tile 57
    ScubaTank,          // +100 Scuba - Tile 59
    NightvisionGoggles, // +100 Nightvision - Tile 60
    ProtectiveBoots,    // +100 Boots - Tile 61
    Jetpack,            // +100 Jetpack - Tile 58
    Holoduke,           // +100 Holoduke - Tile 62
    // Weapons on Ground
    WeaponPistol,     // Tile 21
    WeaponShotgun,    // Tile 22
    WeaponChaingun,   // Tile 23
    WeaponRpg,        // Tile 24
    WeaponPipebomb,   // Tile 25
    WeaponShrinker,   // Tile 26
    WeaponDevastator, // Tile 27
    WeaponTripbomb,   // Tile 28
    WeaponFreezer,    // Tile 29
    WeaponExpander,   // Tile 32
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemPickup {
    pub kind: PickupKind,
    pub respawn_timer: Option<f32>,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrackWall {
    pub health: i32,
    pub stage: u8,
    pub lotag: i16,
    pub is_blown: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Touchplate {
    pub lotag: i16,
    pub hitag: i16,
    pub radius: f32,
    pub triggered: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activator {
    pub lotag: i16,
    pub hitag: i16,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NukeExitSwitch {
    pub is_activated: bool,
    pub is_secret: bool,
    pub lotag: i16,
    pub hitag: i16,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FireExtinguisher {
    pub health: i32,
    pub is_exploded: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MasterSwitch {
    pub lotag: i16,
    pub hitag: i16,
    #[serde(default)]
    pub delay: f32,
    #[serde(default)]
    pub timer: Option<f32>,
    #[serde(default)]
    pub is_triggered: bool,
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
    pub attacker_id: Option<usize>,
}

#[derive(Event, Debug, Clone)]
pub struct PlayerHealEvent {
    pub amount: i32,
}

pub use crate::audio::PlaySoundEvent;

#[derive(Event, Debug, Clone)]
pub struct BarrelExplodeEvent {
    pub origin: Vec3,
    pub radius: f32,
    pub damage: i32,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorProp {
    pub cooldown_timer: f32,
}

impl Default for MirrorProp {
    fn default() -> Self {
        Self {
            cooldown_timer: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct CarrierPlatform {
    pub velocity: Vec3,
    pub sector_bounds_min: Vec2,
    pub sector_bounds_max: Vec2,
    pub sector_idx: Option<usize>,
}

impl Default for CarrierPlatform {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            sector_bounds_min: Vec2::splat(-1000.0),
            sector_bounds_max: Vec2::splat(1000.0),
            sector_idx: None,
        }
    }
}

impl CarrierPlatform {
    pub fn new(velocity: Vec3) -> Self {
        Self {
            velocity,
            sector_bounds_min: Vec2::splat(-1000.0),
            sector_bounds_max: Vec2::splat(1000.0),
            sector_idx: None,
        }
    }

    pub fn with_sector(velocity: Vec3, sector_idx: usize) -> Self {
        Self {
            velocity,
            sector_bounds_min: Vec2::splat(-1000.0),
            sector_bounds_max: Vec2::splat(1000.0),
            sector_idx: Some(sector_idx),
        }
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct EarthquakeCameraShake {
    pub intensity: f32,
    pub offset: Vec3,
}

impl EarthquakeCameraShake {
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            offset: Vec3::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carrier_platform_constructors() {
        let p1 = CarrierPlatform::new(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(p1.velocity, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(p1.sector_idx, None);

        let p2 = CarrierPlatform::with_sector(Vec3::Y, 5);
        assert_eq!(p2.velocity, Vec3::Y);
        assert_eq!(p2.sector_idx, Some(5));
    }

    #[test]
    fn test_earthquake_camera_shake_new() {
        let shake = EarthquakeCameraShake::new();
        assert_eq!(shake.intensity, 0.0);
        assert_eq!(shake.offset, Vec3::ZERO);
    }
}
