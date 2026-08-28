mod grp;
mod art;
mod palette;
mod map;
mod kwv;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::input::mouse::MouseMotion;
use grp::Grp;
use art::Art;
use palette::Palette;
use map::Map;
use kwv::Kwv;
use std::fs;
use bevy_rapier3d::prelude::*;
use lyon_tessellation::math::{point};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Duke Nukem 3D: Build Map Render".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(DukeSounds::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_move, 
            player_look, 
            cursor_grab, 
            update_billboards, 
            play_random_sound,
            update_weapon
        ))
        .run();
}

#[derive(Resource, Default)]
struct DukeSounds {
    handles: Vec<Handle<AudioSource>>,
}

fn play_random_sound(
    keys: Res<ButtonInput<KeyCode>>,
    sounds: Res<DukeSounds>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        if !sounds.handles.is_empty() {
            let idx = rand::random::<usize>() % sounds.handles.len();
            if let Some(handle) = sounds.handles.get(idx) {
                println!("Playing random Duke sound index: {}", idx);
                commands.spawn(AudioBundle {
                    source: handle.clone(),
                    ..default()
                });
            }
        }
    }
}

#[derive(Component)]
struct Player {
    speed: f32,
    pitch: f32,
    yaw: f32,
    velocity_y: f32,
}

