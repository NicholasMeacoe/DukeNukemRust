use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct SkyboxDome;

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
        ..default()
    });

    // Spawn a large inverted cylinder / dome for panoramic sky
    let mut cylinder_mesh = Mesh::from(Cylinder {
        radius: 500.0,
        half_height: 250.0,
    });
    
    // Invert U coordinate so textures are not mirrored when viewed from the inside
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x2(uvs)) = cylinder_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs.iter_mut() {
            uv[0] = 1.0 - uv[0];
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
    ));
}

pub fn update_skybox(
    camera_query: Query<&Transform, (With<Camera>, Without<SkyboxDome>)>,
    mut sky_query: Query<&mut Transform, With<SkyboxDome>>,
) {
    if let Ok(cam_trans) = camera_query.get_single() {
        for mut sky_trans in sky_query.iter_mut() {
            sky_trans.translation.x = cam_trans.translation.x;
            sky_trans.translation.z = cam_trans.translation.z;
        }
    }
}
