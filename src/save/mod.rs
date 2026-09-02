#![allow(dead_code)]

pub mod format;
pub mod snapshot;

pub use format::*;
pub use snapshot::*;

use crate::game_flow::state::*;
use crate::player::types::PlayerController;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct SaveLoadPlugin;

#[derive(Event, Debug, Clone)]
pub struct SaveGameEvent {
    pub slot: Option<usize>, // None = quicksave
    pub title: String,
}

#[derive(Event, Debug, Clone)]
pub struct LoadGameEvent {
    pub slot: Option<usize>, // None = quicksave
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SaveManager {
    pub last_saved_slot: Option<usize>,
    pub save_slots_info: [Option<String>; 10],
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveManager>()
            .add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_systems(
                Update,
                (
                    handle_save_load_hotkeys,
                    handle_save_events,
                    handle_load_events,
                    auto_tag_saveable,
                    hydrate_enemies,
                    hydrate_items,
                    hydrate_switches_and_props,
                    hydrate_projectiles,
                    hydrate_effectors,
                ),
            )
            .add_systems(
                Update,
                apply_pending_save.after(crate::game_flow::level_loader::handle_load_level_events),
            );
    }
}

pub fn handle_save_load_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
) {
    if *state.get() != GamePhase::Playing {
        return;
    }

    // F6: Quicksave
    if keys.just_pressed(KeyCode::F6) {
        println!("F6 Quicksave triggered!");
        save_events.send(SaveGameEvent {
            slot: None,
            title: "Quicksave".to_string(),
        });
    }

    // F9: Quickload
    if keys.just_pressed(KeyCode::F9) {
        println!("F9 Quickload triggered!");
        load_events.send(LoadGameEvent { slot: None });
    }
}

#[derive(Component, Default)]
pub struct Saveable;

#[derive(Component, Default)]
pub struct RequiresHydration;

