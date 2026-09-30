use bevy::prelude::*;
use crate::interactivity::SafeDespawnExt;
use crate::player::weapons::WeaponType;

/// Configuration for dynamic point lighting in the engine.
#[derive(Resource, Clone, Debug)]
pub struct DynamicLightingConfig {
    pub enabled: bool,
    pub max_active_lights: usize,
    pub intensity_scale: f32,
}

impl Default for DynamicLightingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_lights: 32,
            intensity_scale: 1.0,
        }
    }
}

/// Falloff curves for decaying transient lights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightDecayMode {
    Linear,
    Exponential,
    Flicker,
}

/// A short-lived point light attached to muzzle flashes, explosions, or impacts.
#[derive(Component, Clone, Debug)]
pub struct TransientLight {
    pub max_lifetime: f32,
    pub current_time: f32,
    pub initial_intensity: f32,
    pub initial_range: f32,
    pub decay_mode: LightDecayMode,
}

impl TransientLight {
    pub fn new(lifetime: f32, intensity: f32, range: f32, decay_mode: LightDecayMode) -> Self {
        Self {
            max_lifetime: lifetime.max(0.001),
            current_time: 0.0,
            initial_intensity: intensity,
            initial_range: range,
            decay_mode,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.current_time / self.max_lifetime).clamp(0.0, 1.0)
    }

    pub fn calculate_intensity(&self, progress: f32, seed: f32) -> f32 {
        let remaining = (1.0 - progress).clamp(0.0, 1.0);
        match self.decay_mode {
            LightDecayMode::Linear => self.initial_intensity * remaining,
            LightDecayMode::Exponential => self.initial_intensity * remaining * remaining,
            LightDecayMode::Flicker => {
                let jitter = 0.75 + 0.25 * (seed * 13.37).sin().abs();
                self.initial_intensity * remaining * jitter
            }
        }
    }
}

/// Persistent dynamic light attached to traveling projectiles (e.g. RPG rockets, shrinker sparks).
#[derive(Component, Clone, Debug)]
pub struct ProjectileLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
}

/// Event requesting creation of a transient dynamic point light.
#[derive(Event, Clone, Debug)]
pub struct SpawnDynamicLightEvent {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub lifetime: f32,
    pub decay_mode: LightDecayMode,
}

impl SpawnDynamicLightEvent {
    pub fn muzzle_flash(pos: Vec3, weapon: WeaponType) -> Self {
        let (color, intensity, range, lifetime) = match weapon {
            WeaponType::Pistol => (Color::srgb(1.0, 0.85, 0.3), 3_500.0, 7.0, 0.06),
            WeaponType::Shotgun => (Color::srgb(1.0, 0.7, 0.2), 6_000.0, 9.0, 0.08),
            WeaponType::Chaingun => (Color::srgb(1.0, 0.85, 0.3), 4_500.0, 8.0, 0.05),
            WeaponType::Rpg => (Color::srgb(1.0, 0.5, 0.1), 12_000.0, 14.0, 0.12),
            WeaponType::Devastator => (Color::srgb(1.0, 0.6, 0.2), 6_000.0, 9.0, 0.08),
            WeaponType::Shrinker => (Color::srgb(0.3, 1.0, 0.2), 4_500.0, 7.0, 0.08),
            WeaponType::Freezethrower => (Color::srgb(0.3, 0.85, 1.0), 4_000.0, 7.0, 0.07),
            WeaponType::Expander => (Color::srgb(0.85, 0.35, 1.0), 5_000.0, 8.0, 0.07),
            WeaponType::Pipebomb
            | WeaponType::Tripbomb
            | WeaponType::Knee
            | WeaponType::HandRemote => (Color::srgb(1.0, 0.8, 0.4), 1_000.0, 4.0, 0.04),
        };
        Self {
            position: pos,
            color,
            intensity,
            range,
            lifetime,
            decay_mode: LightDecayMode::Exponential,
        }
    }

    pub fn explosion(pos: Vec3, radius: f32) -> Self {
        let radius_scale = (radius / 100.0).clamp(0.5, 4.0);
        Self {
            position: pos,
            color: Color::srgb(1.0, 0.55, 0.12),
            intensity: 450_000.0 * radius_scale,
            range: (16.0 * radius_scale).clamp(8.0, 40.0),
            lifetime: (0.35 * radius_scale.sqrt()).clamp(0.2, 0.6),
            decay_mode: LightDecayMode::Exponential,
        }
    }
}

