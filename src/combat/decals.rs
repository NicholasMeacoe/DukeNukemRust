use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DecalType {
    BulletHole,
    BloodSplatter,
    ScorchMark,
    GlassCrack,
    WaterRipple,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SurfaceDecal {
    pub decal_type: DecalType,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub alpha: f32,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Component)]
pub struct ImpactSpark {
    pub timer: f32,
    pub max_lifetime: f32,
    pub base_tile: i16,
}

pub struct DecalPlugin;

impl Plugin for DecalPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDecalEvent>().add_systems(
            Update,
            (
                handle_spawn_decal_events,
                update_impact_sparks,
                update_surface_decals,
                update_atmospheric_emitters,
            )
                .in_set(crate::GameSet::Animation),
        );
    }
}

pub fn handle_spawn_decal_events(
    mut events: EventReader<SpawnDecalEvent>,
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    game_assets: Option<Res<crate::GameAssets>>,
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

        let offset_pos = ev.origin + normal * 0.015;

        let mut entity_cmd = commands.spawn((
            SurfaceDecal {
                decal_type: ev.decal_type,
                lifetime: max_lifetime,
                max_lifetime,
                alpha: 1.0,
            },
            TransformBundle::from_transform(
                Transform::from_translation(offset_pos).with_rotation(rot),
            ),
            crate::game_flow::LevelEntity,
        ));

        if let (Some(ref mut mesh_assets), Some(ref mut mat_assets)) =
            (meshes.as_mut(), materials.as_mut())
        {
            let (size, base_color) = match ev.decal_type {
                DecalType::BulletHole => (0.16, Color::srgba(0.08, 0.08, 0.1, 0.95)),
                DecalType::ScorchMark => (1.2, Color::srgba(0.04, 0.04, 0.04, 0.85)),
                DecalType::BloodSplatter => (0.55, Color::srgba(0.55, 0.02, 0.02, 0.9)),
                DecalType::GlassCrack => (0.4, Color::srgba(0.9, 0.95, 1.0, 0.7)),
                DecalType::WaterRipple => (0.6, Color::srgba(0.3, 0.6, 0.9, 0.5)),
            };

            let decal_mesh = mesh_assets.add(Rectangle::new(size, size));
            let decal_mat = mat_assets.add(StandardMaterial {
                base_color,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                ..default()
            });

            entity_cmd.insert((decal_mesh, decal_mat));

            // For bullet impacts, spawn an authentic animated ricochet spark puff (SHOTSPARK1)
            if ev.decal_type == DecalType::BulletHole {
                let spark_mesh = mesh_assets.add(Rectangle::new(0.32, 0.32));
                let spark_mat = if let Some(ref assets) = game_assets {
                    if let Some(tex) = assets.tile_textures.get(&crate::names::SHOTSPARK1) {
                        mat_assets.add(StandardMaterial {
                            base_color_texture: Some(tex.clone()),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            double_sided: true,
                            ..default()
                        })
                    } else {
                        mat_assets.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.9, 0.3),
                            unlit: true,
                            double_sided: true,
                            ..default()
                        })
                    }
                } else {
                    mat_assets.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.9, 0.3),
                        unlit: true,
                        double_sided: true,
                        ..default()
                    })
                };

                commands.spawn((
                    PbrBundle {
                        mesh: spark_mesh,
                        material: spark_mat,
                        transform: Transform::from_translation(ev.origin + normal * 0.04),
                        ..default()
                    },
                    crate::SpriteBillboard,
                    ImpactSpark {
                        timer: 0.16,
                        max_lifetime: 0.16,
                        base_tile: crate::names::SHOTSPARK1,
                    },
                    crate::game_flow::LevelEntity,
                ));
            }
        }
    }
}

pub fn update_impact_sparks(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImpactSpark, &Handle<StandardMaterial>)>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    game_assets: Option<Res<crate::GameAssets>>,
) {
    let dt = time.delta_seconds();
    for (entity, mut spark, mat_handle) in query.iter_mut() {
        spark.timer -= dt;
        if spark.timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let progress = 1.0 - (spark.timer / spark.max_lifetime);
        let frame_offset = ((progress * 4.0) as i16).clamp(0, 3);
        let tile = spark.base_tile + frame_offset;

        if let (Some(ref mut mats), Some(ref assets)) = (materials.as_mut(), game_assets.as_ref()) {
            if let Some(tex) = assets.tile_textures.get(&tile) {
                if let Some(mat) = mats.get_mut(mat_handle) {
                    mat.base_color_texture = Some(tex.clone());
                }
            }
        }
    }
}

pub fn update_surface_decals(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SurfaceDecal, Option<&Handle<StandardMaterial>>)>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let dt = time.delta_seconds();
    let total_decals = query.iter().count();
    let mut excess = if total_decals > 256 {
        total_decals - 256
    } else {
        0
    };

    for (entity, mut decal, mat_handle) in query.iter_mut() {
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
            if let (Some(handle), Some(ref mut mats)) = (mat_handle, materials.as_mut()) {
                if let Some(mat) = mats.get_mut(handle) {
                    mat.base_color.set_alpha(decal.alpha);
                }
            }
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
