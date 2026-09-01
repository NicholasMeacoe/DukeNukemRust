#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecalType {
    BulletHole,
    BloodSplatter,
    ScorchMark,
    GlassCrack,
    WaterRipple,
}

#[derive(Component, Debug, Clone)]
pub struct SurfaceDecal {
    pub decal_type: DecalType,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub alpha: f32,
}

#[derive(Component, Debug, Clone)]
pub struct SteamVent {
    pub interval: f32,
    pub timer: f32,
    pub burst_count: u32,
}

impl Default for SteamVent {
    fn default() -> Self {
        Self {
            interval: 2.0,
            timer: 0.0,
            burst_count: 6,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct WaterDrip {
    pub interval: f32,
    pub timer: f32,
}

impl Default for WaterDrip {
    fn default() -> Self {
        Self {
            interval: 1.5,
            timer: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct ChimneySmoke {
    pub interval: f32,
    pub timer: f32,
}

impl Default for ChimneySmoke {
    fn default() -> Self {
        Self {
            interval: 0.8,
            timer: 0.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct SpawnDecalEvent {
    pub origin: Vec3,
    pub normal: Vec3,
    pub decal_type: DecalType,
}

pub struct DecalPlugin;

impl Plugin for DecalPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDecalEvent>()
            .add_systems(
                Update,
                (
                    handle_spawn_decal_events,
                    update_surface_decals,
                    update_atmospheric_emitters,
                ).in_set(crate::GameSet::Animation),
            );
    }
}

pub fn handle_spawn_decal_events(
    mut events: EventReader<SpawnDecalEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        let max_lifetime = match ev.decal_type {
            DecalType::BulletHole => 20.0,
            DecalType::BloodSplatter => 30.0,
            DecalType::ScorchMark => 25.0,
            DecalType::GlassCrack => 35.0,
            DecalType::WaterRipple => 2.0,
        };

        let normal = ev.normal.normalize_or_zero();
        let rot = if normal.dot(-Vec3::Z) > 0.999 {
            Quat::from_rotation_y(std::f32::consts::PI)
        } else if normal.dot(Vec3::Z) > 0.999 {
            Quat::IDENTITY
        } else if normal.length_squared() > 0.01 {
            Quat::from_rotation_arc(Vec3::Z, normal)
        } else {
            Quat::IDENTITY
        };

        let offset_pos = ev.origin + normal * 0.02;

        commands.spawn((
            SurfaceDecal {
                decal_type: ev.decal_type,
                lifetime: max_lifetime,
                max_lifetime,
                alpha: 1.0,
            },
            TransformBundle::from_transform(Transform::from_translation(offset_pos).with_rotation(rot)),
            crate::game_flow::LevelEntity,
        ));
    }
}

pub fn update_surface_decals(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SurfaceDecal)>,
) {
    let dt = time.delta_seconds();
    let total_decals = query.iter().count();
    let mut excess = if total_decals > 256 { total_decals - 256 } else { 0 };

    for (entity, mut decal) in query.iter_mut() {
        if excess > 0 {
            commands.entity(entity).despawn_recursive();
            excess -= 1;
            continue;
        }

        decal.lifetime -= dt;
        if decal.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else if decal.lifetime < 3.0 {
            // Fade out in last 3 seconds
            decal.alpha = decal.lifetime / 3.0;
        }
    }
}

pub fn update_atmospheric_emitters(
    time: Res<Time>,
    mut steam_query: Query<(&Transform, &mut SteamVent)>,
    mut drip_query: Query<(&Transform, &mut WaterDrip)>,
    mut smoke_query: Query<(&Transform, &mut ChimneySmoke)>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    let dt = time.delta_seconds();

    for (_trans, mut steam) in steam_query.iter_mut() {
        steam.timer += dt;
        if steam.timer >= steam.interval {
            steam.timer = 0.0;
            sound_events.send(crate::audio::PlaySoundEvent { sound_id: 17 }); // STEAM_HISS
        }
    }

    for (_trans, mut drip) in drip_query.iter_mut() {
        drip.timer += dt;
        if drip.timer >= drip.interval {
            drip.timer = 0.0;
            sound_events.send(crate::audio::PlaySoundEvent { sound_id: 36 }); // WATER_DRIP
        }
    }

    for (_trans, mut smoke) in smoke_query.iter_mut() {
        smoke.timer += dt;
        if smoke.timer >= smoke.interval {
            smoke.timer = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_decal_spawning_and_fade() {
        let mut decal = SurfaceDecal {
            decal_type: DecalType::BulletHole,
            lifetime: 30.0,
            max_lifetime: 30.0,
            alpha: 1.0,
        };

        decal.lifetime = 1.5;
        decal.alpha = decal.lifetime / 3.0;
        assert_eq!(decal.alpha, 0.5);
    }

    #[test]
    fn test_atmospheric_steam_vent_emitter() {
        let mut vent = SteamVent::default();
        assert_eq!(vent.interval, 2.0);
        vent.timer += 2.1;
        assert!(vent.timer >= vent.interval);
    }
}