pub fn handle_save_events(
    mut events: EventReader<SaveGameEvent>,
    player_query: Query<(&Transform, &PlayerController), With<crate::Player>>,
    q_group1: (
        Query<
            (
                &Transform,
                &crate::combat::types::EnemyActor,
                &crate::scripting::ConActor,
            ),
            With<Saveable>,
        >,
        Query<(&Transform, &crate::interactivity::types::ItemPickup), With<Saveable>>,
        Query<(&Transform, &crate::combat::types::Projectile), With<Saveable>>,
        Query<
            (
                &Transform,
                &crate::interactivity::types::SectorEffectorComponent,
            ),
            With<Saveable>,
        >,
        Query<(&Transform, &crate::interactivity::types::InteractiveSwitch), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::CrackWall), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::BreakableGlass), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::ExplodingBarrel), With<Saveable>>,
    ),
    q_group2: (
        Query<(&Transform, &crate::interactivity::types::WaterFountain), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::ToiletProp), With<Saveable>>,
        Query<
            (
                &Transform,
                &crate::interactivity::props_extended::DancerProp,
            ),
            With<Saveable>,
        >,
        Query<(&Transform, &crate::interactivity::types::ViewscreenProp), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::SecurityCamera), With<Saveable>>,
        Query<
            (
                &Transform,
                &crate::interactivity::props_extended::SecurityCameraMonitor,
            ),
            With<Saveable>,
        >,
        Query<(&Transform, &crate::interactivity::types::NukeExitSwitch), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::MasterSwitch), With<Saveable>>,
    ),
    progress: Res<LevelProgress>,
    mut save_mgr: ResMut<SaveManager>,
) {
    let Ok((trans, player)) = player_query.get_single() else {
        return;
    };

    for event in events.read() {
        let mut snapshot = SaveGameSnapshot::new(
            &event.title,
            progress.current_episode as u8,
            progress.current_level as u8,
            progress.skill as u8,
            progress.kills_count,
            progress.secrets_found,
            progress.level_time_seconds,
        );

        snapshot.player = Some(((*trans).into(), player.clone()));

        for (t, e, c) in &q_group1.0 {
            snapshot.enemies.push(((*t).into(), e.clone(), c.clone()));
        }
        for (t, i) in &q_group1.1 {
            snapshot.items.push(((*t).into(), i.clone()));
        }
        for (t, p) in &q_group1.2 {
            snapshot.projectiles.push(((*t).into(), p.clone()));
        }
        for (t, e) in &q_group1.3 {
            snapshot.sector_effectors.push(((*t).into(), e.clone()));
        }
        for (t, s) in &q_group1.4 {
            snapshot.switches.push(((*t).into(), s.clone()));
        }
        for (t, c) in &q_group1.5 {
            snapshot.crack_walls.push(((*t).into(), c.clone()));
        }
        for (t, g) in &q_group1.6 {
            snapshot.glass_panes.push(((*t).into(), g.clone()));
        }
        for (t, b) in &q_group1.7 {
            snapshot.barrels.push(((*t).into(), b.clone()));
        }

        for (t, f) in &q_group2.0 {
            snapshot.fountains.push(((*t).into(), f.clone()));
        }
        for (t, tp) in &q_group2.1 {
            snapshot.toilets.push(((*t).into(), tp.clone()));
        }
        for (t, d) in &q_group2.2 {
            snapshot.dancers.push(((*t).into(), d.clone()));
        }
        for (t, v) in &q_group2.3 {
            snapshot.viewscreens.push(((*t).into(), v.clone()));
        }
        for (t, c) in &q_group2.4 {
            snapshot.cameras.push(((*t).into(), c.clone()));
        }
        for (t, m) in &q_group2.5 {
            snapshot.camera_monitors.push(((*t).into(), m.clone()));
        }
        for (t, n) in &q_group2.6 {
            snapshot.nuke_switches.push(((*t).into(), n.clone()));
        }
        for (t, m) in &q_group2.7 {
            snapshot.master_switches.push(((*t).into(), m.clone()));
        }

        let path = match event.slot {
            Some(slot) => {
                save_mgr.last_saved_slot = Some(slot);
                if slot < save_mgr.save_slots_info.len() {
                    save_mgr.save_slots_info[slot] = Some(event.title.clone());
                }
                get_save_path_for_slot(slot)
            }
            None => get_quicksave_path(),
        };

        if let Ok(()) = write_save_to_disk(&path, &snapshot) {
            println!("Game successfully saved to {:?}", path);
        } else {
            eprintln!("Failed to write savegame to {:?}", path);
        }
    }
}

#[derive(Resource)]
pub struct PendingSaveLoad(pub SaveGameSnapshot);

pub fn handle_load_events(
    mut commands: Commands,
    mut events: EventReader<LoadGameEvent>,
    mut progress: ResMut<LevelProgress>,
    mut load_level_events: EventWriter<LoadLevelEvent>,
) {
    for event in events.read() {
        let path = match event.slot {
            Some(slot) => get_save_path_for_slot(slot),
            None => get_quicksave_path(),
        };

        if let Ok(snapshot) = read_save_from_disk(&path) {
            println!(
                "Loading savegame from {:?}: Episode {} Level {}",
                path, snapshot.episode, snapshot.level
            );

            let need_level_switch = progress.current_episode != snapshot.episode as usize
                || progress.current_level != snapshot.level as usize;

            progress.current_episode = snapshot.episode as usize;
            progress.current_level = snapshot.level as usize;
            progress.kills_count = snapshot.kills_count;
            progress.secrets_found = snapshot.secrets_found;
            progress.level_time_seconds = snapshot.level_time_seconds;

            if need_level_switch {
                load_level_events.send(LoadLevelEvent {
                    episode: snapshot.episode as usize,
                    level: snapshot.level as usize,
                });
            }

            // Always insert the pending save. It will be applied by `apply_pending_save`
            // after the map is built (if a switch was needed) or immediately (if not).
            commands.insert_resource(PendingSaveLoad(snapshot));
        } else {
            eprintln!("Failed to read savegame from {:?}", path);
        }
    }
}