/// System that handles spawning transient lights while enforcing maximum capacity.
pub fn handle_spawn_dynamic_lights(
    mut commands: Commands,
    config: Res<DynamicLightingConfig>,
    mut events: EventReader<SpawnDynamicLightEvent>,
    existing_lights: Query<(Entity, &TransientLight)>,
) {
    if !config.enabled {
        events.clear();
        return;
    }

    let mut current_count = existing_lights.iter().count();
    let mut evicted_entities: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    for ev in events.read() {
        // Enforce light capacity by despawning the light closest to expiration
        if current_count >= config.max_active_lights {
            if let Some((oldest_entity, _)) = existing_lights
                .iter()
                .filter(|(e, l)| !evicted_entities.contains(e) && l.progress() < 1.0)
                .max_by(|a, b| a.1.progress().partial_cmp(&b.1.progress()).unwrap_or(std::cmp::Ordering::Equal))
            {
                evicted_entities.insert(oldest_entity);
                commands.safe_despawn_recursive(oldest_entity);
                current_count = current_count.saturating_sub(1);
            }
        }

        let initial_intensity = ev.intensity * config.intensity_scale;
        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    color: ev.color,
                    intensity: initial_intensity,
                    range: ev.range,
                    shadows_enabled: false,
                    ..default()
                },
                transform: Transform::from_translation(ev.position),
                ..default()
            },
            TransientLight::new(ev.lifetime, initial_intensity, ev.range, ev.decay_mode),
        ));
        current_count += 1;
    }
}

/// System that steps transient light lifetimes, decays intensities, and despawns expired entities.
pub fn update_transient_lights(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TransientLight, &mut PointLight)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut light, mut point_light) in &mut query {
        light.current_time += dt;
        let progress = light.progress();
        if progress >= 1.0 {
            commands.safe_despawn_recursive(entity);
        } else {
            point_light.intensity = light.calculate_intensity(progress, light.current_time);
        }
    }
}

/// System that automatically spawns child point lights for projectiles carrying `ProjectileLight`.
pub fn attach_projectile_lights(
    mut commands: Commands,
    config: Res<DynamicLightingConfig>,
    query: Query<(Entity, &ProjectileLight), Added<ProjectileLight>>,
) {
    if !config.enabled {
        return;
    }

    for (entity, proj_light) in &query {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(PointLightBundle {
                point_light: PointLight {
                    color: proj_light.color,
                    intensity: proj_light.intensity * config.intensity_scale,
                    range: proj_light.range,
                    shadows_enabled: false,
                    ..default()
                },
                ..default()
            });
        });
    }
}

pub struct DynamicLightingPlugin;

