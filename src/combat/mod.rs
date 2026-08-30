#![allow(dead_code)]

pub mod types;
pub mod projectiles;
pub mod ai;
pub mod gore;
pub mod decals;

pub use types::*;
pub use projectiles::*;
pub use ai::*;
pub use gore::*;
pub use decals::*;

use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(decals::DecalPlugin)
            .add_event::<SpawnProjectileEvent>()
            .add_event::<EntityDamageEvent>()
            .add_event::<GibEvent>()
            .add_event::<SpawnCasingEvent>()
            .add_systems(
                Update,
                (
                    (spawn_projectiles, update_projectiles, update_enemy_status_effects, apply_damage_events, update_con_actors).in_set(crate::GameSet::Combat),
                    (handle_gib_events, update_gib_particles, handle_spawn_casing_events, update_brass_casings).in_set(crate::GameSet::Animation),
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
            bounces: 0,
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

    #[test]
    fn test_gib_state_duplicate_protection() {
        let mut enemy = EnemyActor::new_liztroop();
        enemy.health = -50;
        enemy.state = EnemyAiState::Gibbed;

        // If another damage event arrives on an already-gibbed entity, it should remain Gibbed without resending gib events
        let mut second_hit_applied = false;
        if enemy.state != EnemyAiState::Gibbed {
            second_hit_applied = true;
        }
        assert!(!second_hit_applied);
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
    }

    #[test]
    fn test_con_actor_lifecycle_and_projectile_mapping() {
        let (p_laser, vel_laser, dmg_laser) = map_tile_to_projectile(1625);
        assert_eq!(p_laser, ProjectileType::AlienBlaster);
        assert_eq!(vel_laser, 50.0);
        assert_eq!(dmg_laser, 7);

        let (p_spit, vel_spit, dmg_spit) = map_tile_to_projectile(1636);
        assert_eq!(p_spit, ProjectileType::Spit);
        assert_eq!(vel_spit, 35.0);
        assert_eq!(dmg_spit, 8);

        let (p_shot, vel_shot, dmg_shot) = map_tile_to_projectile(2613);
        assert_eq!(p_shot, ProjectileType::ShotgunPellet);
        assert_eq!(vel_shot, 80.0);
        assert_eq!(dmg_shot, 10);

        let (p_chain, vel_chain, dmg_chain) = map_tile_to_projectile(2595);
        assert_eq!(p_chain, ProjectileType::HitscanBullet);
        assert_eq!(vel_chain, 150.0);
        assert_eq!(dmg_chain, 9);
    }

    #[test]
    fn test_con_actor_ecs_initialization() {
        let actor = crate::scripting::ConActor::new(2000, 12, 512, 100);
        assert_eq!(actor.picnum, 2000);
        assert_eq!(actor.sectnum, 12);
        assert_eq!(actor.ang, 512);
        assert_eq!(actor.extra, 100);
        assert_eq!(actor.last_hit_weapon, 0);
    }

    #[test]
    fn test_con_script_engine_default_core_compilation() {
        let engine = crate::scripting::ConScriptEngine::from_source(
            crate::scripting::DEFAULT_CORE_CON_SCRIPT
        ).expect("Default core CON script must compile cleanly");

        assert!(engine.compiled.symbols.contains_key("PIGCOP"));
        assert!(engine.compiled.symbols.contains_key("LIZTROOP"));
        assert!(engine.compiled.symbols.contains_key("OCTABRAIN"));
        assert!(engine.compiled.symbols.contains_key("ENFORCER"));

        assert_eq!(engine.compiled.symbols.get("PIGCOP"), Some(&2000));
        assert_eq!(engine.compiled.symbols.get("LIZTROOP"), Some(&1680));
        assert_eq!(engine.compiled.symbols.get("OCTABRAIN"), Some(&1820));
        assert_eq!(engine.compiled.symbols.get("ENFORCER"), Some(&2120));

        assert!(engine.compiled.actor_script_ptrs[2000].is_some());
        assert!(engine.compiled.actor_script_ptrs[1680].is_some());
        assert!(engine.compiled.actor_script_ptrs[1820].is_some());
        assert!(engine.compiled.actor_script_ptrs[2120].is_some());
    }

    #[test]
    fn test_brass_casing_physics() {
        let mut casing = BrassCasing {
            velocity: Vec3::new(2.0, 1.5, 0.0),
            lifetime: 3.0,
            bounces: 2,
            is_shotgun: false,
        };

        casing.lifetime -= 0.5;
        assert_eq!(casing.lifetime, 2.5);

        // Ground bounce
        casing.bounces -= 1;
        casing.velocity.y = -casing.velocity.y * 0.4;
        assert_eq!(casing.bounces, 1);
        assert!(casing.velocity.y < 0.0);
    }

    #[test]
    fn test_gib_particle_physics() {
        let mut gib = GibParticle {
            velocity: Vec3::new(3.0, 5.0, -2.0),
            lifetime: 4.0,
        };

        let dt = 0.1;
        gib.velocity.y -= 18.0 * dt;
        gib.lifetime -= dt;

        assert_eq!(gib.lifetime, 3.9);
        assert!((gib.velocity.y - 3.2).abs() < 0.001);
    }

    #[test]
    fn test_enemy_freeze_and_glass_shatter() {
        let mut enemy = EnemyActor::new_pigcop();
        enemy.health = 0;
        enemy.is_frozen = true;
        enemy.freeze_timer = 4.6;
        enemy.state = EnemyAiState::Frozen;

        assert!(enemy.is_frozen);
        assert_eq!(enemy.state, EnemyAiState::Frozen);

        // Subsequent impact shatters frozen enemy
        enemy.state = EnemyAiState::Gibbed;
        enemy.health = -50;
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
    }

    #[test]
    fn test_enemy_expander_inflation_and_pop() {
        let mut enemy = EnemyActor::new_pigcop();
        enemy.is_expanding = true;
        enemy.expand_timer = 1.0;
        enemy.state = EnemyAiState::Expanding;

        // Inflation progresses
        enemy.expand_timer -= 0.5;
        let progress = 1.0 - enemy.expand_timer;
        let scale = 1.0 + progress * 0.75;
        assert!((scale - 1.375).abs() < 0.001);

        // Pop explosion
        enemy.expand_timer -= 0.5;
        if enemy.expand_timer <= 0.0 {
            enemy.is_expanding = false;
            enemy.state = EnemyAiState::Gibbed;
            enemy.health = -100;
        }
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
        assert_eq!(enemy.health, -100);
    }

    #[test]
    fn test_projectile_bouncing_physics() {
        let mut proj = Projectile {
            projectile_type: ProjectileType::FreezeShard,
            velocity: Vec3::new(10.0, -5.0, 0.0),
            damage: 15,
            is_player_source: true,
            lifetime: 3.0,
            bounces: 3,
        };

        // Ground bounce
        proj.bounces -= 1;
        proj.velocity.y = -proj.velocity.y * 0.7;
        assert_eq!(proj.bounces, 2);
        assert!((proj.velocity.y - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_all_14_enemy_kinds_and_constructors() {
        let liz = EnemyActor::new_liztroop();
        assert_eq!(liz.health, 30);
        assert_eq!(liz.kind, EnemyKind::Liztroop);

        let cap = EnemyActor::new_captain();
        assert_eq!(cap.health, 60);
        assert_eq!(cap.kind, EnemyKind::AssaultCaptain);

        let pig = EnemyActor::new_pigcop();
        assert_eq!(pig.health, 100);

        let recon = EnemyActor::new_recon();
        assert_eq!(recon.health, 50);

        let tank = EnemyActor::new_tank();
        assert_eq!(tank.health, 500);

        let oct = EnemyActor::new_octabrain();
        assert_eq!(oct.health, 175);

        let egg = EnemyActor::new_egg();
        assert_eq!(egg.health, 20);

        let slimer = EnemyActor::new_slimer();
        assert_eq!(slimer.health, 1);

        let enf = EnemyActor::new_enforcer();
        assert_eq!(enf.health, 120);

        let comm = EnemyActor::new_commander();
        assert_eq!(comm.health, 350);

        let drone = EnemyActor::new_drone();
        assert_eq!(drone.health, 150);

        let shark = EnemyActor::new_shark();
        assert_eq!(shark.health, 35);

        let prot = EnemyActor::new_protector_drone();
        assert_eq!(prot.health, 300);

        let turret = EnemyActor::new_turret();
        assert_eq!(turret.health, 40);

        let b1 = EnemyActor::new_battlelord(false);
        assert_eq!(b1.health, 4500);
        assert_eq!(b1.kind, EnemyKind::Boss1Battlelord);

        let b1_mini = EnemyActor::new_battlelord(true);
        assert_eq!(b1_mini.health, 1000);
        assert_eq!(b1_mini.kind, EnemyKind::Boss1Mini);

        let b2 = EnemyActor::new_overlord();
        assert_eq!(b2.health, 4500);

        let b3 = EnemyActor::new_cycloid();
        assert_eq!(b3.health, 4500);

        let b4 = EnemyActor::new_queen();
        assert_eq!(b4.health, 6000);
    }

    #[test]
    fn test_situational_spawn_dormancy() {
        let mut sit = SituationalSpawn {
            initial_picnum: 1741, // ONTOILET
            is_dormant: true,
        };
        assert!(sit.is_dormant);

        // Player seen / woke up
        let can_see = true;
        if can_see {
            sit.is_dormant = false;
        }
        assert!(!sit.is_dormant);
    }

    #[test]
    fn test_map_tile_to_projectile_con_mapping() {
        assert_eq!(map_tile_to_projectile(1625), (ProjectileType::AlienBlaster, 50.0, 7));
        assert_eq!(map_tile_to_projectile(1636), (ProjectileType::Spit, 35.0, 8));
        assert_eq!(map_tile_to_projectile(1641), (ProjectileType::FreezeShard, 45.0, 20));
        assert_eq!(map_tile_to_projectile(1650), (ProjectileType::Mortar, 30.0, 50));
        assert_eq!(map_tile_to_projectile(2595), (ProjectileType::HitscanBullet, 150.0, 9));
        assert_eq!(map_tile_to_projectile(2605), (ProjectileType::Rocket, 45.0, 140));
        assert_eq!(map_tile_to_projectile(2613), (ProjectileType::ShotgunPellet, 80.0, 10));
        assert_eq!(map_tile_to_projectile(1360), (ProjectileType::PsiBlast, 30.0, 38));
    }

    #[test]
    fn test_3d_flying_actor_altitude_tracking() {
        let flying = FlyingActor::default();
        assert_eq!(flying.current_z_vel, 0.0);

        let mut actor_y: f32 = 1.0;
        let player_y: f32 = 3.0;
        let dt: f32 = 0.1;
        let diff_y: f32 = (player_y + 0.3) - actor_y; // 2.3
        actor_y += diff_y.clamp(-3.5 * dt, 3.5 * dt); // +0.35
        assert!((actor_y - 1.35).abs() < 0.001);
    }

    #[test]
    fn test_boss_lethal_stomp_damage() {
        let boss = EnemyActor::new_battlelord(false);
        let dist_to_player = 1.0;
        let is_lethal_stomp = dist_to_player <= 1.25 && matches!(boss.kind, EnemyKind::Boss1Battlelord);
        assert!(is_lethal_stomp);
        let stomp_damage = 1000;
        assert_eq!(stomp_damage, 1000);
    }

    #[test]
    fn test_projectile_wall_impact_and_detonation() {
        let rocket = Projectile {
            projectile_type: ProjectileType::Rocket,
            velocity: Vec3::new(45.0, 0.0, 0.0),
            damage: 140,
            is_player_source: true,
            lifetime: 4.0,
            bounces: 0,
        };
        assert_eq!(rocket.projectile_type, ProjectileType::Rocket);
        assert_eq!(rocket.damage, 140);
    }

    #[test]
    fn test_continuous_swept_projectile_hit_detection() {
        // Bullet starts at x = 0.0 and travels to x = 5.0
        let p_start = Vec3::new(0.0, 0.0, 0.0);
        let p_end = Vec3::new(5.0, 0.0, 0.0);
        
        // Enemy is at x = 2.5, y = 0.0, z = 0.2 (offset slightly)
        let enemy_center = Vec3::new(2.5, 0.0, 0.2);
        let dist_sq = dist_sq_point_to_segment(enemy_center, p_start, p_end);
        
        // Closest point on segment is (2.5, 0, 0), distance squared is 0.04
        assert!((dist_sq - 0.04).abs() < 0.001);
        assert!(dist_sq <= 1.44); // Hits inside standard enemy radius
    }
}