pub fn apply_pending_save(
    mut commands: Commands,
    pending: Option<Res<PendingSaveLoad>>,
    mut player_query: Query<(&mut Transform, &mut PlayerController), With<crate::Player>>,
    dynamic_query: Query<Entity, With<Saveable>>,
) {
    let Some(pending_res) = pending else {
        return;
    };
    let snapshot = &pending_res.0;

    // Despawn all existing dynamic entities (either from current level or freshly spawned by Level Builder)
    for e in &dynamic_query {
        commands.entity(e).despawn_recursive();
    }

    // Restore Player
    if let Some((saved_trans, saved_player)) = &snapshot.player {
        if let Ok((mut trans, mut player)) = player_query.get_single_mut() {
            *trans = saved_trans.clone().into();
            *player = saved_player.clone();
        }
    }

    // Restore Dynamics
    for (st, e, c) in &snapshot.enemies {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            e.clone(),
            c.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, i) in &snapshot.items {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            i.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, p) in &snapshot.projectiles {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            p.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, e) in &snapshot.sector_effectors {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            e.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, s) in &snapshot.switches {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            s.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, c) in &snapshot.crack_walls {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            c.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, g) in &snapshot.glass_panes {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            g.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, b) in &snapshot.barrels {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            b.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, f) in &snapshot.fountains {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            f.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, tp) in &snapshot.toilets {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            tp.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, d) in &snapshot.dancers {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            d.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, v) in &snapshot.viewscreens {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            v.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, c) in &snapshot.cameras {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            c.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, m) in &snapshot.camera_monitors {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            m.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, n) in &snapshot.nuke_switches {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            n.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, m) in &snapshot.master_switches {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            m.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }

    commands.remove_resource::<PendingSaveLoad>();
}

pub fn auto_tag_saveable(
    mut commands: Commands,
    q1: Query<
        Entity,
        (
            Or<(
                With<crate::combat::types::EnemyActor>,
                With<crate::interactivity::types::ItemPickup>,
                With<crate::combat::types::Projectile>,
                With<crate::interactivity::types::SectorEffectorComponent>,
                With<crate::interactivity::types::InteractiveSwitch>,
                With<crate::interactivity::types::CrackWall>,
                With<crate::interactivity::types::BreakableGlass>,
                With<crate::interactivity::types::ExplodingBarrel>,
            )>,
            Without<Saveable>,
        ),
    >,
    q2: Query<
        Entity,
        (
            Or<(
                With<crate::interactivity::types::WaterFountain>,
                With<crate::interactivity::types::ToiletProp>,
                With<crate::interactivity::props_extended::DancerProp>,
                With<crate::interactivity::types::ViewscreenProp>,
                With<crate::interactivity::types::SecurityCamera>,
                With<crate::interactivity::props_extended::SecurityCameraMonitor>,
                With<crate::interactivity::types::NukeExitSwitch>,
                With<crate::interactivity::types::MasterSwitch>,
            )>,
            Without<Saveable>,
        ),
    >,
) {
    for e in &q1 {
        commands.entity(e).insert(Saveable);
    }
    for e in &q2 {
        commands.entity(e).insert(Saveable);
    }
}

pub fn hydrate_enemies(
    mut commands: Commands,
    query: Query<
        (Entity, Option<&crate::scripting::ConActor>),
        (
            With<RequiresHydration>,
            With<crate::combat::types::EnemyActor>,
        ),
    >,
    assets: Option<Res<crate::GameAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(game_assets) = assets else {
        return;
    };
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for (e, con) in &query {
        let picnum = con.map_or(2000, |c| c.picnum);
        let mat = game_assets
            .tile_textures
            .get(&picnum)
            .cloned()
            .map(|t| {
                materials.add(StandardMaterial {
                    base_color_texture: Some(t),
                    alpha_mode: AlphaMode::Mask(0.5),
                    unlit: true,
                    ..default()
                })
            })
            .unwrap_or(game_assets.default_material.clone());
        commands.entity(e).insert((
            PbrBundle {
                mesh: quad_mesh.clone(),
                material: mat,
                ..default()
            },
            crate::SpriteBillboard,
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.1),
            crate::Destructible {
                health: 100,
                _picnum: picnum,
            },
        ));
        commands.entity(e).remove::<RequiresHydration>();
    }
}