#[derive(Component)]
struct FirstPersonWeapon {
    fire_timer: f32,
    base_y: f32,
    bob_timer: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut duke_sounds: ResMut<DukeSounds>,
) {
    let grp_path = "dukenukem3d/duke3d.grp";
    let mut tile_textures = std::collections::HashMap::new();

    println!("Attempting to load assets from {}", grp_path);
    if let Ok(grp) = Grp::open(grp_path) {
        if let Ok(pal_data) = grp.read_file("PALETTE.DAT") {
            if let Ok(pal) = Palette::from_bytes(&pal_data) {
                let mut art_files_found = 0;
                for entry in &grp.entries {
                    if entry.name.to_uppercase().starts_with("TILES") && entry.name.to_uppercase().ends_with(".ART") {
                        art_files_found += 1;
                        if let Ok(art_data) = grp.read_file(&entry.name) {
                            if let Ok(art) = Art::from_bytes(&art_data) {
                                println!("Successfully loaded {} (tiles {} to {})", entry.name, art.local_tile_start, art.local_tile_end);
                                for tile_idx in art.local_tile_start..=art.local_tile_end {
                                    if let Some((w, h, rgba)) = art.get_tile_rgba(tile_idx, &pal.colors) {
                                        if w > 0 && h > 0 {
                                            let mut image = Image::new_fill(
                                                bevy::render::render_resource::Extent3d {
                                                    width: w,
                                                    height: h,
                                                    depth_or_array_layers: 1,
                                                },
                                                bevy::render::render_resource::TextureDimension::D2,
                                                &rgba,
                                                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                                RenderAssetUsages::default(),
                                            );
                                            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
                                            tile_textures.insert(tile_idx as i16, images.add(image));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                println!("Total ART files found in GRP: {}", art_files_found);
            }
        }

        if let Ok(kwv_data) = grp.read_file("WAVES.KWV") {
            if let Ok(kwv) = Kwv::from_bytes(&kwv_data) {
                println!("Loaded {} sounds from WAVES.KWV", kwv.waves.len());
                for wave in kwv.waves {
                    if wave.data.is_empty() { continue; }
                    let wav_bytes = wave.to_wav_bytes();
                    let source = AudioSource {
                        bytes: wav_bytes.into(),
                    };
                    duke_sounds.handles.push(audio_sources.add(source));
                }
            }
        }
    }

    let default_material = materials.add(Color::srgb(0.5, 0.5, 0.6));
    
    commands.insert_resource(GameAssets {
        tile_textures: tile_textures.clone(),
        default_material: default_material.clone(),
    });

    let mut start_pos = Vec3::new(0.0, 1.5, 5.0);
    let mut start_yaw = 0.0;

    let map_name = "E1L1.MAP";
    println!("Attempting to load map {} from GRP", map_name);
    
    // We need to re-open or borrow the grp to get the map. 
    // Since we consumed the OK(grp) above, let's just open it again or we can refactor.
    // For simplicity, just open it again.
    if let Ok(grp) = Grp::open(grp_path) {
        if let Ok(map_data) = grp.read_file(map_name) {
            if let Ok(map) = Map::from_bytes(&map_data) {
                println!("Map loaded successfully: {} sectors, {} walls, {} sprites", map.sectors.len(), map.walls.len(), map.sprites.len());
            
            // Build units to meters: 1024 units ~= 1 meter (approx)
            // Build Z units are 16x smaller
            start_pos = Vec3::new(
                map.posx as f32 / 1024.0,
                -(map.posz as f32) / (1024.0 * 16.0),
                map.posy as f32 / 1024.0,
            );
            start_yaw = -(map.ang as f32 / 2048.0) * std::f32::consts::TAU + std::f32::consts::FRAC_PI_2;
            println!("Player start position: {:?}", start_pos);

            let default_material = materials.add(Color::srgb(0.5, 0.5, 0.6));

            for sector in &map.sectors {
                let floor_y = -(sector.floorz as f32) / (1024.0 * 16.0);
                let ceil_y = -(sector.ceilingz as f32) / (1024.0 * 16.0);
                
                let mut loops = Vec::new();
                let mut current_loop = Vec::new();
                let mut visited_walls = std::collections::HashSet::new();

                for i in 0..sector.wallnum {
                    let wall_idx = (sector.wallptr + i) as usize;
                    if visited_walls.contains(&wall_idx) { continue; }

                    let mut w = wall_idx;
                    loop {
                        if visited_walls.contains(&w) { break; }
                        visited_walls.insert(w);
                        let wall = &map.walls[w];
                        current_loop.push(Vec2::new(wall.x as f32 / 1024.0, wall.y as f32 / 1024.0));
                        w = wall.point2 as usize;
                        if w == wall_idx { break; }
                        if w < sector.wallptr as usize || w >= (sector.wallptr + sector.wallnum) as usize {
                            break;
                        }
                    }
                    if !current_loop.is_empty() {
                        loops.push(std::mem::take(&mut current_loop));
                    }
                }

                if !loops.is_empty() {
                    let mut tessellator = FillTessellator::new();
                    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
                    
                    let mut path_builder = lyon_tessellation::path::Path::builder();
                    for poly_points in &loops {
                        if poly_points.len() < 3 { continue; }
                        path_builder.begin(point(poly_points[0].x, poly_points[0].y));
                        for p in poly_points.iter().skip(1) {
                            path_builder.line_to(point(p.x, p.y));
                        }
                        path_builder.end(true);
                    }
                    let path = path_builder.build();

                    if let Ok(_) = tessellator.tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                            [vertex.position().x, vertex.position().y]
                        }),
                    ) {
                        let vertices: Vec<[f32; 3]> = buffers.vertices
                            .iter()
                            .map(|v| [v[0], 0.0, v[1]])
                            .collect();
                        let uvs: Vec<[f32; 2]> = buffers.vertices
                            .iter()
                            // Divide by a larger factor so repeating is less dense
                            .map(|v| [v[0] / 4.0, v[1] / 4.0])
                            .collect();
                        let indices = buffers.indices;
                        
                        let floor_mat = if let Some(handle) = tile_textures.get(&sector.floorpicnum) {
                            materials.add(StandardMaterial {
                                base_color_texture: Some(handle.clone()),
                                unlit: true,
                                ..default()
                            })
                        } else {
                            default_material.clone()
                        };

                        let mut floor_mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
                        floor_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
                        floor_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
                        floor_mesh.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
                        floor_mesh.duplicate_vertices();
                        floor_mesh.compute_flat_normals();

                        let floor_collider_vertices: Vec<Vect> = vertices.iter().map(|v| Vect::new(v[0], v[1], v[2])).collect();
                        let floor_collider_indices: Vec<[u32; 3]> = indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

                        if !floor_collider_indices.is_empty() {
                            commands.spawn((
                                PbrBundle {
                                    mesh: meshes.add(floor_mesh),
                                    material: floor_mat,
                                    transform: Transform::from_xyz(0.0, floor_y, 0.0),
                                    ..default()
                                },
                                RigidBody::Fixed,
                                Collider::trimesh(floor_collider_vertices, floor_collider_indices),
                            ));
                        }

                        let ceil_mat = if let Some(handle) = tile_textures.get(&sector.ceilingpicnum) {
                            materials.add(StandardMaterial {
                                base_color_texture: Some(handle.clone()),
                                unlit: true,
                                ..default()
                            })
                        } else {
                            default_material.clone()
                        };

                        let mut ceil_mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
                        ceil_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
                        ceil_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                        ceil_mesh.insert_indices(bevy::render::mesh::Indices::U32(indices.clone()));
                        ceil_mesh.duplicate_vertices();
                        ceil_mesh.compute_flat_normals();

                        let ceil_collider_vertices: Vec<Vect> = vertices.iter().map(|v| Vect::new(v[0], v[1], v[2])).collect();
                        let ceil_collider_indices: Vec<[u32; 3]> = indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

                        if !ceil_collider_indices.is_empty() {
                            commands.spawn((
                                PbrBundle {
                                    mesh: meshes.add(ceil_mesh),
                                    material: ceil_mat,
                                    transform: Transform::from_xyz(0.0, ceil_y, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                                    ..default()
                                },
                                RigidBody::Fixed,
                                Collider::trimesh(ceil_collider_vertices, ceil_collider_indices),
                            ));
                        }
                    }
                }

                for i in 0..sector.wallnum {
                    let wall_idx = (sector.wallptr + i) as usize;
                    if wall_idx >= map.walls.len() { continue; }
                    let wall = &map.walls[wall_idx];
                    let next_wall_idx = wall.point2 as usize;
                    if next_wall_idx >= map.walls.len() { continue; }
                    let next_wall = &map.walls[next_wall_idx];

                    let p1 = Vec2::new(wall.x as f32 / 1024.0, wall.y as f32 / 1024.0);
                    let p2 = Vec2::new(next_wall.x as f32 / 1024.0, next_wall.y as f32 / 1024.0);

                    let mid_point = (p1 + p2) / 2.0;
                    let diff = p2 - p1;
                    let length = diff.length();
                    let angle = f32::atan2(diff.y, diff.x);
                    let height = (ceil_y - floor_y).abs();

                    if length > 0.01 && height > 0.01 {
                        let material = if let Some(handle) = tile_textures.get(&wall.picnum) {
                            materials.add(StandardMaterial {
                                base_color_texture: Some(handle.clone()),
                                alpha_mode: AlphaMode::Mask(0.5),
                                unlit: true,
                                double_sided: true,
                                ..default()
                            })
                        } else {
                            default_material.clone()
                        };

                        // Fix UVs for the quad so textures tile instead of stretching massively
                        let mut wall_mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
                        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
                            [-length/2.0, -height/2.0, 0.0],
                            [length/2.0, -height/2.0, 0.0],
                            [length/2.0, height/2.0, 0.0],
                            [-length/2.0, height/2.0, 0.0],
                        ]);
                        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                        ]);
                        wall_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
                            [0.0, height / 2.0],
                            [length / 2.0, height / 2.0],
                            [length / 2.0, 0.0],
                            [0.0, 0.0],
                        ]);
                        wall_mesh.insert_indices(bevy::render::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]));


                        commands.spawn((
                            PbrBundle {
                                mesh: meshes.add(wall_mesh),
                                material,
                                transform: Transform::from_xyz(mid_point.x, (ceil_y + floor_y) / 2.0, mid_point.y)
                                    .with_rotation(Quat::from_rotation_y(-angle)),
                                ..default()
                            },
                            RigidBody::Fixed,
                            Collider::cuboid(length / 2.0, height / 2.0, 0.025),
                        ));
                    }
                }
            }

            for sprite in &map.sprites {
                if let Some(handle) = tile_textures.get(&sprite.picnum) {
                    let pos = Vec3::new(
                        sprite.x as f32 / 1024.0,
                        -(sprite.z as f32) / (1024.0 * 16.0),
                        sprite.y as f32 / 1024.0,
                    );
                    
                    let scale_x = (sprite.xrepeat as f32 / 64.0) * 3.0;
                    let scale_y = (sprite.yrepeat as f32 / 64.0) * 3.0;
                    
                    let is_enemy = sprite.picnum == 2000; // PIGCOP

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Rectangle::new(scale_x, scale_y)),
                            material: materials.add(StandardMaterial {
                                base_color_texture: Some(handle.clone()),
                                alpha_mode: AlphaMode::Mask(0.5),
                                unlit: true,
                                double_sided: true,
                                ..default()
                            }),
                            transform: Transform::from_translation(pos),
                            ..default()
                        },
                        SpriteBillboard,
                        RigidBody::Fixed,
                        Collider::cuboid(scale_x / 2.0, scale_y / 2.0, 0.1),
                        Destructible {
                            health: if is_enemy { 100 } else { 10 },
                            _picnum: sprite.picnum,
                        }
                    ));
                }
            }
        }
    }
} // Closes `if let Ok(grp) = Grp::open(grp_path)`

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.0,
            range: 2000.0,
            ..default()
        },
        transform: Transform::from_xyz(start_pos.x, start_pos.y + 20.0, start_pos.z),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.0,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 20000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });

    let player_entity = commands.spawn((
        Player {
            speed: 10.0,
            pitch: 0.0,
            yaw: start_yaw,
            velocity_y: 0.0,
        },
        TransformBundle::from_transform(Transform::from_translation(start_pos + Vec3::Y * 0.5)),
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.5, 0.3),
        LockedAxes::ROTATION_LOCKED, // Prevent the player from tipping over
        KinematicCharacterController {
            up: Vec3::Y,
            offset: CharacterLength::Relative(0.01),
            ..default()
        },
    )).id();

    let _camera_entity = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.4, 0.0).with_rotation(Quat::from_rotation_y(start_yaw)),
        ..default()
    }).set_parent(player_entity).id();

    // Spawn First Person Weapon (Pistol) using UI
    // The shareware version might not have 2524, let's try 2524 (FIRSTGUN) or fallback to something else, or a colored block
    let pistol_tile = 2524;
    let weapon_image = if let Some(handle) = tile_textures.get(&pistol_tile) {
        handle.clone()
    } else {
        // Fallback to shotgun or just something visible if pistol is missing in this GRP
        tile_textures.get(&2613).cloned().unwrap_or_else(|| {
             images.add(Image::default()) // dummy
        })
    };

    let base_y = 0.0; // Rest position at bottom of screen
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd, // Align to bottom
                ..default()
            },
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            ImageBundle {
                image: UiImage::new(weapon_image),
                style: Style {
                    width: Val::Px(400.0),
                    height: Val::Px(400.0),
                    margin: UiRect::bottom(Val::Px(base_y)), // Offset from bottom
                    ..default()
                },
                // Add background color white to ensure image is visible, even if texture has alpha issues
                background_color: Color::WHITE.into(),
                ..default()
            },
            FirstPersonWeapon {
                fire_timer: 0.0,
                base_y,
                bob_timer: 0.0,
            },
        ));
    });
}

