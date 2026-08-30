#![allow(dead_code)]

pub mod format;
pub mod snapshot;

pub use format::*;
pub use snapshot::*;

use bevy::prelude::*;
use crate::game_flow::state::*;
use crate::player::types::PlayerController;

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
            .add_systems(Update, (handle_save_load_hotkeys, handle_save_events, handle_load_events));
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

pub fn handle_save_events(
    mut events: EventReader<SaveGameEvent>,
    player_query: Query<(&Transform, &PlayerController), With<crate::Player>>,
    progress: Res<LevelProgress>,
    mut save_mgr: ResMut<SaveManager>,
) {
    let Ok((trans, player)) = player_query.get_single() else { return; };

    for event in events.read() {
        let player_pos = (trans.translation.x, trans.translation.y, trans.translation.z);
        let snapshot = SaveGameSnapshot::new(
            &event.title,
            progress.current_episode as u8,
            progress.current_level as u8,
            progress.skill as u8,
            player_pos,
            player,
            progress.kills_count,
            progress.secrets_found,
            progress.level_time_seconds,
        );

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

pub fn handle_load_events(
    mut events: EventReader<LoadGameEvent>,
    mut player_query: Query<(&mut Transform, &mut PlayerController), With<crate::Player>>,
    mut progress: ResMut<LevelProgress>,
    mut load_level_events: EventWriter<LoadLevelEvent>,
) {
    for event in events.read() {
        let path = match event.slot {
            Some(slot) => get_save_path_for_slot(slot),
            None => get_quicksave_path(),
        };

        if let Ok(snapshot) = read_save_from_disk(&path) {
            println!("Loading savegame from {:?}: Episode {} Level {}", path, snapshot.episode, snapshot.level);

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

            if let Ok((mut trans, mut player)) = player_query.get_single_mut() {
                trans.translation = Vec3::new(snapshot.pos_x, snapshot.pos_y, snapshot.pos_z);
                snapshot.apply_to_player(&mut player);
                println!("Player restored: pos={:?}, health={}, armor={}", trans.translation, player.health, player.armor);
            }
        } else {
            eprintln!("Failed to read savegame from {:?}", path);
        }
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

        let player_pos = (-15.5, 4.2, 8.0);
        let snapshot = SaveGameSnapshot::new(
            "Test Save Slot",
            1,
            2,
            1,
            player_pos,
            &player,
            12,
            1,
            65.4,
        );

        let bytes = snapshot.to_bytes();
        let loaded = SaveGameSnapshot::from_bytes(&bytes).expect("Failed to deserialize snapshot");

        assert_eq!(loaded.magic, *SAVEGAME_MAGIC);
        assert_eq!(loaded.version, BYTEVERSION);
        assert_eq!(loaded.title, "Test Save Slot");
        assert_eq!(loaded.episode, 1);
        assert_eq!(loaded.level, 2);
        assert_eq!(loaded.health, 85);
        assert_eq!(loaded.armor, 50);
        assert_eq!(loaded.current_weapon, 2);
        assert_eq!(loaded.weapons[2].0, 18);
        assert_eq!(loaded.jetpack_amount, 75);
        assert!(loaded.jetpack_active);
        assert!(loaded.has_blue_key);
        assert_eq!(loaded.kills_count, 12);
        assert_eq!(loaded.secrets_found, 1);
        assert!((loaded.level_time_seconds - 65.4).abs() < 0.001);

        let mut restored_player = PlayerController::default();
        loaded.apply_to_player(&mut restored_player);
        assert_eq!(restored_player.health, 85);
        assert_eq!(restored_player.armor, 50);
        assert_eq!(restored_player.current_weapon, crate::player::types::WeaponType::Shotgun);
        assert_eq!(restored_player.weapons[2].ammo, 18);
        assert_eq!(restored_player.inventory.jetpack_amount, 75);
        assert!(restored_player.inventory.jetpack_active);
        assert!(restored_player.has_blue_key);
    }
}
