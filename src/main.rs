mod grp;
mod art;
mod palette;
mod map;
mod kwv;
mod builder;
mod sky;
mod animation;
mod scripting;
mod interactivity;
pub mod player;
pub mod combat;
pub mod audio;
pub mod hud;
pub mod game_flow;

pub type Player = player::PlayerController;

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
use bevy_rapier3d::prelude::*;
use builder::MapMeshBuilder;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Input,
    Movement,
    Combat,
    Interactivity,
    Animation,
    RenderSync,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<animation::EngineClock>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Duke Nukem 3D: Build Map Render".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(interactivity::InteractivityPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(combat::CombatPlugin)
        .add_plugins(audio::DukeAudioPlugin)
        .add_plugins(hud::DukeHudPlugin)
        .add_plugins(game_flow::GameFlowPlugin)
        .insert_resource(DukeSounds::default())
        .configure_sets(Update, (
            GameSet::Input,
            GameSet::Movement,
            GameSet::Combat,
            GameSet::Interactivity,
            GameSet::Animation,
            GameSet::RenderSync,
        ).chain())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            (player_look, cursor_grab, emit_player_interaction).in_set(GameSet::Input),
            (play_random_sound, update_weapon).in_set(GameSet::Combat),
            (animation::update_engine_clock, animation::update_tile_animations).in_set(GameSet::Animation),
            (update_billboards, sky::update_skybox).in_set(GameSet::RenderSync),
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
    let mut tile_sizes = std::collections::HashMap::new();
    let mut picanm_map = std::collections::HashMap::new();

    println!("Attempting to load assets from {}", grp_path);
    if let Ok(grp) = Grp::open(grp_path) {
        if let Ok(pal_data) = grp.read_file("PALETTE.DAT") {
            if let Ok(mut pal) = Palette::from_bytes(&pal_data) {
                if let Ok(lookup_data) = grp.read_file("LOOKUP.DAT") {
                    let _ = pal.load_lookups(&lookup_data);
                    println!("Successfully loaded LOOKUP.DAT ({} remappings)", pal.lookups.len());
                }

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
                                            tile_sizes.insert(tile_idx as i16, (w, h));
                                            if let Some(picanm) = art.get_picanm(tile_idx) {
                                                picanm_map.insert(tile_idx as i16, picanm);
                                            }
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
    
    let spark_tile = 2595;
    let spark_material = if let Some(handle) = tile_textures.get(&spark_tile) {
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

    let spark_mesh = meshes.add(Rectangle::new(0.3, 0.3));

    commands.insert_resource(GameAssets {
        tile_textures: tile_textures.clone(),
        default_material: default_material.clone(),
        spark_material,
        spark_mesh,
    });

    let mut start_pos = Vec3::new(0.0, 1.5, 5.0);
    let mut start_yaw = 0.0;

    let map_name = "E1L1.MAP";
    println!("Attempting to load map {} from GRP", map_name);
    
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

                // Build map geometry with Phase 1 & 2 portal compiler, slope tessellation, shading and animations
                let mesh_builder = MapMeshBuilder::new(
                    &map,
                    &tile_textures,
                    &tile_sizes,
                    &picanm_map,
                    default_material.clone(),
                );
                mesh_builder.build(&mut commands, &mut meshes, &mut materials);

                // Spawn Phase 4 interactive sector effectors, switches, touchplates, and props
                interactivity::spawn_interactive_elements_from_map(&mut commands, &map);

                // Check for parallax sky
                let has_sky = map.sectors.iter().any(|s| s.is_ceiling_parallax());
                if has_sky {
                    // Tile 80 is MOONSKY1 (Episode 1 Hollywood Holocaust sky)
                    sky::spawn_skybox(&mut commands, &mut meshes, &mut materials, &tile_textures, 80);
                }
            }
        }
    }

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
        player::PlayerController {
            yaw: start_yaw,
            ..default()
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
pub struct Destructible {
    pub health: i32,
    pub _picnum: i16,
}

#[derive(Component)]
pub struct SpriteBillboard;

fn update_billboards(
    mut query: Query<&mut Transform, With<SpriteBillboard>>,
    camera_query: Query<&Transform, (With<Camera>, Without<SpriteBillboard>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut transform in query.iter_mut() {
            let mut target = camera_transform.translation;
            target.y = transform.translation.y; 
            if target.xz().distance_squared(transform.translation.xz()) > 0.001 {
                transform.look_at(target, Vec3::Y);
            }
        }
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

fn emit_player_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    mut interact_events: EventWriter<interactivity::InteractEvent>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        if let Ok(player_trans) = player_query.get_single() {
            let player_dir = if let Ok(cam_trans) = camera_query.get_single() {
                cam_trans.forward().into()
            } else {
                Vec3::NEG_Z
            };

            interact_events.send(interactivity::InteractEvent {
                player_pos: player_trans.translation,
                player_dir,
            });
        }
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub tile_textures: std::collections::HashMap<i16, Handle<Image>>,
    pub default_material: Handle<StandardMaterial>,
    pub spark_material: Handle<StandardMaterial>,
    pub spark_mesh: Handle<Mesh>,
}

fn update_weapon(
    time: Res<Time>,
    mut query: Query<(&mut Style, &mut FirstPersonWeapon)>,
    player_query: Query<&KinematicCharacterControllerOutput, With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<FirstPersonWeapon>)>,
    mut destructibles: Query<&mut Destructible>,
    barrels: Query<&Transform, With<interactivity::ExplodingBarrel>>,
    mut explosion_events: EventWriter<interactivity::ExplosionDamageEvent>,
    btn: Res<ButtonInput<MouseButton>>,
    sounds: Res<DukeSounds>,
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    assets: Res<GameAssets>,
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
                        if let Ok(barrel_trans) = barrels.get(entity) {
                            explosion_events.send(interactivity::ExplosionDamageEvent {
                                origin: barrel_trans.translation,
                                radius: 6.0,
                                damage: 100,
                            });
                        }
                        commands.entity(entity).despawn_recursive();
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

                // Spawn a bullet hole decal (SHOTSPARK1 is tile 2595) using pre-cached material
                let spark_mat = assets.spark_material.clone();

                // Move slightly towards the camera to prevent z-fighting
                let decal_pos = hit_point - ray_dir * 0.05;

                commands.spawn((
                    PbrBundle {
                        mesh: assets.spark_mesh.clone(),
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