pub fn hydrate_items(
    mut commands: Commands,
    query: Query<(Entity, &crate::interactivity::types::ItemPickup), With<RequiresHydration>>,
    assets: Option<Res<crate::GameAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(game_assets) = assets else {
        return;
    };
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for (e, item) in &query {
        let picnum = match item.kind {
            crate::interactivity::types::PickupKind::SmallMedkit => 51,
            crate::interactivity::types::PickupKind::LargeMedkit => 52,
            crate::interactivity::types::PickupKind::PortableMedkit => 53,
            crate::interactivity::types::PickupKind::AtomicHealth => 100,
            crate::interactivity::types::PickupKind::ArmorVest => 54,
            crate::interactivity::types::PickupKind::PistolClip => 40,
            crate::interactivity::types::PickupKind::ShotgunBox => 49,
            crate::interactivity::types::PickupKind::RpgRocket => 47,
            _ => 100,
        };
        let mat = game_assets
            .tile_textures
            .get(&picnum)
            .cloned()
            .map(|t| {
                materials.add(StandardMaterial {
                    base_color_texture: Some(t),
                    alpha_mode: AlphaMode::Mask(0.5),
                    unlit: true,
                    ..default()
                })
            })
            .unwrap_or(game_assets.default_material.clone());
        commands.entity(e).insert((
            PbrBundle {
                mesh: quad_mesh.clone(),
                material: mat,
                ..default()
            },
            crate::SpriteBillboard,
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.1),
            crate::Destructible {
                health: 10,
                _picnum: picnum,
            },
        ));
        commands.entity(e).remove::<RequiresHydration>();
    }
}

pub fn hydrate_switches_and_props(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&crate::interactivity::types::InteractiveSwitch>,
            Option<&crate::interactivity::types::CrackWall>,
            Option<&crate::interactivity::types::BreakableGlass>,
            Option<&crate::interactivity::types::ExplodingBarrel>,
            Option<&crate::interactivity::types::WaterFountain>,
            Option<&crate::interactivity::types::ToiletProp>,
            Option<&crate::interactivity::props_extended::DancerProp>,
            Option<&crate::interactivity::types::ViewscreenProp>,
            Option<&crate::interactivity::types::SecurityCamera>,
            Option<&crate::interactivity::props_extended::SecurityCameraMonitor>,
            Option<&crate::interactivity::types::NukeExitSwitch>,
            Option<&crate::interactivity::types::MasterSwitch>,
        ),
        With<RequiresHydration>,
    >,
    assets: Option<Res<crate::GameAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(game_assets) = assets else {
        return;
    };
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for (
        e,
        sw,
        crack,
        glass,
        barrel,
        fountain,
        toilet,
        dancer,
        viewscreen,
        camera,
        monitor,
        nuke,
        master,
    ) in &query
    {
        let mut picnum = -1;
        if let Some(s) = sw {
            picnum = if s.is_on { s.on_tile } else { s.off_tile };
        } else if crack.is_some() {
            picnum = 546;
        } else if glass.is_some() {
            picnum = 502;
        } else if barrel.is_some() {
            picnum = 122;
        } else if fountain.is_some() {
            picnum = 564;
        } else if toilet.is_some() {
            picnum = 1210;
        } else if dancer.is_some() {
            picnum = 1324;
        } else if viewscreen.is_some() {
            picnum = 660;
        } else if camera.is_some() {
            picnum = 650;
        } else if monitor.is_some() {
            picnum = 660;
        } else if nuke.is_some() {
            picnum = 142;
        } else if master.is_some() {
            picnum = 130;
        }

        if picnum != -1 {
            let mat = game_assets
                .tile_textures
                .get(&picnum)
                .cloned()
                .map(|t| {
                    materials.add(StandardMaterial {
                        base_color_texture: Some(t),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: true,
                        ..default()
                    })
                })
                .unwrap_or(game_assets.default_material.clone());
            commands.entity(e).insert((
                PbrBundle {
                    mesh: quad_mesh.clone(),
                    material: mat,
                    ..default()
                },
                crate::SpriteBillboard,
                RigidBody::Fixed,
                Collider::cuboid(0.5, 0.5, 0.1),
                crate::Destructible {
                    health: 10,
                    _picnum: picnum,
                },
            ));
        }
        commands.entity(e).remove::<RequiresHydration>();
    }
}

