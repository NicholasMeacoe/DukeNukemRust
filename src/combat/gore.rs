use crate::combat::types::GibEvent;
use crate::interactivity::SafeDespawnExt;
use bevy::prelude::*;

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
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    for ev in events.read() {
        let (mesh_handle, mat_handle) = if let (Some(ref mut m), Some(ref mut mat)) =
            (meshes.as_mut(), materials.as_mut())
        {
            (
                Some(m.add(Cuboid::new(0.12, 0.08, 0.1))),
                Some(mat.add(StandardMaterial {
                    base_color: Color::srgb(0.55, 0.04, 0.04),
                    perceptual_roughness: 0.8,
                    ..default()
                })),
            )
        } else {
            (None, None)
        };

        for _ in 0..ev.gib_count {
            let vel = Vec3::new(
                (rand::random::<f32>() - 0.5) * 8.0,
                rand::random::<f32>() * 6.0 + 2.0,
                (rand::random::<f32>() - 0.5) * 8.0,
            );

            let mut ent = commands.spawn((
                GibParticle {
                    velocity: vel,
                    lifetime: 4.0,
                },
                TransformBundle::from_transform(Transform::from_translation(ev.origin)),
                crate::game_flow::LevelEntity,
            ));

            if let (Some(ref mesh), Some(ref mat)) = (&mesh_handle, &mat_handle) {
                ent.insert(((*mesh).clone(), (*mat).clone()));
            }
        }
    }
}

pub fn handle_spawn_casing_events(
    mut events: EventReader<SpawnCasingEvent>,
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    for ev in events.read() {
        let right = ev.direction.cross(Vec3::Y).normalize_or_zero();
        let vel = right * (2.0 + rand::random::<f32>() * 2.0)
            + Vec3::Y * (1.5 + rand::random::<f32>() * 1.5);

        let mut ent = commands.spawn((
            BrassCasing {
                velocity: vel,
                lifetime: 3.0,
                bounces: 2,
                is_shotgun: ev.is_shotgun,
            },
            TransformBundle::from_transform(Transform::from_translation(ev.origin)),
            crate::game_flow::LevelEntity,
        ));

        if let (Some(ref mut m), Some(ref mut mat)) = (meshes.as_mut(), materials.as_mut()) {
            let (mesh, material) = if ev.is_shotgun {
                (
                    m.add(Cuboid::new(0.035, 0.035, 0.075)),
                    mat.add(StandardMaterial {
                        base_color: Color::srgb(0.85, 0.1, 0.1), // Red shotgun hull
                        metallic: 0.5,
                        perceptual_roughness: 0.3,
                        ..default()
                    }),
                )
            } else {
                (
                    m.add(Cuboid::new(0.02, 0.02, 0.05)),
                    mat.add(StandardMaterial {
                        base_color: Color::srgb(0.95, 0.8, 0.25), // Golden brass casing
                        metallic: 0.85,
                        perceptual_roughness: 0.2,
                        ..default()
                    }),
                )
            };
            ent.insert((mesh, material));
        }
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
            commands.safe_despawn_recursive(entity);
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
            commands.safe_despawn_recursive(entity);
        }
    }
}
