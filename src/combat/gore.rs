#![allow(dead_code)]

use bevy::prelude::*;
use crate::combat::types::GibEvent;

#[derive(Component)]
pub struct GibParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct BrassCasing {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub bounces: u8,
    pub is_shotgun: bool,
}

#[derive(Component)]
pub struct BloodDecal {
    pub lifetime: f32,
}

#[derive(Event, Debug, Clone)]
pub struct SpawnCasingEvent {
    pub origin: Vec3,
    pub direction: Vec3,
    pub is_shotgun: bool,
}

pub fn handle_gib_events(
    mut events: EventReader<GibEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        for _ in 0..ev.gib_count {
            let vel = Vec3::new(
                (rand::random::<f32>() - 0.5) * 8.0,
                rand::random::<f32>() * 6.0 + 2.0,
                (rand::random::<f32>() - 0.5) * 8.0,
            );

            commands.spawn((
                GibParticle {
                    velocity: vel,
                    lifetime: 4.0,
                },
                TransformBundle::from_transform(Transform::from_translation(ev.origin)),
                crate::game_flow::LevelEntity,
            ));
        }
    }
}

pub fn handle_spawn_casing_events(
    mut events: EventReader<SpawnCasingEvent>,
    mut commands: Commands,
) {
    for ev in events.read() {
        let right = ev.direction.cross(Vec3::Y).normalize_or_zero();
        let vel = right * (2.0 + rand::random::<f32>() * 2.0) + Vec3::Y * (1.5 + rand::random::<f32>() * 1.5);
        commands.spawn((
            BrassCasing {
                velocity: vel,
                lifetime: 3.0,
                bounces: 2,
                is_shotgun: ev.is_shotgun,
            },
            TransformBundle::from_transform(Transform::from_translation(ev.origin)),
            crate::game_flow::LevelEntity,
        ));
    }
}

pub fn update_brass_casings(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BrassCasing)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut trans, mut casing) in query.iter_mut() {
        casing.lifetime -= dt;
        casing.velocity.y -= 18.0 * dt;
        trans.translation += casing.velocity * dt;

        // Ground bounce check
        if trans.translation.y <= 0.05 && casing.bounces > 0 {
            casing.bounces -= 1;
            casing.velocity.y = -casing.velocity.y * 0.4;
            casing.velocity.x *= 0.6;
            casing.velocity.z *= 0.6;
            trans.translation.y = 0.05;
        }

        if casing.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn update_gib_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GibParticle)>,
) {
    let dt = time.delta_seconds();

    for (entity, mut trans, mut gib) in query.iter_mut() {
        gib.lifetime -= dt;
        gib.velocity.y -= 18.0 * dt; // Gravity
        trans.translation += gib.velocity * dt;

        if gib.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
