#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileType {
    HitscanBullet,
    ShotgunPellet,
    MightyBoot,
    Rocket,
    Pipebomb,
    ShrinkRay,
    DevastatorMissile,
    FreezeShard,
    ExpanderRay,
    AlienBlaster,
    PsiBlast,
    Mortar,
    Spit,
}

#[derive(Event, Debug, Clone)]
pub struct SpawnProjectileEvent {
    pub projectile_type: ProjectileType,
    pub origin: Vec3,
    pub direction: Vec3,
    pub velocity: f32,
    pub damage: i32,
    pub is_player_source: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub projectile_type: ProjectileType,
    pub velocity: Vec3,
    pub damage: i32,
    pub is_player_source: bool,
    pub lifetime: f32,
    pub bounces: u8,
}

#[derive(Component, Debug, Clone)]
pub struct LaserTripbomb {
    pub normal: Vec3,
    pub arm_timer: f32,
    pub is_armed: bool,
    pub beam_length: f32,
    pub damage: i32,
    pub damage_radius: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct FlyingActor {
    pub current_z_vel: f32,
    pub target_altitude: f32,
}

#[derive(Component, Debug, Clone)]
pub struct SituationalSpawn {
    pub initial_picnum: i16,
    pub is_dormant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteViewType {
    SingleView,
    FiveViewMirror,
    EightView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Liztroop,
    AssaultCaptain,
    Pigcop,
    ReconCar,
    RiotTank,
    Octabrain,
    ProtozoidEgg,
    ProtozoidSlimer,
    Enforcer,
    AssaultCommander,
    SentryDrone,
    Shark,
    ProtectorDrone,
    Turret,
    Boss1Battlelord,
    Boss1Mini,
    Boss2Overlord,
    Boss3Cycloid,
    Boss4Queen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyAiState {
    Idle,
    Patrol,
    Seeking,
    Attacking,
    Flinching,
    Frozen,
    Expanding,
    Dying,
    Gibbed,
}

#[derive(Component, Debug, Clone)]
pub struct EnemyActor {
    pub kind: EnemyKind,
    pub state: EnemyAiState,
    pub health: i32,
    pub max_health: i32,
    pub speed: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub sight_radius: f32,
    pub attack_range: f32,
    pub is_shrunk: bool,
    pub shrink_timer: f32,
    pub is_frozen: bool,
    pub freeze_timer: f32,
    pub is_expanding: bool,
    pub expand_timer: f32,
}

impl EnemyActor {
    pub fn new_pigcop() -> Self {
        Self {
            kind: EnemyKind::Pigcop,
            state: EnemyAiState::Idle,
            health: 100,
            max_health: 100,
            speed: 4.5,
            attack_cooldown: 1.2,
            attack_timer: 0.0,
            sight_radius: 25.0,
            attack_range: 15.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_liztroop() -> Self {
        Self {
            kind: EnemyKind::Liztroop,
            state: EnemyAiState::Idle,
            health: 30,
            max_health: 30,
            speed: 5.0,
            attack_cooldown: 0.8,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 20.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_captain() -> Self {
        Self {
            kind: EnemyKind::AssaultCaptain,
            state: EnemyAiState::Idle,
            health: 60,
            max_health: 60,
            speed: 5.5,
            attack_cooldown: 0.7,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 22.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_octabrain() -> Self {
        Self {
            kind: EnemyKind::Octabrain,
            state: EnemyAiState::Idle,
            health: 175,
            max_health: 175,
            speed: 3.5,
            attack_cooldown: 1.5,
            attack_timer: 0.0,
            sight_radius: 25.0,
            attack_range: 18.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_enforcer() -> Self {
        Self {
            kind: EnemyKind::Enforcer,
            state: EnemyAiState::Idle,
            health: 120,
            max_health: 120,
            speed: 6.0,
            attack_cooldown: 0.4,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 20.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_recon() -> Self {
        Self {
            kind: EnemyKind::ReconCar,
            state: EnemyAiState::Idle,
            health: 50,
            max_health: 50,
            speed: 7.0,
            attack_cooldown: 0.6,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 25.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_tank() -> Self {
        Self {
            kind: EnemyKind::RiotTank,
            state: EnemyAiState::Idle,
            health: 500,
            max_health: 500,
            speed: 2.0,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
            sight_radius: 35.0,
            attack_range: 30.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_egg() -> Self {
        Self {
            kind: EnemyKind::ProtozoidEgg,
            state: EnemyAiState::Idle,
            health: 20,
            max_health: 20,
            speed: 0.0,
            attack_cooldown: 5.0,
            attack_timer: 0.0,
            sight_radius: 10.0,
            attack_range: 5.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_slimer() -> Self {
        Self {
            kind: EnemyKind::ProtozoidSlimer,
            state: EnemyAiState::Idle,
            health: 1,
            max_health: 1,
            speed: 6.0,
            attack_cooldown: 0.5,
            attack_timer: 0.0,
            sight_radius: 15.0,
            attack_range: 2.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_commander() -> Self {
        Self {
            kind: EnemyKind::AssaultCommander,
            state: EnemyAiState::Idle,
            health: 350,
            max_health: 350,
            speed: 4.0,
            attack_cooldown: 1.2,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 25.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_drone() -> Self {
        Self {
            kind: EnemyKind::SentryDrone,
            state: EnemyAiState::Idle,
            health: 150,
            max_health: 150,
            speed: 8.0,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 20.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_shark() -> Self {
        Self {
            kind: EnemyKind::Shark,
            state: EnemyAiState::Idle,
            health: 35,
            max_health: 35,
            speed: 5.0,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
            sight_radius: 20.0,
            attack_range: 3.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_protector_drone() -> Self {
        Self {
            kind: EnemyKind::ProtectorDrone,
            state: EnemyAiState::Idle,
            health: 300,
            max_health: 300,
            speed: 7.5,
            attack_cooldown: 0.6,
            attack_timer: 0.0,
            sight_radius: 30.0,
            attack_range: 22.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_turret() -> Self {
        Self {
            kind: EnemyKind::Turret,
            state: EnemyAiState::Idle,
            health: 40,
            max_health: 40,
            speed: 0.0,
            attack_cooldown: 0.5,
            attack_timer: 0.0,
            sight_radius: 35.0,
            attack_range: 30.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_battlelord(is_mini: bool) -> Self {
        let (kind, hp) = if is_mini {
            (EnemyKind::Boss1Mini, 1000)
        } else {
            (EnemyKind::Boss1Battlelord, 4500)
        };
        Self {
            kind,
            state: EnemyAiState::Idle,
            health: hp,
            max_health: hp,
            speed: 4.0,
            attack_cooldown: 0.8,
            attack_timer: 0.0,
            sight_radius: 40.0,
            attack_range: 35.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_overlord() -> Self {
        Self {
            kind: EnemyKind::Boss2Overlord,
            state: EnemyAiState::Idle,
            health: 4500,
            max_health: 4500,
            speed: 4.5,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
            sight_radius: 40.0,
            attack_range: 35.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_cycloid() -> Self {
        Self {
            kind: EnemyKind::Boss3Cycloid,
            state: EnemyAiState::Idle,
            health: 4500,
            max_health: 4500,
            speed: 4.5,
            attack_cooldown: 0.9,
            attack_timer: 0.0,
            sight_radius: 45.0,
            attack_range: 40.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }

    pub fn new_queen() -> Self {
        Self {
            kind: EnemyKind::Boss4Queen,
            state: EnemyAiState::Idle,
            health: 6000,
            max_health: 6000,
            speed: 3.5,
            attack_cooldown: 1.2,
            attack_timer: 0.0,
            sight_radius: 45.0,
            attack_range: 40.0,
            is_shrunk: false,
            shrink_timer: 0.0,
            is_frozen: false,
            freeze_timer: 0.0,
            is_expanding: false,
            expand_timer: 0.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct GibEvent {
    pub origin: Vec3,
    pub gib_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageSource {
    PlayerWeapon(ProjectileType),
    EnemyWeapon(ProjectileType),
    Explosion,
    Environmental,
}

#[derive(Event, Debug, Clone)]
pub struct EntityDamageEvent {
    pub target: Entity,
    pub amount: i32,
    pub source: DamageSource,
    pub hit_origin: Vec3,
}