#[derive(Component)]
struct Destructible {
    health: i32,
    _picnum: i16,
}

#[derive(Component)]
struct SpriteBillboard;

fn update_billboards(
    mut query: Query<&mut Transform, With<SpriteBillboard>>,
    camera_query: Query<&Transform, (With<Camera>, Without<SpriteBillboard>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut transform in query.iter_mut() {
            let mut target = camera_transform.translation;
            target.y = transform.translation.y; 
            transform.look_at(target, Vec3::Y);
        }
    }
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut KinematicCharacterController)>,
    camera_query: Query<&Transform, (With<Camera>, Without<FirstPersonWeapon>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return; };

    for (mut player, mut controller) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        // Use Bevy's built-in directions, extracting the yaw rotation from the camera
        let forward = camera_transform.forward().with_y(0.0).normalize_or_zero();
        let right = camera_transform.right().with_y(0.0).normalize_or_zero();

        if keys.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= right; 
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += right;
        }

        let mut movement = Vec3::ZERO;
        if direction != Vec3::ZERO {
            movement += direction.normalize() * player.speed * time.delta_seconds();
        }
        
        if keys.just_pressed(KeyCode::Space) {
            player.velocity_y = 5.0;
        }

        player.velocity_y -= 15.0 * time.delta_seconds();
        movement.y += player.velocity_y * time.delta_seconds();

        controller.translation = Some(movement);
    }
}

