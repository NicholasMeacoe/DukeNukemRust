#![allow(dead_code)]

use bevy::prelude::*;
use crate::combat::types::GibEvent;

#[derive(Component)]
pub struct GibParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
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
            ));
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
