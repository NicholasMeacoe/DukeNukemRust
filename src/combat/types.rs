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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Liztroop,
    Pigcop,
    Octabrain,
    Enforcer,
    Drone,
    Commander,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyAiState {
    Idle,
    Patrol,
    Seeking,
    Attacking,
    Flinching,
    Frozen,
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
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct GibEvent {
    pub origin: Vec3,
    pub gib_count: usize,
}
