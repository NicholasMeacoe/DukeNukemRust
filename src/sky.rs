use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct SkyboxDome;

/// Calculate authentic Build engine 360-degree cylindrical panorama repeat count.
/// Duke 3D skies wrap across 1024 Build units (e.g. 256px texture -> 4 wraps).
pub fn calculate_sky_tile_repeats(tile_width: u32) -> f32 {
    if tile_width == 0 {
        4.0
    } else {
        (1024.0 / tile_width as f32).clamp(1.0, 16.0)
    }
}

/// Returns the authentic Duke Nukem 3D sky tile index for a given episode:
/// - Episode 1 ("L.A. Meltdown"): LA_SKY (#89)
/// - Episode 2 ("Lunar Apocalypse"): MOONSKY1 (#80)
/// - Episode 3 ("Shrapnel City"): CITY_SKY / BIGORBIT1 (#84)
/// - Episode 4 ("The Birth" / Plutonium Pak): REDSKY1 (#98)
/// - Default: LA_SKY (#89)
pub fn sky_tile_for_episode(episode: usize) -> i16 {
    match episode {
        1 => crate::names::LA_SKY,
        2 => crate::names::MOONSKY1,
        3 => crate::names::CITY_SKY,
        4 => crate::names::REDSKY1,
        _ => crate::names::LA_SKY,
    }
}

pub fn spawn_skybox(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tile_textures: &HashMap<i16, Handle<Image>>,
    sky_tile: i16,
) {
    let texture = tile_textures.get(&sky_tile).cloned();
    let sky_mat = materials.add(StandardMaterial {
        base_color_texture: texture,
        unlit: true,
        cull_mode: None, // Render inside of dome/cylinder
        double_sided: true,
        fog_enabled: false,
        ..default()
    });

    // Spawn a large inverted cylinder for authentic panoramic sky
    let mut cylinder_mesh = Mesh::from(Cylinder {
        radius: 500.0,
        half_height: 250.0,
    });

    let repeats = calculate_sky_tile_repeats(256);

    // Scale UVs so the sky texture tiles correctly around the panorama and invert U for inner visibility
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x2(uvs)) =
        cylinder_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        for uv in uvs.iter_mut() {
            uv[0] = (1.0 - uv[0]) * repeats;
        }
    }

    cylinder_mesh.duplicate_vertices();
    cylinder_mesh.compute_flat_normals();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(cylinder_mesh),
            material: sky_mat,
            transform: Transform::from_xyz(0.0, 50.0, 0.0),
            ..default()
        },
        SkyboxDome,
        crate::game_flow::LevelEntity,
    ));
}

pub fn update_skybox(
    camera_query: Query<&Transform, (With<Camera>, Without<SkyboxDome>, Changed<Transform>)>,
    mut sky_query: Query<&mut Transform, With<SkyboxDome>>,
) {
    if let Ok(cam_trans) = camera_query.get_single() {
        for mut sky_trans in sky_query.iter_mut() {
            sky_trans.translation.x = cam_trans.translation.x;
            sky_trans.translation.z = cam_trans.translation.z;
            let (yaw, pitch, _) = cam_trans.rotation.to_euler(EulerRot::YXZ);
            let clamped_pitch = pitch.clamp(-0.85, 0.85);
            sky_trans.translation.y = cam_trans.translation.y + clamped_pitch * 25.0;
            sky_trans.rotation = Quat::from_rotation_y(yaw * 0.5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sky_tile_repeats_calculation() {
        assert_eq!(calculate_sky_tile_repeats(256), 4.0);
        assert_eq!(calculate_sky_tile_repeats(128), 8.0);
        assert_eq!(calculate_sky_tile_repeats(512), 2.0);
        assert_eq!(calculate_sky_tile_repeats(1024), 1.0);
        assert_eq!(calculate_sky_tile_repeats(0), 4.0);
    }

    #[test]
    fn test_skybox_camera_alignment_and_pitch() {
        let mut app = App::new();
        app.add_systems(Update, update_skybox);

        let cam_pos = Vec3::new(15.0, 2.5, -30.0);
        let _cam_entity = app
            .world_mut()
            .spawn(Camera3dBundle {
                transform: Transform::from_translation(cam_pos)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ..default()
            })
            .id();

        let sky_entity = app
            .world_mut()
            .spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
                SkyboxDome,
            ))
            .id();

        app.update();

        let sky_trans = app.world().entity(sky_entity).get::<Transform>().unwrap();
        assert_eq!(sky_trans.translation.x, cam_pos.x);
        assert_eq!(sky_trans.translation.z, cam_pos.z);
        // Half-speed yaw rotation: PI/4
        let (yaw, _, _) = sky_trans.rotation.to_euler(EulerRot::YXZ);
        assert!((yaw - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
    }

    #[test]
    fn test_sky_tile_for_episode() {
        assert_eq!(sky_tile_for_episode(1), crate::names::LA_SKY);
        assert_eq!(sky_tile_for_episode(2), crate::names::MOONSKY1);
        assert_eq!(sky_tile_for_episode(3), crate::names::CITY_SKY);
        assert_eq!(sky_tile_for_episode(4), crate::names::REDSKY1);
        assert_eq!(sky_tile_for_episode(99), crate::names::LA_SKY);
    }
}
