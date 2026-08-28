#![allow(dead_code)]

pub mod types;
pub mod projectiles;
pub mod ai;
pub mod gore;

pub use types::*;
pub use projectiles::*;
pub use ai::*;
pub use gore::*;

use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnProjectileEvent>()
            .add_event::<EntityDamageEvent>()
            .add_event::<GibEvent>()
            .add_systems(
                Update,
                (
                    (spawn_projectiles, update_projectiles, apply_damage_events, update_enemy_ai).in_set(crate::GameSet::Combat),
                    (handle_gib_events, update_gib_particles).in_set(crate::GameSet::Animation),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_actor_initialization() {
        let pigcop = EnemyActor::new_pigcop();
        assert_eq!(pigcop.health, 100);
        assert_eq!(pigcop.kind, EnemyKind::Pigcop);
        assert_eq!(pigcop.state, EnemyAiState::Idle);

        let liztroop = EnemyActor::new_liztroop();
        assert_eq!(liztroop.health, 30);
        assert_eq!(liztroop.kind, EnemyKind::Liztroop);

        let octabrain = EnemyActor::new_octabrain();
        assert_eq!(octabrain.health, 175);
    }

    #[test]
    fn test_enemy_shrink_and_freeze_status() {
        let mut enemy = EnemyActor::new_pigcop();
        enemy.is_shrunk = true;
        enemy.shrink_timer = 10.0;
        assert!(enemy.is_shrunk);

        enemy.is_frozen = true;
        enemy.freeze_timer = 15.0;
        enemy.state = EnemyAiState::Frozen;
        assert_eq!(enemy.state, EnemyAiState::Frozen);
    }

    #[test]
    fn test_gib_threshold() {
        let mut enemy = EnemyActor::new_liztroop();
        enemy.health -= 80; // Health = -50 (<= -40 gib threshold)
        if enemy.health <= -40 {
            enemy.state = EnemyAiState::Gibbed;
        }
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
    }

    #[test]
    fn test_projectile_creation() {
        let proj = Projectile {
            projectile_type: ProjectileType::Rocket,
            velocity: Vec3::new(0.0, 0.0, -35.0),
            damage: 120,
            is_player_source: true,
            lifetime: 4.0,
        };
        assert_eq!(proj.damage, 120);
        assert_eq!(proj.projectile_type, ProjectileType::Rocket);
    }

    #[test]
    fn test_enemy_ai_state_transition() {
        let mut enemy = EnemyActor::new_pigcop();
        assert_eq!(enemy.state, EnemyAiState::Idle);

        // Player enters sight range -> Seeking
        let dist_to_player = 20.0;
        if dist_to_player <= enemy.sight_radius {
            enemy.state = EnemyAiState::Seeking;
        }
        assert_eq!(enemy.state, EnemyAiState::Seeking);

        // Player enters attack range -> Attacking
        let dist_to_player = 10.0;
        if dist_to_player <= enemy.attack_range {
            enemy.state = EnemyAiState::Attacking;
        }
        assert_eq!(enemy.state, EnemyAiState::Attacking);
    }
}
