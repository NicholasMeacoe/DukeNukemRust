#![allow(dead_code)]

use bevy::prelude::*;
use crate::builder::MapMeshBuilder;
use crate::game_flow::state::*;
use crate::grp::Grp;
use crate::map::Map;
use crate::player::PlayerController;

pub fn handle_load_level_events(
    mut events: EventReader<LoadLevelEvent>,
    mut commands: Commands,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut player_query: Query<(&mut Transform, &mut PlayerController), With<crate::Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Option<Res<crate::GameAssets>>,
    mut progress: ResMut<LevelProgress>,
    mut sound_events: EventWriter<crate::audio::PlayNamedSoundEvent>,
) {
    let Some(ref game_assets) = assets else { return; };

    for event in events.read() {
        let map_name = format!("E{}L{}.MAP", event.episode, event.level);
        println!("Loading level {} (Episode {}, Level {})...", map_name, event.episode, event.level);

        // 1. Teardown all previous level entities
        let mut despawned_count = 0;
        for entity in level_entities.iter() {
            commands.entity(entity).despawn_recursive();
            despawned_count += 1;
        }
        println!("Despawned {} previous level entities.", despawned_count);

        // 2. Open GRP and read target map
        if let Ok(grp) = Grp::open(&game_assets.grp_path) {
            if let Ok(map_data) = grp.read_file(&map_name) {
                if let Ok(map) = Map::from_bytes(&map_data) {
                    println!("Map {} loaded successfully: {} sectors, {} walls, {} sprites",
                        map_name, map.sectors.len(), map.walls.len(), map.sprites.len());

                    // 3. Compute spawn position & orientation
                    let floor_y = if (map.cursectnum as usize) < map.sectors.len() {
                        map.sectors[map.cursectnum as usize].get_floor_y_at(&map.walls, map.posx, map.posy)
                    } else {
                        -(map.posz as f32) / (1024.0 * 16.0) - 0.85
                    };

                    let start_pos = Vec3::new(
                        map.posx as f32 / 1024.0,
                        floor_y + 0.85,
                        map.posy as f32 / 1024.0,
                    );
                    let start_yaw = -(map.ang as f32 / 2048.0) * std::f32::consts::TAU + std::f32::consts::FRAC_PI_2;

                    // 4. Reposition player
                    if let Ok((mut p_trans, mut p_ctrl)) = player_query.get_single_mut() {
                        p_trans.translation = start_pos;
                        p_ctrl.spawn_position = start_pos;
                        p_ctrl.yaw = start_yaw;
                        p_ctrl.pitch = 0.0;
                        println!("Player repositioned to: {:?}", p_trans.translation);
                    }

                    // 5. Update level statistics (monsters & secrets)
                    let monster_count = map.sprites.iter()
                        .filter(|s| matches!(s.picnum, 2000 | 1680 | 1820 | 2120))
                        .count() as i32;
                    let secret_count = map.sectors.iter()
                        .filter(|s| s.lotag == 32767)
                        .count() as i32
                        + map.sprites.iter()
                            .filter(|s| s.lotag != 0 && matches!(s.picnum, 142..=145))
                            .count() as i32;

                    progress.current_episode = event.episode;
                    progress.current_level = event.level;
                    progress.total_monsters = monster_count.max(1);
                    progress.total_secrets = secret_count.max(1);
                    progress.kills_count = 0;
                    progress.secrets_found = 0;
                    progress.level_time_seconds = 0.0;
                    progress.is_level_completed = false;

                    // 6. Build map geometry & colliders
                    let mesh_builder = MapMeshBuilder::new(
                        &map,
                        &game_assets.tile_textures,
                        &game_assets.tile_sizes,
                        &game_assets.picanm_map,
                        game_assets.default_material.clone(),
                    );
                    mesh_builder.build(&mut commands, &mut meshes, &mut materials);

                    // 7. Spawn interactive effectors, props, and enemies
                    crate::interactivity::spawn_interactive_elements_from_map(&mut commands, &map);

                    // 8. Spawn parallax skybox if needed
                    let has_sky = map.sectors.iter().any(|s| s.is_ceiling_parallax());
                    if has_sky {
                        let sky_tile = match event.episode {
                            1 => 80,  // MOONSKY1
                            2 => 84,  // BIGORBIT1
                            _ => 89,  // LA_SKY
                        };
                        crate::sky::spawn_skybox(&mut commands, &mut meshes, &mut materials, &game_assets.tile_textures, sky_tile);
                    }

                    // 9. Play authentic level music track
                    let track = crate::audio::LevelMidiTrack::for_level(event.episode, event.level);
                    sound_events.send(crate::audio::PlayNamedSoundEvent {
                        name: track.filename().to_string(),
                        volume: 0.8,
                        position: None,
                    });
                }
            } else {
                eprintln!("Failed to read {} from GRP!", map_name);
            }
        } else {
            eprintln!("Failed to open GRP from path {}", game_assets.grp_path);
        }
    }
}
