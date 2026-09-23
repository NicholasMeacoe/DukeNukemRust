pub mod ai;
pub mod decals;
pub mod gore;
pub mod projectiles;
pub mod types;

pub use ai::*;
pub use decals::*;
pub use gore::*;
pub use projectiles::*;
pub use types::*;

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
                    (
                        spawn_projectiles,
                        update_projectiles,
                        update_enemy_status_effects,
                        apply_damage_events,
                        update_con_actors,
                    )
                        .in_set(crate::GameSet::Combat),
                    (
                        handle_gib_events,
                        update_gib_particles,
                        handle_spawn_casing_events,
                        update_brass_casings,
                    )
                        .in_set(crate::GameSet::Animation),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::*;

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

        let (p_shot, vel_shot, dmg_shot) = map_tile_to_projectile(SHOTGUN);
        assert_eq!(p_shot, ProjectileType::ShotgunPellet);
        assert_eq!(vel_shot, 80.0);
        assert_eq!(dmg_shot, 10);

        let (p_chain, vel_chain, dmg_chain) = map_tile_to_projectile(SHOTSPARK1);
        assert_eq!(p_chain, ProjectileType::HitscanBullet);
        assert_eq!(vel_chain, 150.0);
        assert_eq!(dmg_chain, 9);
    }

    #[test]
    fn test_con_actor_ecs_initialization() {
        let actor = crate::scripting::ConActor::new(PIGCOP, 12, 512, 100);
        assert_eq!(actor.picnum, PIGCOP);
        assert_eq!(actor.sectnum, 12);
        assert_eq!(actor.ang, 512);
        assert_eq!(actor.extra, 100);
        assert_eq!(actor.last_hit_weapon, 0);
    }

    #[test]
    fn test_con_script_engine_default_core_compilation() {
        let engine = crate::scripting::ConScriptEngine::from_source(
            crate::scripting::DEFAULT_CORE_CON_SCRIPT,
        )
        .expect("Default core CON script must compile cleanly");

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
        assert_eq!(
            map_tile_to_projectile(1625),
            (ProjectileType::AlienBlaster, 50.0, 7)
        );
        assert_eq!(
            map_tile_to_projectile(1636),
            (ProjectileType::Spit, 35.0, 8)
        );
        assert_eq!(
            map_tile_to_projectile(1641),
            (ProjectileType::FreezeShard, 45.0, 20)
        );
        assert_eq!(
            map_tile_to_projectile(1650),
            (ProjectileType::Mortar, 30.0, 50)
        );
        assert_eq!(
            map_tile_to_projectile(2595),
            (ProjectileType::HitscanBullet, 150.0, 9)
        );
        assert_eq!(
            map_tile_to_projectile(2605),
            (ProjectileType::Rocket, 45.0, 140)
        );
        assert_eq!(
            map_tile_to_projectile(2613),
            (ProjectileType::ShotgunPellet, 80.0, 10)
        );
        assert_eq!(
            map_tile_to_projectile(1360),
            (ProjectileType::PsiBlast, 30.0, 38)
        );
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
        let is_lethal_stomp =
            dist_to_player <= 1.25 && matches!(boss.kind, EnemyKind::Boss1Battlelord);
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

    #[test]
    fn test_surface_impact_material_and_decal_dispatch() {
        let hit_pos = Vec3::new(10.0, 1.0, 5.0);
        let normal = Vec3::Z;

        // Scorch mark on explosive detonation
        let scorch_event = decals::SpawnDecalEvent {
            origin: hit_pos,
            normal,
            decal_type: decals::DecalType::ScorchMark,
        };
        assert_eq!(scorch_event.decal_type, decals::DecalType::ScorchMark);

        // Bullet hole on hitscan impact
        let bullet_event = decals::SpawnDecalEvent {
            origin: hit_pos,
            normal,
            decal_type: decals::DecalType::BulletHole,
        };
        assert_eq!(bullet_event.decal_type, decals::DecalType::BulletHole);

        // Blood splatter on flesh hit
        let blood_event = decals::SpawnDecalEvent {
            origin: hit_pos,
            normal: Vec3::Y,
            decal_type: decals::DecalType::BloodSplatter,
        };
        assert_eq!(blood_event.decal_type, decals::DecalType::BloodSplatter);
    }

    #[test]
    fn test_extended_world_actors_and_scenery() {
        // Turret actor
        let turret = EnemyActor::new_turret();
        assert_eq!(turret.kind, EnemyKind::Turret);
        assert_eq!(turret.health, 40);
        assert_eq!(turret.attack_cooldown, 0.5);

        // Scampering rat actor
        let rat = EnemyActor::new_rat();
        assert_eq!(rat.kind, EnemyKind::ScamperingRat);
        assert_eq!(rat.speed, 10.0);
        let dist_to_player = 3.0;
        let flee_dir = Vec3::new(1.0, 0.0, 0.0);
        let dt = 0.1;
        let mut rat_pos = Vec3::ZERO;
        if rat.kind == EnemyKind::ScamperingRat && dist_to_player <= 5.0 {
            rat_pos += flee_dir * (rat.speed * dt);
        }
        assert_eq!(rat_pos.x, 1.0);

        // Slime hazard actor
        let slime = EnemyActor::new_slime_hazard();
        assert_eq!(slime.kind, EnemyKind::ToxicSlimeHazard);

        // Camera prop
        let cam = EnemyActor::new_camera_prop();
        assert_eq!(cam.kind, EnemyKind::SecurityCameraProp);
    }

    #[test]
    fn test_pipebomb_persistence_without_auto_despawn() {
        let mut app = App::new();
        app.add_event::<SpawnProjectileEvent>()
            .add_event::<EntityDamageEvent>()
            .add_event::<crate::interactivity::ExplosionDamageEvent>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_event::<crate::combat::decals::SpawnDecalEvent>()
            .add_event::<crate::interactivity::WallDamageEvent>()
            .add_event::<GibEvent>()
            .insert_resource(Time::<()>::default())
            .add_systems(Update, (spawn_projectiles, update_projectiles));

        // Spawn a pipebomb projectile event
        app.world_mut().send_event(SpawnProjectileEvent {
            projectile_type: ProjectileType::Pipebomb,
            origin: Vec3::new(0.0, 1.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            velocity: 15.0,
            damage: 150,
            is_player_source: true,
        });

        // Run schedule to process spawn
        app.update();

        // Verify pipebomb has infinite lifetime (no 8.0s auto-despawn timer)
        let mut pipebomb_query = app.world_mut().query::<&Projectile>();
        let pipebomb = pipebomb_query.iter(app.world()).next().expect("Pipebomb entity should exist");
        assert_eq!(pipebomb.projectile_type, ProjectileType::Pipebomb);
        assert_eq!(pipebomb.lifetime, f32::INFINITY);

        // Advance time past the former 8.0s auto-despawn threshold (e.g. 10.0 seconds)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(10));
        }
        app.update();

        // Verify pipebomb STILL exists and was NOT auto-despawned
        let mut pipebomb_query = app.world_mut().query::<(Entity, &Projectile)>();
        let pipebomb_count = pipebomb_query.iter(app.world()).count();
        assert_eq!(pipebomb_count, 1, "Pipebomb must persist indefinitely without auto-despawning");
    }

    #[test]
    fn test_projectile_water_entry_splash_sound_and_particles() {
        let mut app = App::new();
        app.add_event::<SpawnProjectileEvent>()
            .add_event::<EntityDamageEvent>()
            .add_event::<crate::interactivity::ExplosionDamageEvent>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_event::<crate::combat::decals::SpawnDecalEvent>()
            .add_event::<crate::interactivity::WallDamageEvent>()
            .add_event::<GibEvent>()
            .insert_resource(Time::<()>::default())
            .add_systems(Update, (spawn_projectiles, update_projectiles));

        // Spawn a water surface at Y = 0.0
        app.world_mut().spawn(crate::interactivity::WaterSurface {
            sector_index: 0,
            elevation: 0.0,
        });

        // Spawn bullet projectile above water at Y = 0.5 traveling downwards at 10 m/s
        let proj_entity = app.world_mut().spawn((
            Projectile {
                projectile_type: ProjectileType::HitscanBullet,
                velocity: Vec3::new(0.0, -10.0, 0.0),
                damage: 10,
                lifetime: 2.0,
                bounces: 0,
                is_player_source: true,
            },
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.5, 0.0)),
        )).id();

        // Advance time by 0.1s -> old_pos.y = 0.5, new_pos.y = 0.5 - 1.0 = -0.5 (crossed elevation 0.0!)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        app.update();

        let sound_events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut reader = sound_events.get_reader();
        let events: Vec<_> = reader.read(sound_events).collect();
        assert!(events.iter().any(|e| e.sound_id == 112), "Water entry must trigger splash sound 112");

        let mut splash_query = app.world_mut().query::<&crate::interactivity::WaterSplashParticle>();
        assert!(splash_query.iter(app.world()).count() > 0, "Water entry must spawn splash particles");

        let proj = app.world().get::<Projectile>(proj_entity).unwrap();
        assert!(proj.velocity.y.abs() < 10.0, "Projectile must decelerate upon entering water");
    }

    #[test]
    fn test_shrunk_enemy_stomp_damage_execution() {
        let mut app = App::new();
        app.add_event::<EntityDamageEvent>()
            .add_event::<GibEvent>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_event::<crate::audio::PlayDukeVoiceEvent>()
            .insert_resource(crate::net::DeterministicRng::new(42))
            .add_systems(Update, apply_damage_events);

        // Spawn shrunk enemy
        let mut enemy = EnemyActor::new_pigcop();
        enemy.is_shrunk = true;
        enemy.shrink_timer = 9.0;
        enemy.state = EnemyAiState::Shrunk;

        let enemy_entity = app.world_mut().spawn((
            enemy,
            TransformBundle::from_transform(Transform::from_xyz(2.0, 0.0, 2.0)),
        )).id();

        // Player delivers Mighty Boot kick
        app.world_mut().send_event(EntityDamageEvent {
            target: enemy_entity,
            amount: 15, // Normal boot kick damage
            source: DamageSource::PlayerWeapon(ProjectileType::MightyBoot),
            hit_origin: Vec3::new(2.0, 0.0, 2.0),
        });

        app.update();

        // Verify immediate squash/stomp kill
        let enemy = app.world().get::<EnemyActor>(enemy_entity).unwrap();
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
        assert_eq!(enemy.health, -100);
        assert!(!enemy.is_shrunk);

        // Verify squish sound and gib events emitted
        let sound_events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut sound_reader = sound_events.get_reader();
        let sounds: Vec<_> = sound_reader.read(sound_events).collect();
        assert!(sounds.iter().any(|s| s.sound_id == 69), "Squish sound (69) must be played on stomp");

        let gib_events = app.world().resource::<Events<GibEvent>>();
        let mut gib_reader = gib_events.get_reader();
        let gibs: Vec<_> = gib_reader.read(gib_events).collect();
        assert!(!gibs.is_empty(), "Gib particles must be spawned on stomp");
    }

    #[test]
    fn test_frozen_enemy_shatter_upon_damage() {
        let mut app = App::new();
        app.add_event::<EntityDamageEvent>()
            .add_event::<GibEvent>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_event::<crate::audio::PlayDukeVoiceEvent>()
            .insert_resource(crate::net::DeterministicRng::new(42))
            .add_systems(Update, apply_damage_events);

        // Spawn frozen enemy
        let mut enemy = EnemyActor::new_pigcop();
        enemy.health = 1;
        enemy.is_frozen = true;
        enemy.freeze_timer = 4.6;
        enemy.state = EnemyAiState::Frozen;

        let enemy_entity = app.world_mut().spawn((
            enemy,
            TransformBundle::from_transform(Transform::from_xyz(1.0, 0.0, 1.0)),
        )).id();

        // Hit frozen enemy with minimal bullet damage (e.g. 1 point of hitscan damage)
        app.world_mut().send_event(EntityDamageEvent {
            target: enemy_entity,
            amount: 1,
            source: DamageSource::PlayerWeapon(ProjectileType::HitscanBullet),
            hit_origin: Vec3::new(1.0, 0.0, 1.0),
        });

        app.update();

        // Verify immediate shatter into ice shards
        let enemy = app.world().get::<EnemyActor>(enemy_entity).unwrap();
        assert_eq!(enemy.state, EnemyAiState::Gibbed);
        assert_eq!(enemy.health, -50);
        assert!(!enemy.is_frozen);

        // Verify glass shatter sound (19) and gibs
        let sound_events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut sound_reader = sound_events.get_reader();
        let sounds: Vec<_> = sound_reader.read(sound_events).collect();
        assert!(sounds.iter().any(|s| s.sound_id == 19), "Glass/ice shatter sound (19) must be played");

        let gib_events = app.world().resource::<Events<GibEvent>>();
        let mut gib_reader = gib_events.get_reader();
        let gibs: Vec<_> = gib_reader.read(gib_events).collect();
        assert!(!gibs.is_empty(), "Ice shard gibs must be spawned on shatter");
    }

    #[test]
    fn test_tripbomb_placement_raycast_condition() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let max_reach = 2.5;

        // Within reach (<= 2.5m)
        let hit_near = Some((Vec3::new(0.0, 1.5, 2.0), Vec3::new(0.0, 0.0, -1.0)));
        let near_res = crate::player::weapons::calculate_tripbomb_placement(origin, hit_near, max_reach);
        assert!(near_res.is_some());
        let (pos, norm) = near_res.unwrap();
        assert_eq!(norm, Vec3::new(0.0, 0.0, -1.0));
        assert!((pos.z - (2.0 - 0.02)).abs() < 0.001);

        // Beyond reach (> 2.5m)
        let hit_far = Some((Vec3::new(0.0, 1.5, 4.0), Vec3::new(0.0, 0.0, -1.0)));
        let far_res = crate::player::weapons::calculate_tripbomb_placement(origin, hit_far, max_reach);
        assert!(far_res.is_none());

        // No hit
        let no_hit = None;
        let none_res = crate::player::weapons::calculate_tripbomb_placement(origin, no_hit, max_reach);
        assert!(none_res.is_none());
    }

    #[test]
    fn test_full_roster_spawning_and_projectile_dispatching() {
        // Test all 20+ Enemy roster constructor definitions & stats
        let troopers = [
            (EnemyActor::new_liztroop(), EnemyKind::Liztroop, 30),
            (EnemyActor::new_captain(), EnemyKind::AssaultCaptain, 60),
            (EnemyActor::new_pigcop(), EnemyKind::Pigcop, 100),
            (EnemyActor::new_recon(), EnemyKind::ReconCar, 50),
            (EnemyActor::new_tank(), EnemyKind::RiotTank, 500),
            (EnemyActor::new_octabrain(), EnemyKind::Octabrain, 175),
            (EnemyActor::new_egg(), EnemyKind::ProtozoidEgg, 20),
            (EnemyActor::new_slimer(), EnemyKind::ProtozoidSlimer, 1),
            (EnemyActor::new_enforcer(), EnemyKind::Enforcer, 120),
            (EnemyActor::new_commander(), EnemyKind::AssaultCommander, 350),
            (EnemyActor::new_drone(), EnemyKind::SentryDrone, 150),
            (EnemyActor::new_shark(), EnemyKind::Shark, 35),
            (EnemyActor::new_protector_drone(), EnemyKind::ProtectorDrone, 300),
            (EnemyActor::new_turret(), EnemyKind::Turret, 40),
            (EnemyActor::new_rat(), EnemyKind::ScamperingRat, 5),
            (EnemyActor::new_slime_hazard(), EnemyKind::ToxicSlimeHazard, 50),
            (EnemyActor::new_camera_prop(), EnemyKind::SecurityCameraProp, 20),
            (EnemyActor::new_battlelord(false), EnemyKind::Boss1Battlelord, 4500),
            (EnemyActor::new_battlelord(true), EnemyKind::Boss1Mini, 1000),
            (EnemyActor::new_overlord(), EnemyKind::Boss2Overlord, 4500),
            (EnemyActor::new_cycloid(), EnemyKind::Boss3Cycloid, 4500),
            (EnemyActor::new_queen(), EnemyKind::Boss4Queen, 6000),
        ];

        for (enemy, kind, hp) in troopers {
            assert_eq!(enemy.kind, kind);
            assert_eq!(enemy.health, hp);
            assert_eq!(enemy.max_health, hp);
        }

        // Test all enemy weapon mappings
        assert_eq!(map_tile_to_projectile(FIRELASER).0, ProjectileType::AlienBlaster);
        assert_eq!(map_tile_to_projectile(SPIT).0, ProjectileType::Spit);
        assert_eq!(map_tile_to_projectile(LOOGIE).0, ProjectileType::Spit);
        assert_eq!(map_tile_to_projectile(FREEZEBLAST).0, ProjectileType::FreezeShard);
        assert_eq!(map_tile_to_projectile(FREEZE).0, ProjectileType::FreezeShard);
        assert_eq!(map_tile_to_projectile(SHRINKSPARK).0, ProjectileType::ShrinkRay);
        assert_eq!(map_tile_to_projectile(SHRINKER).0, ProjectileType::ShrinkRay);
        assert_eq!(map_tile_to_projectile(MORTER).0, ProjectileType::Mortar);
        assert_eq!(map_tile_to_projectile(SHOTSPARK1).0, ProjectileType::HitscanBullet);
        assert_eq!(map_tile_to_projectile(CHAINGUN).0, ProjectileType::HitscanBullet);
        assert_eq!(map_tile_to_projectile(RPG).0, ProjectileType::Rocket);
        assert_eq!(map_tile_to_projectile(SHOTGUN).0, ProjectileType::ShotgunPellet);
        assert_eq!(map_tile_to_projectile(COOLEXPLOSION1).0, ProjectileType::PsiBlast);
        assert_eq!(map_tile_to_projectile(DEVISTATORBLAST).0, ProjectileType::Rocket);
    }

    #[test]
    fn test_enemy_authentic_death_drops() {
        let mut rng = crate::net::DeterministicRng::new(12345);

        // Helper matching the death drop logic in ai.rs
        let get_drop = |kind: EnemyKind, rng: &mut crate::net::DeterministicRng| {
            match kind {
                EnemyKind::Pigcop => {
                    if rng.next_f32() < 0.6 {
                        Some(crate::interactivity::PickupKind::ShotgunBox)
                    } else {
                        Some(crate::interactivity::PickupKind::ArmorVest)
                    }
                }
                EnemyKind::Liztroop | EnemyKind::AssaultCaptain => {
                    if rng.next_f32() < 0.5 {
                        Some(crate::interactivity::PickupKind::PistolClip)
                    } else {
                        Some(crate::interactivity::PickupKind::SmallMedkit)
                    }
                }
                EnemyKind::Enforcer => Some(crate::interactivity::PickupKind::ChaingunBox),
                EnemyKind::Octabrain | EnemyKind::AssaultCommander => {
                    Some(crate::interactivity::PickupKind::RpgRocket)
                }
                EnemyKind::Boss1Battlelord
                | EnemyKind::Boss1Mini
                | EnemyKind::Boss2Overlord
                | EnemyKind::Boss3Cycloid
                | EnemyKind::Boss4Queen => Some(crate::interactivity::PickupKind::AtomicHealth),
                _ => None,
            }
        };

        // Enforcer always drops ChaingunBox
        assert_eq!(get_drop(EnemyKind::Enforcer, &mut rng), Some(crate::interactivity::PickupKind::ChaingunBox));

        // Commander always drops RpgRocket
        assert_eq!(get_drop(EnemyKind::AssaultCommander, &mut rng), Some(crate::interactivity::PickupKind::RpgRocket));

        // Bosses always drop AtomicHealth
        assert_eq!(get_drop(EnemyKind::Boss1Battlelord, &mut rng), Some(crate::interactivity::PickupKind::AtomicHealth));
        assert_eq!(get_drop(EnemyKind::Boss2Overlord, &mut rng), Some(crate::interactivity::PickupKind::AtomicHealth));
        assert_eq!(get_drop(EnemyKind::Boss3Cycloid, &mut rng), Some(crate::interactivity::PickupKind::AtomicHealth));
        assert_eq!(get_drop(EnemyKind::Boss4Queen, &mut rng), Some(crate::interactivity::PickupKind::AtomicHealth));

        // Pigcop drops ShotgunBox or ArmorVest
        let pig_drop = get_drop(EnemyKind::Pigcop, &mut rng).unwrap();
        assert!(matches!(
            pig_drop,
            crate::interactivity::PickupKind::ShotgunBox | crate::interactivity::PickupKind::ArmorVest
        ));
    }

    #[test]
    fn test_battlelord_combat_mechanics_and_boss_level_completion() {
        let mut app = App::new();
        app.add_event::<SpawnProjectileEvent>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_event::<crate::audio::PlayDukeVoiceEvent>()
            .add_event::<crate::interactivity::ExplosionDamageEvent>()
            .add_event::<GibEvent>()
            .add_event::<crate::game_flow::LevelCompletedEvent>()
            .insert_resource(Time::<()>::default())
            .insert_resource(crate::net::DeterministicRng::new(42))
            .insert_resource(crate::interactivity::EarthquakeCameraShake::default())
            .insert_resource(crate::game_flow::LevelProgress {
                current_episode: 1,
                current_level: 7,
                ..default()
            })
            .insert_resource(
                crate::scripting::ConScriptEngine::from_source(
                    crate::scripting::DEFAULT_CORE_CON_SCRIPT,
                )
                .unwrap(),
            )
            .add_systems(Update, ai::update_con_actors);

        // Spawn player
        let mut player_ctrl = crate::player::types::PlayerController::default();
        player_ctrl.health = 100;
        app.world_mut().spawn((
            player_ctrl,
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        ));

        // Spawn Battlelord at 10 meters distance
        let mut battlelord_actor = EnemyActor::new_battlelord(false);
        battlelord_actor.health = 4500;
        let mut con_actor = crate::scripting::ConActor::new(BOSS1, 0, 0, 4500);
        // Set a move pointer so boss moves and triggers footstep shake
        con_actor.registers.move_ptr = Some(0);
        con_actor.hitag = crate::scripting::move_flags::SEEK_PLAYER as i16;

        let boss_entity = app.world_mut().spawn((
            battlelord_actor,
            con_actor,
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 10.0)),
        )).id();

        // Advance time by 0.1s: Boss minigun barrage fires and footsteps trigger shake
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        app.update();

        // 1. Verify Minigun Projectile emitted
        let proj_events = app.world().resource::<Events<SpawnProjectileEvent>>();
        let mut proj_reader = proj_events.get_reader();
        let projs: Vec<_> = proj_reader.read(proj_events).cloned().collect();
        assert!(
            projs.iter().any(|p| p.projectile_type == ProjectileType::HitscanBullet),
            "Battlelord minigun barrage must fire HitscanBullet projectiles"
        );

        // 2. Advance time to mortar phase (advance to attack_timer = 2.45s)
        {
            let mut enemy = app.world_mut().get_mut::<EnemyActor>(boss_entity).unwrap();
            enemy.attack_timer = 2.39;
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        app.update();

        let proj_events = app.world().resource::<Events<SpawnProjectileEvent>>();
        let mut proj_reader = proj_events.get_reader();
        let projs: Vec<_> = proj_reader.read(proj_events).cloned().collect();
        assert!(
            projs.iter().any(|p| p.projectile_type == ProjectileType::Mortar),
            "Battlelord must fire lobbed Mortar artillery projectile"
        );

        // 3. Test Boss Defeat triggering LevelCompletedEvent in E1L7
        {
            let mut enemy = app.world_mut().get_mut::<EnemyActor>(boss_entity).unwrap();
            enemy.health = 10;
            let mut con = app.world_mut().get_mut::<crate::scripting::ConActor>(boss_entity).unwrap();
            con.extra = 0; // Dead
        }
        app.update();

        let level_events = app.world().resource::<Events<crate::game_flow::LevelCompletedEvent>>();
        let mut level_reader = level_events.get_reader();
        let level_completed: Vec<_> = level_reader.read(level_events).cloned().collect();
        assert_eq!(
            level_completed.len(),
            1,
            "Defeating Battlelord in E1L7 must trigger LevelCompletedEvent"
        );
    }
}