fn player_look(
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<&mut Player>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<FirstPersonWeapon>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let Ok(mut camera_transform) = camera_query.get_single_mut() else { return; };

    for mut player in query.iter_mut() {
        player.yaw -= delta.x * 0.002;
        player.pitch -= delta.y * 0.002;
        player.pitch = player.pitch.clamp(-1.54, 1.54);

        camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, player.yaw)
            * Quat::from_axis_angle(Vec3::X, player.pitch);
    }
}

#[derive(Resource)]
struct GameAssets {
    tile_textures: std::collections::HashMap<i16, Handle<Image>>,
    default_material: Handle<StandardMaterial>,
}

fn update_weapon(
    time: Res<Time>,
    mut query: Query<(&mut Style, &mut FirstPersonWeapon)>,
    player_query: Query<&KinematicCharacterControllerOutput, With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<FirstPersonWeapon>)>,
    mut destructibles: Query<&mut Destructible>,
    btn: Res<ButtonInput<MouseButton>>,
    sounds: Res<DukeSounds>,
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let is_moving = if let Ok(output) = player_query.get_single() {
        output.effective_translation.xz().length_squared() > 0.001
    } else {
        false
    };

    let Ok(camera_transform) = camera_query.get_single() else { return; };

    for (mut style, mut weapon) in query.iter_mut() {
        // Simple View Bobbing
        if is_moving {
            weapon.bob_timer += time.delta_seconds() * 10.0;
        } else {
            weapon.bob_timer = weapon.bob_timer.lerp(0.0, time.delta_seconds() * 5.0);
            if weapon.bob_timer < 0.1 { weapon.bob_timer = 0.0; }
        }

        let bob_offset = (weapon.bob_timer.sin() * 20.0).abs() * -1.0;
        
        // Shooting
        if weapon.fire_timer > 0.0 {
            weapon.fire_timer -= time.delta_seconds();
        }

        let mut recoil_offset = 0.0;

        if btn.just_pressed(MouseButton::Left) && weapon.fire_timer <= 0.0 {
            // "Fire" recoil
            weapon.fire_timer = 0.5; 

            // Play firing sound
            if !sounds.handles.is_empty() {
                // The pistol sound index depends on the KWV file structure. We will just play index 5 for now.
                let fire_sound_idx = 5.min(sounds.handles.len() - 1);
                if let Some(handle) = sounds.handles.get(fire_sound_idx) {
                    println!("Playing fire sound at index: {}", fire_sound_idx);
                    commands.spawn(AudioBundle {
                        source: handle.clone(),
                        ..default()
                    });
                }
            }

            // Hitscan Logic
            let ray_pos = camera_transform.translation;
            // The camera looks down its negative Z axis
            let ray_dir = camera_transform.forward(); 
            let max_toi = 100.0;
            let solid = true;
            let filter = QueryFilter::exclude_kinematic();

            if let Some((entity, toi)) = rapier_context.cast_ray(
                ray_pos, *ray_dir, max_toi, solid, filter
            ) {
                let hit_point = ray_pos + ray_dir * toi;
                
                // Play ricochet sound (1 = PISTOL_RICOCHET) by default
                let mut hit_sound_idx = 1;

                if let Ok(mut destructible) = destructibles.get_mut(entity) {
                    destructible.health -= 6; // PISTOL_WEAPON_STRENGTH
                    
                    // 2 = PISTOL_BODYHIT
                    hit_sound_idx = 2; 

                    if destructible.health <= 0 {
                        commands.entity(entity).despawn_recursive();
                        // Optional: play death sound or spawn gibs here
                    }
                }

                if let Some(handle) = sounds.handles.get(hit_sound_idx) {
                    commands.spawn((
                        AudioBundle {
                            source: handle.clone(),
                            ..default()
                        },
                        TransformBundle::from_transform(Transform::from_translation(hit_point)),
                    ));
                }

                // Spawn a bullet hole decal (SHOTSPARK1 is tile 2595)
                let spark_tile = 2595;
                let spark_mat = if let Some(handle) = assets.tile_textures.get(&spark_tile) {
                    materials.add(StandardMaterial {
                        base_color_texture: Some(handle.clone()),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: true,
                        double_sided: true,
                        ..default()
                    })
                } else {
                    assets.default_material.clone()
                };

                // Move slightly towards the camera to prevent z-fighting
                let decal_pos = hit_point - ray_dir * 0.05;

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Rectangle::new(0.3, 0.3)),
                        material: spark_mat,
                        transform: Transform::from_translation(decal_pos),
                        ..default()
                    },
                    SpriteBillboard,
                ));
            }
        }

        if weapon.fire_timer > 0.4 {
            recoil_offset = -50.0; // Recoil push down
        }

        style.margin.bottom = Val::Px(weapon.base_y + bob_offset + recoil_offset);
    }
}

fn cursor_grab(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return; };

    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