impl Plugin for DynamicLightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicLightingConfig>()
            .add_event::<SpawnDynamicLightEvent>()
            .add_systems(
                Update,
                (
                    handle_spawn_dynamic_lights,
                    update_transient_lights,
                    attach_projectile_lights,
                ),
            );
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_transient_light_linear_and_exponential_decay() {
        let linear = TransientLight::new(1.0, 1000.0, 10.0, LightDecayMode::Linear);
        assert_eq!(linear.progress(), 0.0);
        assert!((linear.calculate_intensity(0.0, 0.0) - 1000.0).abs() < 0.01);
        assert!((linear.calculate_intensity(0.5, 0.0) - 500.0).abs() < 0.01);
        assert!((linear.calculate_intensity(1.0, 0.0) - 0.0).abs() < 0.01);

        let expo = TransientLight::new(1.0, 1000.0, 10.0, LightDecayMode::Exponential);
        // at 50% progress, (0.5)^2 * 1000 = 250
        assert!((expo.calculate_intensity(0.5, 0.0) - 250.0).abs() < 0.01);
    }

    #[test]
    fn test_muzzle_flash_light_events() {
        let pistol_flash = SpawnDynamicLightEvent::muzzle_flash(Vec3::new(1.0, 2.0, 3.0), WeaponType::Pistol);
        assert_eq!(pistol_flash.position, Vec3::new(1.0, 2.0, 3.0));
        assert!(pistol_flash.intensity > 1_000.0);
        assert!(pistol_flash.lifetime < 0.15);

        let rpg_flash = SpawnDynamicLightEvent::muzzle_flash(Vec3::ZERO, WeaponType::Rpg);
        assert!(rpg_flash.intensity > pistol_flash.intensity);
        assert!(rpg_flash.range > pistol_flash.range);
    }

    #[test]
    fn test_explosion_light_event_scaling() {
        let small_boom = SpawnDynamicLightEvent::explosion(Vec3::ZERO, 50.0);
        let big_boom = SpawnDynamicLightEvent::explosion(Vec3::ZERO, 200.0);
        assert!(big_boom.intensity > small_boom.intensity);
        assert!(big_boom.range > small_boom.range);
        assert_eq!(big_boom.decay_mode, LightDecayMode::Exponential);
    }

    #[test]
    fn test_dynamic_lighting_config_default() {
        let cfg = DynamicLightingConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.max_active_lights, 32);
        assert_eq!(cfg.intensity_scale, 1.0);
    }

    #[test]
    fn test_dynamic_lighting_pool_max_capacity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(DynamicLightingConfig {
                enabled: true,
                max_active_lights: 5,
                intensity_scale: 1.0,
            })
            .add_event::<SpawnDynamicLightEvent>()
            .add_systems(Update, handle_spawn_dynamic_lights);

        // Send 10 light spawn events
        for i in 0..10 {
            app.world_mut().send_event(SpawnDynamicLightEvent {
                position: Vec3::new(i as f32, 0.0, 0.0),
                color: Color::WHITE,
                intensity: 1000.0,
                range: 10.0,
                lifetime: 0.5 + (i as f32) * 0.1,
                decay_mode: LightDecayMode::Linear,
            });
            app.update();
        }

        let mut query = app.world_mut().query::<&TransientLight>();
        let active_count = query.iter(app.world()).count();
        assert!(active_count <= 5, "Dynamic light count should not exceed max_active_lights: got {}", active_count);
    }

    #[test]
    fn test_transient_light_despawn_on_expiration() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .add_systems(Update, update_transient_lights);

        let entity = app.world_mut().spawn((
            PointLightBundle {
                point_light: PointLight {
                    intensity: 1000.0,
                    ..default()
                },
                ..default()
            },
            TransientLight::new(0.1, 1000.0, 10.0, LightDecayMode::Linear),
        )).id();

        // Advance time by 0.05s - should still exist
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(50));
        }
        app.update();
        assert!(app.world().get_entity(entity).is_some());

        // Advance time by another 0.10s - total 0.15s >= 0.10s - should be despawned
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        app.update();
        assert!(app.world().get_entity(entity).is_none());
    }

    #[test]
    fn test_dynamic_lights_batch_eviction_in_same_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(DynamicLightingConfig {
                enabled: true,
                max_active_lights: 3,
                intensity_scale: 1.0,
            })
            .add_event::<SpawnDynamicLightEvent>()
            .add_systems(Update, handle_spawn_dynamic_lights);

        // First frame: spawn 3 lights to reach capacity
        for i in 0..3 {
            app.world_mut().send_event(SpawnDynamicLightEvent {
                position: Vec3::new(i as f32, 0.0, 0.0),
                color: Color::WHITE,
                intensity: 1000.0,
                range: 10.0,
                lifetime: 0.5 + (i as f32) * 0.1,
                decay_mode: LightDecayMode::Linear,
            });
        }
        app.update();

        let mut query = app.world_mut().query::<&TransientLight>();
        assert_eq!(query.iter(app.world()).count(), 3);

        // Second frame: send 3 MORE light events simultaneously in a single frame batch
        for i in 3..6 {
            app.world_mut().send_event(SpawnDynamicLightEvent {
                position: Vec3::new(i as f32, 0.0, 0.0),
                color: Color::WHITE,
                intensity: 1000.0,
                range: 10.0,
                lifetime: 0.5 + (i as f32) * 0.1,
                decay_mode: LightDecayMode::Linear,
            });
        }
        // Update must safely evict without duplicate despawn error or panic
        app.update();

        let mut query2 = app.world_mut().query::<&TransientLight>();
        let active_count = query2.iter(app.world()).count();
        assert!(active_count <= 3, "Dynamic light count should not exceed max_active_lights: got {}", active_count);
    }
}