pub fn hydrate_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &crate::combat::types::Projectile), With<RequiresHydration>>,
    assets: Option<Res<crate::GameAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(game_assets) = assets else {
        return;
    };
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for (e, proj) in &query {
        let picnum = match proj.projectile_type {
            crate::combat::types::ProjectileType::Rocket => 2605,
            crate::combat::types::ProjectileType::ShrinkRay => 1646,
            crate::combat::types::ProjectileType::FreezeShard => 1641,
            crate::combat::types::ProjectileType::Spit => 1636,
            crate::combat::types::ProjectileType::Mortar => 1650,
            _ => 2595,
        };
        let mat = game_assets
            .tile_textures
            .get(&picnum)
            .cloned()
            .map(|t| {
                materials.add(StandardMaterial {
                    base_color_texture: Some(t),
                    alpha_mode: AlphaMode::Mask(0.5),
                    unlit: true,
                    ..default()
                })
            })
            .unwrap_or(game_assets.default_material.clone());
        commands.entity(e).insert((
            PbrBundle {
                mesh: quad_mesh.clone(),
                material: mat,
                ..default()
            },
            crate::SpriteBillboard,
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.1),
            crate::Destructible {
                health: 10,
                _picnum: picnum,
            },
        ));
        commands.entity(e).remove::<RequiresHydration>();
    }
}

pub fn hydrate_effectors(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<RequiresHydration>,
            With<crate::interactivity::types::SectorEffectorComponent>,
        ),
    >,
) {
    for e in &query {
        commands.entity(e).remove::<RequiresHydration>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_game_snapshot_roundtrip() {
        let mut player = PlayerController::default();
        player.health = 85;
        player.armor = 50;
        player.current_weapon = crate::player::types::WeaponType::Shotgun;
        player.weapons[2].ammo = 18; // Shotgun ammo
        player.inventory.jetpack_amount = 75;
        player.inventory.jetpack_active = true;
        player.has_blue_key = true;

        let mut snapshot = SaveGameSnapshot::new("Test Save Slot", 1, 2, 1, 12, 1, 65.4);
        snapshot.player = Some((
            SavedTransform {
                translation: [-15.5, 4.2, 8.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            player,
        ));

        let bytes = bincode::serialize(&snapshot).expect("Failed to serialize");
        let loaded: SaveGameSnapshot =
            bincode::deserialize(&bytes).expect("Failed to deserialize snapshot");

        assert_eq!(loaded.magic, *SAVEGAME_MAGIC);
        assert_eq!(loaded.version, BYTEVERSION);
        assert_eq!(loaded.title, "Test Save Slot");
        assert_eq!(loaded.episode, 1);
        assert_eq!(loaded.level, 2);
        assert_eq!(loaded.kills_count, 12);
        assert_eq!(loaded.secrets_found, 1);
        assert!((loaded.level_time_seconds - 65.4).abs() < 0.001);

        let restored_player = loaded.player.unwrap().1;
        assert_eq!(restored_player.health, 85);
        assert_eq!(restored_player.armor, 50);
        assert_eq!(
            restored_player.current_weapon,
            crate::player::types::WeaponType::Shotgun
        );
        assert_eq!(restored_player.weapons[2].ammo, 18);
        assert_eq!(restored_player.inventory.jetpack_amount, 75);
        assert!(restored_player.inventory.jetpack_active);
        assert!(restored_player.has_blue_key);
    }
}
