#![allow(dead_code)]

pub mod format;
pub mod snapshot;

pub use format::*;
pub use snapshot::*;

use crate::game_flow::state::*;
use crate::player::types::PlayerController;
use crate::names::*;
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

#[derive(Resource, Debug, Clone)]
pub struct SaveManager {
    pub last_saved_slot: Option<usize>,
    pub save_slots_info: [Option<String>; 10],
}

impl Default for SaveManager {
    fn default() -> Self {
        let mut mgr = Self {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        };
        mgr.scan_save_slots();
        mgr
    }
}

impl SaveManager {
    pub fn scan_save_slots(&mut self) {
        for slot in 0..10 {
            let path = get_save_path_for_slot(slot);
            if path.exists() {
                if let Ok(snapshot) = read_save_from_disk(&path) {
                    self.save_slots_info[slot] = Some(snapshot.title);
                } else {
                    self.save_slots_info[slot] = None;
                }
            } else {
                self.save_slots_info[slot] = None;
            }
        }
    }
}

pub fn scan_save_slots_system(mut save_mgr: ResMut<SaveManager>) {
    save_mgr.scan_save_slots();
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        let mut save_mgr = SaveManager::default();
        save_mgr.scan_save_slots();
        app.insert_resource(save_mgr)
            .add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_systems(Startup, scan_save_slots_system)
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
    mut next_state: ResMut<NextState<GamePhase>>,
    mut cursor: Option<ResMut<crate::game_flow::menu::MenuCursor>>,
    mut save_load_origin: Option<ResMut<crate::game_flow::state::SaveLoadOrigin>>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
) {
    if *state.get() != GamePhase::Playing {
        return;
    }

    // F2: open Save Menu
    if keys.just_pressed(KeyCode::F2) {
        if let Some(ref mut c) = cursor {
            c.selected_index = 0;
            c.max_items = 10;
        }
        if let Some(ref mut origin) = save_load_origin {
            origin.0 = GamePhase::Paused;
        }
        next_state.set(GamePhase::SaveMenu);
    }

    // F3: open Load Menu
    if keys.just_pressed(KeyCode::F3) {
        if let Some(ref mut c) = cursor {
            c.selected_index = 0;
            c.max_items = 10;
        }
        if let Some(ref mut origin) = save_load_origin {
            origin.0 = GamePhase::Paused;
        }
        next_state.set(GamePhase::LoadMenu);
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
    q_group3: (
        Query<(&Transform, &crate::interactivity::types::FireExtinguisher), With<Saveable>>,
        Query<(&Transform, &crate::interactivity::types::MirrorProp), With<Saveable>>,
    ),
    progress: Res<LevelProgress>,
    mut save_mgr: ResMut<SaveManager>,
    sector_map: Option<Res<crate::sector_map::SectorMap>>,
    found_secrets: Option<Res<crate::game_flow::state::FoundSecretSectors>>,
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

        if let Some(ref sm) = sector_map {
            for (idx, sector) in sm.sectors.iter().enumerate() {
                snapshot.sector_elevations.push((idx, sector.floorz, sector.ceilingz));
            }
        }

        if let Some(ref sec) = found_secrets {
            let mut secrets: Vec<usize> = sec.0.iter().copied().collect();
            secrets.sort_unstable();
            snapshot.found_secrets = secrets;
        }

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

        for (t, fe) in &q_group3.0 {
            snapshot.fire_extinguishers.push(((*t).into(), fe.clone()));
        }
        for (t, mp) in &q_group3.1 {
            snapshot.mirrors.push(((*t).into(), mp.clone()));
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
    sector_map: Option<ResMut<crate::sector_map::SectorMap>>,
    mut dynamic_sector_meshes: Query<
        (&crate::interactivity::DynamicSectorMesh, &mut Transform),
        Without<crate::Player>,
    >,
    found_secrets: Option<ResMut<crate::game_flow::state::FoundSecretSectors>>,
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
    for (st, fe) in &snapshot.fire_extinguishers {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            fe.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }
    for (st, mp) in &snapshot.mirrors {
        commands.spawn((
            Into::<Transform>::into(st.clone()),
            mp.clone(),
            Saveable,
            RequiresHydration,
            crate::game_flow::state::LevelEntity,
        ));
    }

    // Restore sector elevations into SectorMap and update DynamicSectorMesh transforms
    if let Some(mut sm) = sector_map {
        let mut deltas: std::collections::HashMap<usize, (f32, f32)> =
            std::collections::HashMap::new();

        for &(idx, floorz, ceilingz) in &snapshot.sector_elevations {
            let base_floorz = sm.get_base_floorz(idx);
            let base_ceilingz = sm.get_base_ceilingz(idx);

            let delta_floor_y = -((floorz - base_floorz) as f32) / (1024.0 * 16.0);
            let delta_ceiling_y = -((ceilingz - base_ceilingz) as f32) / (1024.0 * 16.0);
            deltas.insert(idx, (delta_floor_y, delta_ceiling_y));

            if let Some(sec) = sm.sectors.get_mut(idx) {
                sec.floorz = floorz;
                sec.ceilingz = ceilingz;
            }
        }

        for (dyn_mesh, mut transform) in &mut dynamic_sector_meshes {
            if let Some(&(delta_floor_y, delta_ceiling_y)) = deltas.get(&dyn_mesh.sector_idx) {
                match dyn_mesh.part {
                    crate::interactivity::types::SectorMeshPart::Floor
                    | crate::interactivity::types::SectorMeshPart::LowerWall => {
                        transform.translation.y = delta_floor_y;
                    }
                    crate::interactivity::types::SectorMeshPart::Ceiling
                    | crate::interactivity::types::SectorMeshPart::UpperWall => {
                        transform.translation.y = delta_ceiling_y;
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore found secrets
    if let Some(mut found_sec) = found_secrets {
        found_sec.0 = snapshot.found_secrets.iter().copied().collect();
    } else {
        commands.insert_resource(crate::game_flow::state::FoundSecretSectors(
            snapshot.found_secrets.iter().copied().collect(),
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
                With<crate::interactivity::types::FireExtinguisher>,
                With<crate::interactivity::types::MirrorProp>,
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
            Option<&crate::interactivity::types::FireExtinguisher>,
            Option<&crate::interactivity::types::MirrorProp>,
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
        ext,
        mirror,
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
            picnum = NUKEBUTTON;
        } else if master.is_some() {
            picnum = 130;
        } else if ext.is_some() {
            picnum = FIREEXT;
        } else if mirror.is_some() {
            picnum = MIRROR;
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
            crate::combat::types::ProjectileType::Rocket => RPG,
            crate::combat::types::ProjectileType::ShrinkRay => 1646,
            crate::combat::types::ProjectileType::FreezeShard => 1641,
            crate::combat::types::ProjectileType::Spit => 1636,
            crate::combat::types::ProjectileType::Mortar => 1650,
            _ => SHOTSPARK1,
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
pub mod tests {
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

    #[test]
    fn test_fire_extinguisher_and_mirror_save_roundtrip() {
        let mut snapshot = SaveGameSnapshot::new("Prop Test", 1, 1, 1, 0, 0, 10.0);
        snapshot.fire_extinguishers.push((
            SavedTransform {
                translation: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            crate::interactivity::types::FireExtinguisher {
                health: 0,
                is_exploded: true,
            },
        ));
        snapshot.mirrors.push((
            SavedTransform {
                translation: [4.0, 5.0, 6.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            crate::interactivity::types::MirrorProp {
                cooldown_timer: 1.5,
            },
        ));

        let bytes = bincode::serialize(&snapshot).expect("Serialization failed");
        let loaded: SaveGameSnapshot = bincode::deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(loaded.fire_extinguishers.len(), 1);
        assert!(loaded.fire_extinguishers[0].1.is_exploded);
        assert_eq!(loaded.fire_extinguishers[0].1.health, 0);

        assert_eq!(loaded.mirrors.len(), 1);
        assert_eq!(loaded.mirrors[0].1.cooldown_timer, 1.5);
    }

    #[test]
    fn test_sector_elevations_and_secrets_snapshot_roundtrip() {
        let mut snapshot = SaveGameSnapshot::new("Sector Elevation & Secret Test", 1, 1, 1, 5, 2, 42.0);
        snapshot.sector_elevations = vec![(0, 4096 * 16, -20480), (1, 8192, -32768)];
        snapshot.found_secrets = vec![2, 5, 11];

        let bytes = bincode::serialize(&snapshot).expect("Serialization failed");
        let loaded: SaveGameSnapshot = bincode::deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(loaded.sector_elevations, vec![(0, 4096 * 16, -20480), (1, 8192, -32768)]);
        assert_eq!(loaded.found_secrets, vec![2, 5, 11]);
    }

    #[test]
    fn test_apply_pending_save_sector_elevations_and_secrets() {
        let s0 = crate::map::Sector {
            wallptr: 0,
            wallnum: 4,
            ceilingz: -16384,
            floorz: 0,
            ceilingstat: 0,
            floorstat: 0,
            ceilingpicnum: 0,
            ceilingheinum: 0,
            ceilingshade: 0,
            ceilingpal: 0,
            ceilingxpanning: 0,
            ceilingypanning: 0,
            floorpicnum: 0,
            floorheinum: 0,
            floorshade: 0,
            floorpal: 0,
            floorxpanning: 0,
            floorypanning: 0,
            visibility: 0,
            _filler: 0,
            lotag: 0,
            hitag: 0,
            extra: 0,
        };
        let s1 = s0.clone();
        let map = crate::map::Map {
            version: 7,
            posx: 0,
            posy: 0,
            posz: 0,
            ang: 0,
            cursectnum: 0,
            sectors: vec![s0, s1],
            walls: vec![],
            sprites: vec![],
        };
        let sm = crate::sector_map::SectorMap::from_map(&map);

        let mut app = App::new();
        app.insert_resource(sm);
        app.insert_resource(crate::game_flow::state::FoundSecretSectors(Default::default()));

        // Spawn player controller
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            PlayerController::default(),
        ));

        // Spawn a DynamicSectorMesh for floor and ceiling of sector 0
        let mesh_floor = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                crate::interactivity::DynamicSectorMesh {
                    sector_idx: 0,
                    part: crate::interactivity::SectorMeshPart::Floor,
                },
            ))
            .id();

        let mesh_ceil = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                crate::interactivity::DynamicSectorMesh {
                    sector_idx: 0,
                    part: crate::interactivity::SectorMeshPart::Ceiling,
                },
            ))
            .id();

        let mut snapshot = SaveGameSnapshot::new("Elev Hydration Test", 1, 1, 1, 0, 1, 10.0);
        // Base floor was 0; 1024 * 16 units delta -> delta_floor_y = -1.0
        // Base ceiling was -16384; -32768 (-16384 - 1024*16) -> delta_ceiling_y = 1.0
        snapshot.sector_elevations = vec![(0, 1024 * 16, -32768)];
        snapshot.found_secrets = vec![42, 99];
        app.insert_resource(PendingSaveLoad(snapshot));

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_pending_save);
        schedule.run(app.world_mut());

        // Verify SectorMap elevations updated
        let sm_res = app.world().resource::<crate::sector_map::SectorMap>();
        assert_eq!(sm_res.sectors[0].floorz, 1024 * 16);
        assert_eq!(sm_res.sectors[0].ceilingz, -32768);

        // Verify DynamicSectorMesh transforms updated
        let floor_trans = app.world().entity(mesh_floor).get::<Transform>().unwrap();
        assert!((floor_trans.translation.y - (-1.0)).abs() < 1e-4);

        let ceil_trans = app.world().entity(mesh_ceil).get::<Transform>().unwrap();
        assert!((ceil_trans.translation.y - 1.0).abs() < 1e-4);

        // Verify FoundSecretSectors updated
        let secrets_res = app.world().resource::<crate::game_flow::state::FoundSecretSectors>();
        assert!(secrets_res.0.contains(&42));
        assert!(secrets_res.0.contains(&99));
        assert_eq!(secrets_res.0.len(), 2);
    }

    pub static SAVE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_handle_save_load_hotkeys() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<crate::game_flow::menu::MenuCursor>();
        app.init_resource::<crate::game_flow::state::SaveLoadOrigin>();
        app.add_event::<SaveGameEvent>();
        app.add_event::<LoadGameEvent>();
        app.add_systems(Update, handle_save_load_hotkeys);

        // Initial default state is MainMenu: pressing hotkeys does nothing
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F2);
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::MainMenu);

        // Transition to Playing state
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::Playing);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Playing);

        // 1. F2: open Save Menu
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F2);
        app.update();
        let cursor = app.world().resource::<crate::game_flow::menu::MenuCursor>();
        assert_eq!(cursor.selected_index, 0);
        assert_eq!(cursor.max_items, 10);
        let origin = app.world().resource::<crate::game_flow::state::SaveLoadOrigin>();
        assert_eq!(origin.0, GamePhase::Paused);

        // Apply state transition
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::SaveMenu);

        // Reset to Playing
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::Playing);
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Playing);

        // 2. F3: open Load Menu
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F3);
        app.update();
        let cursor = app.world().resource::<crate::game_flow::menu::MenuCursor>();
        assert_eq!(cursor.selected_index, 0);
        assert_eq!(cursor.max_items, 10);
        let origin = app.world().resource::<crate::game_flow::state::SaveLoadOrigin>();
        assert_eq!(origin.0, GamePhase::Paused);

        // Apply state transition
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::LoadMenu);

        // Reset to Playing
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::Playing);
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Playing);

        // 3. F6: Quicksave
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F6);
        app.update();
        {
            let save_events = app.world().resource::<Events<SaveGameEvent>>();
            let mut reader = save_events.get_reader();
            let emitted: Vec<_> = reader.read(save_events).cloned().collect();
            assert_eq!(emitted.len(), 1);
            assert_eq!(emitted[0].slot, None);
            assert_eq!(emitted[0].title, "Quicksave");
        }

        // 4. F9: Quickload
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F9);
        app.update();
        {
            let load_events = app.world().resource::<Events<LoadGameEvent>>();
            let mut reader = load_events.get_reader();
            let emitted: Vec<_> = reader.read(load_events).cloned().collect();
            assert_eq!(emitted.len(), 1);
            assert_eq!(emitted[0].slot, None);
        }
    }

    #[test]
    fn test_save_manager_scan_save_slots() {
        let _lock = SAVE_TEST_LOCK.lock().unwrap();

        let test_slot = 7;
        let path = get_save_path_for_slot(test_slot);

        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let _guard = Cleanup(path.clone());

        // Ensure initially clean
        let _ = std::fs::remove_file(&path);

        let snapshot = SaveGameSnapshot::new("Disk Meta Test E1L4", 1, 4, 2, 25, 3, 110.0);
        write_save_to_disk(&path, &snapshot).expect("Failed to write test save file");

        let mut save_mgr = SaveManager {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        };
        save_mgr.scan_save_slots();

        assert_eq!(
            save_mgr.save_slots_info[test_slot],
            Some("Disk Meta Test E1L4".to_string())
        );

        // Clean up file and re-scan
        let _ = std::fs::remove_file(&path);
        save_mgr.scan_save_slots();
        assert_eq!(save_mgr.save_slots_info[test_slot], None);
    }

    #[test]
    fn test_roundtrip_write_read_all_10_slots_with_cleanup() {
        let _lock = SAVE_TEST_LOCK.lock().unwrap();

        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                for slot in 0..10 {
                    let _ = std::fs::remove_file(get_save_path_for_slot(slot));
                }
            }
        }
        let _guard = Guard;

        // Clean any pre-existing files
        for slot in 0..10 {
            let _ = std::fs::remove_file(get_save_path_for_slot(slot));
            assert!(!slot_save_exists(slot));
        }

        let mut save_mgr = SaveManager {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        };
        save_mgr.scan_save_slots();
        for slot in 0..10 {
            assert_eq!(save_mgr.save_slots_info[slot], None);
        }

        // Write snapshots to all 10 slots (0..9)
        for slot in 0..10 {
            let title = format!("Ep1 Mission {}", slot + 1);
            let mut snapshot = SaveGameSnapshot::new(
                &title,
                1,
                (slot as u8) + 1,
                1,
                (slot as i32) * 5,
                slot as i32,
                45.5 * ((slot + 1) as f32),
            );
            let mut player = PlayerController::default();
            player.health = 100 - (slot as i32) * 5;
            player.armor = (slot as i32) * 8;
            snapshot.player = Some((
                SavedTransform {
                    translation: [slot as f32, 1.0, 2.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                player,
            ));

            let path = get_save_path_for_slot(slot);
            write_save_to_disk(&path, &snapshot).expect("Failed to write snapshot to disk");
            assert!(slot_save_exists(slot));

            let loaded = read_save_from_disk(&path).expect("Failed to read snapshot from disk");
            assert_eq!(loaded.magic, *SAVEGAME_MAGIC);
            assert_eq!(loaded.version, BYTEVERSION);
            assert_eq!(loaded.title, title);
            assert_eq!(loaded.episode, 1);
            assert_eq!(loaded.level, (slot as u8) + 1);
            assert_eq!(loaded.kills_count, (slot as i32) * 5);
            assert_eq!(loaded.secrets_found, slot as i32);
            assert!((loaded.level_time_seconds - 45.5 * ((slot + 1) as f32)).abs() < 1e-3);
            let p = loaded.player.unwrap().1;
            assert_eq!(p.health, 100 - (slot as i32) * 5);
            assert_eq!(p.armor, (slot as i32) * 8);
        }

        // Verify scan_save_slots discovers all 10 slots
        save_mgr.scan_save_slots();
        for slot in 0..10 {
            let expected_title = format!("Ep1 Mission {}", slot + 1);
            assert_eq!(save_mgr.save_slots_info[slot], Some(expected_title));
        }

        // Cleanup files and verify
        for slot in 0..10 {
            let path = get_save_path_for_slot(slot);
            let _ = std::fs::remove_file(&path);
            assert!(!slot_save_exists(slot));
        }

        save_mgr.scan_save_slots();
        for slot in 0..10 {
            assert_eq!(save_mgr.save_slots_info[slot], None);
        }
    }
}
