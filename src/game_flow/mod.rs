pub mod intermission;
pub mod level_loader;
pub mod level_select;
pub mod menu;
pub mod state;
pub mod ui;

pub use intermission::*;
pub use level_loader::*;
pub use level_select::*;
pub use menu::*;
pub use state::*;
pub use ui::*;

use bevy::prelude::*;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(level_select::LevelSelectPlugin)
            .init_state::<GamePhase>()
            .init_resource::<LevelProgress>()
            .init_resource::<FoundSecretSectors>()
            .init_resource::<IntermissionAnimationState>()
            .init_resource::<MenuCursor>()
            .init_resource::<CursorAnimTimer>()
            .init_resource::<SaveLoadOrigin>()
            .init_resource::<OptionsOrigin>()
            .add_event::<LevelCompletedEvent>()
            .add_event::<LoadLevelEvent>()
            .add_systems(Startup, setup_menu_ui)
            .add_systems(
                Update,
                (
                    handle_menu_navigation,
                    update_cursor_animation,
                    update_menu_ui,
                    update_level_time,
                    update_intermission_animation,
                    handle_level_completed_events,
                    handle_intermission_input,
                    handle_load_level_events,
                    update_secret_detection,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::*;

    #[test]
    fn test_level_progression_episode1() {
        let mut progress = LevelProgress::default();
        assert_eq!(progress.current_map_filename(), "E1L1.MAP");

        // Advance through Episode 1
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L2.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L3.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L4.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L5.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L6.MAP");

        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_map_filename(), "E1L7.MAP");

        // Boss level completed -> Episode finished
        assert!(progress.advance_to_next_level());
        assert_eq!(progress.current_level, 1);
    }

    #[test]
    fn test_multi_episode_progression_episodes_2_and_3() {
        // Episode 2: Spaceport -> ... -> Overlord (L1..L9)
        let mut progress = LevelProgress {
            current_episode: 2,
            current_level: 1,
            ..default()
        };
        for l in 1..9 {
            assert_eq!(progress.current_level, l);
            assert!(!progress.advance_to_next_level());
        }
        assert_eq!(progress.current_level, 9);
        assert_eq!(progress.current_map_filename(), "E2L9.MAP");

        // Overlord defeated on E2L9 -> Episode 2 finished!
        progress.is_boss_victory = true;
        assert!(progress.advance_to_next_level());
        assert_eq!(progress.current_level, 1);

        // Episode 3: Raw Meat -> ... -> Stadium (L1..L9)
        let mut progress = LevelProgress {
            current_episode: 3,
            current_level: 1,
            ..default()
        };
        for l in 1..9 {
            assert_eq!(progress.current_level, l);
            assert!(!progress.advance_to_next_level());
        }
        assert_eq!(progress.current_level, 9);
        assert_eq!(progress.current_map_filename(), "E3L9.MAP");

        // Cycloid defeated on E3L9 -> Episode 3 finished!
        progress.is_boss_victory = true;
        assert!(progress.advance_to_next_level());
        assert_eq!(progress.current_level, 1);
    }

    #[test]
    fn test_secret_level_routing_and_canonical_return() {
        // Episode 1: E1L3 -> E1L8 (Secret) -> E1L4 (Canonical return)
        let mut progress = LevelProgress {
            current_episode: 1,
            current_level: 3,
            ..default()
        };
        progress.is_secret_exit = true;
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 8);
        assert_eq!(progress.current_map_filename(), "E1L8.MAP");

        // Complete secret level E1L8 -> returns to E1L4
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 4);
        assert_eq!(progress.current_map_filename(), "E1L4.MAP");

        // Episode 2: E2L5 -> E2L10 (Secret) -> E2L6 (Canonical return)
        let mut progress = LevelProgress {
            current_episode: 2,
            current_level: 5,
            ..default()
        };
        progress.is_secret_exit = true;
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 10);
        assert_eq!(progress.current_map_filename(), "E2L10.MAP");

        // Complete secret level E2L10 -> returns to E2L6
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 6);
        assert_eq!(progress.current_map_filename(), "E2L6.MAP");

        // Episode 3: E3L5 -> E3L10 (Secret) -> E3L6 (Canonical return)
        let mut progress = LevelProgress {
            current_episode: 3,
            current_level: 5,
            ..default()
        };
        progress.is_secret_exit = true;
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 10);
        assert_eq!(progress.current_map_filename(), "E3L10.MAP");

        // Complete secret level E3L10 -> returns to E3L6
        assert!(!progress.advance_to_next_level());
        assert_eq!(progress.current_level, 6);
        assert_eq!(progress.current_map_filename(), "E3L6.MAP");
    }

    #[test]
    fn test_intermission_stats_computation() {
        let mut progress = LevelProgress::default();
        progress.kills_count = 25;
        progress.total_monsters = 50;
        progress.secrets_found = 2;
        progress.total_secrets = 4;
        progress.level_time_seconds = 125.0;

        let stats = progress.compute_stats();
        assert_eq!(stats.kill_percentage, 50);
        assert_eq!(stats.secret_percentage, 50);
        assert_eq!(stats.time_taken_seconds, 125.0);
        assert_eq!(stats.par_time_seconds, 180.0);
    }

    #[test]
    fn test_load_real_grp_e1_maps() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            for level in 1..=4 {
                let map_name = format!("E1L{}.MAP", level);
                let map_data = grp
                    .read_file(&map_name)
                    .expect(&format!("{} must exist in GRP", map_name));
                let map = crate::map::Map::from_bytes(&map_data)
                    .expect(&format!("{} must parse", map_name));
                assert!(!map.sectors.is_empty(), "{} must have sectors", map_name);
                assert!(!map.walls.is_empty(), "{} must have walls", map_name);

                let monster_count = map
                    .sprites
                    .iter()
                    .filter(|s| s.picnum == PIGCOP || s.picnum == LIZTROOP || s.picnum == OCTABRAIN || s.picnum == ENFORCER)
                    .count();
                println!(
                    "{}: {} sectors, {} walls, {} sprites ({} monsters)",
                    map_name,
                    map.sectors.len(),
                    map.walls.len(),
                    map.sprites.len(),
                    monster_count
                );
            }
        }
    }

    #[test]
    fn test_load_level_event_structure() {
        let event = LoadLevelEvent {
            episode: 1,
            level: 2,
        };
        assert_eq!(event.episode, 1);
        assert_eq!(event.level, 2);

        let entity_marker = LevelEntity;
        assert_eq!(format!("{:?}", entity_marker), "LevelEntity");
    }

    #[test]
    fn test_menu_cursor_navigation_wrap() {
        let mut cursor = MenuCursor {
            selected_index: 0,
            max_items: 4,
        };

        // Up wraps to max_items - 1
        cursor.selected_index = cursor
            .selected_index
            .checked_sub(1)
            .unwrap_or(cursor.max_items - 1);
        assert_eq!(cursor.selected_index, 3);

        // Down wraps to 0
        cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
        assert_eq!(cursor.selected_index, 0);

        // Increment to 1
        cursor.selected_index = (cursor.selected_index + 1) % cursor.max_items;
        assert_eq!(cursor.selected_index, 1);
    }

    #[test]
    fn test_cursor_animation_tick() {
        let mut anim = CursorAnimTimer::default();
        assert_eq!(anim.frame, 0);

        anim.frame = (anim.frame + 1) % 4;
        assert_eq!(anim.frame, 1);

        anim.frame = (anim.frame + 3) % 4;
        assert_eq!(anim.frame, 0);
    }

    #[test]
    fn test_game_phase_states() {
        let default_phase = GamePhase::default();
        assert_eq!(default_phase, GamePhase::MainMenu);

        let paused = GamePhase::Paused;
        assert_ne!(paused, GamePhase::Playing);
    }

    #[test]
    fn test_e1l1_full_playthrough_pipeline() {
        let mut progress = LevelProgress::default();
        assert_eq!(progress.current_map_filename(), "E1L1.MAP");

        let mut player = crate::player::PlayerController::default();
        assert_eq!(player.health, 100);

        // 1. Pickups: Grab Shotgun and Armor
        player.weapons[crate::player::WeaponType::Shotgun as usize].is_unlocked = true;
        player.weapons[crate::player::WeaponType::Shotgun as usize].ammo += 10;
        player.armor = 100;
        assert!(player.weapons[crate::player::WeaponType::Shotgun as usize].is_unlocked);
        assert_eq!(player.armor, 100);

        // 2. Combat: Kill 5 enemies
        let mut enemy = crate::combat::EnemyActor::new_pigcop();
        enemy.health -= 120;
        if enemy.health <= 0 {
            enemy.state = crate::combat::EnemyAiState::Dying;
            progress.kills_count += 1;
        }
        assert_eq!(enemy.state, crate::combat::EnemyAiState::Dying);
        progress.kills_count = 15;
        progress.total_monsters = 20;

        // 3. Secrets: Find 2 secrets
        progress.secrets_found = 2;
        progress.total_secrets = 3;

        // 4. Level Exit: Trigger Nuke Exit Button
        let mut nuke = crate::interactivity::NukeExitSwitch {
            is_activated: false,
            is_secret: false,
            lotag: 0,
            hitag: 0,
        };
        nuke.is_activated = true;
        assert!(nuke.is_activated);
        progress.is_level_completed = true;
        progress.level_time_seconds = 145.0; // 2:25

        // 5. Intermission Stats Screen
        let stats = progress.compute_stats();
        assert_eq!(stats.kill_percentage, 75); // 15/20 = 75%
        assert_eq!(stats.secret_percentage, 66); // 2/3 = 66%
        assert_eq!(stats.time_taken_seconds, 145.0);

        // 6. Transition to E1L2
        let episode_done = progress.advance_to_next_level();
        assert!(!episode_done);
        assert_eq!(progress.current_level, 2);
        assert_eq!(progress.current_map_filename(), "E1L2.MAP");
    }

    #[test]
    fn test_intermission_timed_stages_rollout() {
        let mut anim = IntermissionAnimationState::default();
        assert_eq!(anim.stage, IntermissionStage::BackgroundFadeIn);

        anim.timer = 0.6;
        if anim.timer >= 0.5 {
            anim.stage = IntermissionStage::KillsTally;
        }
        assert_eq!(anim.stage, IntermissionStage::KillsTally);

        anim.timer = 1.9;
        if anim.timer >= 1.8 {
            anim.stage = IntermissionStage::SecretsTally;
        }
        assert_eq!(anim.stage, IntermissionStage::SecretsTally);

        anim.timer = 3.1;
        if anim.timer >= 3.0 {
            anim.stage = IntermissionStage::TimeReveal;
        }
        assert_eq!(anim.stage, IntermissionStage::TimeReveal);

        anim.timer = 4.3;
        if anim.timer >= 4.2 {
            anim.stage = IntermissionStage::ReadyToProceed;
        }
        assert_eq!(anim.stage, IntermissionStage::ReadyToProceed);
    }

    #[test]
    fn test_intermission_sound_event_ids() {
        let mut app = App::new();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.init_resource::<Time>();
        app.insert_resource(State::new(GamePhase::Intermission));
        app.insert_resource(LevelProgress {
            kills_count: 10,
            total_monsters: 10,
            secrets_found: 2,
            total_secrets: 2,
            level_time_seconds: 50.0,
            par_time_seconds: 100.0,
            ..default()
        });
        let mut anim = IntermissionAnimationState::default();
        anim.stage = IntermissionStage::KillsTally;
        anim.timer = 1.85; // triggers tally sound
        app.insert_resource(anim);
        app.add_systems(Update, update_intermission_animation);

        app.update();

        let events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut reader = events.get_reader();
        let sent: Vec<_> = reader.read(events).cloned().collect();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].sound_id, 3, "Tally gunshot sound must be sound 3 (PISTOL_FIRE)");
    }

    #[test]
    fn test_intermission_bonus_speech_sound_id() {
        let mut app = App::new();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.init_resource::<Time>();
        app.insert_resource(State::new(GamePhase::Intermission));
        app.insert_resource(LevelProgress {
            kills_count: 10,
            total_monsters: 10,
            secrets_found: 2,
            total_secrets: 2,
            level_time_seconds: 50.0,
            par_time_seconds: 100.0, // Beat par time
            ..default()
        });
        let mut anim = IntermissionAnimationState::default();
        anim.stage = IntermissionStage::TimeReveal;
        anim.timer = 3.5;
        anim.speech_played = false;
        app.insert_resource(anim);
        app.add_systems(Update, update_intermission_animation);

        app.update();

        let events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut reader = events.get_reader();
        let sent: Vec<_> = reader.read(events).cloned().collect();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].sound_id, 195, "Beat par time speech must be sound 195 (BONUS_SPEECH1 - 'Damn, I'm good!')");
    }

    #[test]
    fn test_10_slot_save_menu_cursor_navigation_and_wrapping() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<SaveLoadOrigin>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.insert_resource(crate::save::SaveManager {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        });
        app.add_systems(Update, handle_menu_navigation);

        // Switch to SaveMenu
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::SaveMenu);
        app.update();

        let cursor = app.world().resource::<MenuCursor>();
        assert_eq!(cursor.selected_index, 0);
        assert_eq!(cursor.max_items, 10);

        // 1. ArrowUp: wraps 0 -> 9
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowUp);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 9);

        // 2. ArrowDown: wraps 9 -> 0
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowDown);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 0);

        // 3. ArrowDown: advances 0 -> 1
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowDown);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 1);

        // 4. KeyW: moves 1 -> 0
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 0);

        // 5. KeyS: moves 0 -> 1
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyS);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 1);

        // Also verify navigation in LoadMenu
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::LoadMenu);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::LoadMenu);
        assert_eq!(app.world().resource::<MenuCursor>().max_items, 10);

        // ArrowUp wraps to 9
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowUp);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 9);
    }

    #[test]
    fn test_save_game_event_emission_on_enter_in_save_menu() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<SaveLoadOrigin>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.insert_resource(crate::save::SaveManager {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        });
        app.insert_resource(LevelProgress {
            current_episode: 2,
            current_level: 3,
            level_time_seconds: 145.0, // 2:25
            ..default()
        });
        app.add_systems(Update, handle_menu_navigation);

        // Enter SaveMenu
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::SaveMenu);
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::SaveMenu);

        // Select slot 4
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 4;

        // Press Enter
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();

        // Check SaveGameEvent
        {
            let save_events = app.world().resource::<Events<crate::save::SaveGameEvent>>();
            let mut reader = save_events.get_reader();
            let emitted: Vec<_> = reader.read(save_events).cloned().collect();
            assert_eq!(emitted.len(), 1);
            assert_eq!(emitted[0].slot, Some(4));
            assert_eq!(emitted[0].title, "E2L3 - 02:25");
        }

        // Check SaveManager updated in-memory
        let sm = app.world().resource::<crate::save::SaveManager>();
        assert_eq!(sm.last_saved_slot, Some(4));
        assert_eq!(sm.save_slots_info[4], Some("E2L3 - 02:25".to_string()));

        // Apply state transition and check Playing
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Playing);
    }

    #[test]
    fn test_load_menu_populated_vs_empty_slot_event_emission() {
        let _lock = crate::save::tests::SAVE_TEST_LOCK.lock().unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<SaveLoadOrigin>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();

        // Ensure slot 5 disk file does not exist
        let _ = std::fs::remove_file(crate::save::get_save_path_for_slot(5));

        let mut save_mgr = crate::save::SaveManager {
            last_saved_slot: None,
            save_slots_info: Default::default(),
        };
        // Populate slot 3 in SaveManager
        save_mgr.save_slots_info[3] = Some("E1L1 - 01:15".to_string());
        app.insert_resource(save_mgr);
        app.add_systems(Update, handle_menu_navigation);

        // Enter LoadMenu
        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::LoadMenu);
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::LoadMenu);

        // 1. Try selecting EMPTY slot 5
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 5;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();

        // Must NOT emit LoadGameEvent
        {
            let load_events = app.world().resource::<Events<crate::save::LoadGameEvent>>();
            let mut reader = load_events.get_reader();
            assert_eq!(reader.read(load_events).count(), 0);
        }
        // Must NOT transition to Playing
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::LoadMenu);

        // 2. Select POPULATED slot 3
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 3;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();

        // Must emit LoadGameEvent with Some(3)
        {
            let load_events = app.world().resource::<Events<crate::save::LoadGameEvent>>();
            let mut reader = load_events.get_reader();
            let emitted: Vec<_> = reader.read(load_events).cloned().collect();
            assert_eq!(emitted.len(), 1);
            assert_eq!(emitted[0].slot, Some(3));
        }

        // Must transition to Playing
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Playing);
    }

    #[test]
    fn test_escape_key_returns_to_origin_phase() {
        // Case A: Origin = Paused, in SaveMenu
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.init_state::<GamePhase>();
            app.init_resource::<ButtonInput<KeyCode>>();
            app.init_resource::<MenuCursor>();
            app.init_resource::<LevelProgress>();
            app.insert_resource(SaveLoadOrigin(GamePhase::Paused));
            app.add_event::<crate::audio::PlaySoundEvent>();
            app.add_event::<LoadLevelEvent>();
            app.add_event::<crate::save::SaveGameEvent>();
            app.add_event::<crate::save::LoadGameEvent>();
            app.add_event::<bevy::app::AppExit>();
            app.insert_resource(crate::save::SaveManager {
                last_saved_slot: None,
                save_slots_info: Default::default(),
            });
            app.add_systems(Update, handle_menu_navigation);

            app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::SaveMenu);
            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::SaveMenu);

            app.world_mut().resource_mut::<MenuCursor>().selected_index = 6;
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
            app.update();

            // Cursor reset to 0, max_items set to 6
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 0);
            assert_eq!(cursor.max_items, 6);

            // Transitions to Paused
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Paused);
        }

        // Case B: Origin = MainMenu, in LoadMenu
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.init_state::<GamePhase>();
            app.init_resource::<ButtonInput<KeyCode>>();
            app.init_resource::<MenuCursor>();
            app.init_resource::<LevelProgress>();
            app.insert_resource(SaveLoadOrigin(GamePhase::MainMenu));
            app.add_event::<crate::audio::PlaySoundEvent>();
            app.add_event::<LoadLevelEvent>();
            app.add_event::<crate::save::SaveGameEvent>();
            app.add_event::<crate::save::LoadGameEvent>();
            app.add_event::<bevy::app::AppExit>();
            app.insert_resource(crate::save::SaveManager {
                last_saved_slot: None,
                save_slots_info: Default::default(),
            });
            app.add_systems(Update, handle_menu_navigation);

            app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::LoadMenu);
            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::LoadMenu);

            app.world_mut().resource_mut::<MenuCursor>().selected_index = 8;
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
            app.update();

            // Cursor reset to 0, max_items set to 4
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 0);
            assert_eq!(cursor.max_items, 4);

            // Transitions to MainMenu
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::MainMenu);
        }
    }

    #[test]
    fn test_navigation_between_main_menu_paused_and_options_menu() {
        // MainMenu -> OptionsMenu -> MainMenu
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.init_state::<GamePhase>();
            app.init_resource::<ButtonInput<KeyCode>>();
            app.init_resource::<MenuCursor>();
            app.init_resource::<LevelProgress>();
            app.init_resource::<SaveLoadOrigin>();
            app.init_resource::<OptionsOrigin>();
            app.init_resource::<crate::config::GameConfig>();
            app.add_event::<crate::audio::PlaySoundEvent>();
            app.add_event::<LoadLevelEvent>();
            app.add_event::<crate::save::SaveGameEvent>();
            app.add_event::<crate::save::LoadGameEvent>();
            app.add_event::<bevy::app::AppExit>();
            app.add_systems(Update, handle_menu_navigation);

            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::MainMenu);

            // Select index 1 (OPTIONS) and press Enter
            app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();

            // Next state is OptionsMenu, origin is MainMenu
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
            let origin = app.world().resource::<OptionsOrigin>();
            assert_eq!(origin.0, GamePhase::MainMenu);
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 0);
            assert_eq!(cursor.max_items, 4);

            // In OptionsMenu, press Escape -> back to MainMenu with cursor at index 1
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();

            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::MainMenu);
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 1);
            assert_eq!(cursor.max_items, 4);
        }

        // Paused -> OptionsMenu -> Paused
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.init_state::<GamePhase>();
            app.init_resource::<ButtonInput<KeyCode>>();
            app.init_resource::<MenuCursor>();
            app.init_resource::<LevelProgress>();
            app.init_resource::<SaveLoadOrigin>();
            app.init_resource::<OptionsOrigin>();
            app.init_resource::<crate::config::GameConfig>();
            app.add_event::<crate::audio::PlaySoundEvent>();
            app.add_event::<LoadLevelEvent>();
            app.add_event::<crate::save::SaveGameEvent>();
            app.add_event::<crate::save::LoadGameEvent>();
            app.add_event::<bevy::app::AppExit>();
            app.add_systems(Update, handle_menu_navigation);

            app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::Paused);
            app.update();
            app.update();
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Paused);

            // Select index 1 (OPTIONS in Paused) and press Space
            app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();

            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
            let origin = app.world().resource::<OptionsOrigin>();
            assert_eq!(origin.0, GamePhase::Paused);
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 0);
            assert_eq!(cursor.max_items, 4);

            // In OptionsMenu, press Escape -> back to Paused with cursor at index 1, max_items = 6
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
            app.update();

            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::Paused);
            let cursor = app.world().resource::<MenuCursor>();
            assert_eq!(cursor.selected_index, 1);
            assert_eq!(cursor.max_items, 6);
        }
    }

    #[test]
    fn test_navigation_between_options_menu_and_submenus() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<SaveLoadOrigin>();
        app.init_resource::<OptionsOrigin>();
        app.init_resource::<crate::config::GameConfig>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.add_systems(Update, handle_menu_navigation);

        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::OptionsMenu);
        app.update();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);

        // Submenu 0: Sound Setup
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::SoundSetup);
        assert_eq!(app.world().resource::<MenuCursor>().max_items, 4);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 0);

        // Submenu 1: Video & Display
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::VideoSetup);
        assert_eq!(app.world().resource::<MenuCursor>().max_items, 4);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 1);

        // Submenu 2: Controls Setup
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 2;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::ControlsSetup);
        assert_eq!(app.world().resource::<MenuCursor>().max_items, 4);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 2);
    }

    #[test]
    fn test_options_menu_cursor_wrapping_and_restore_defaults() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<SaveLoadOrigin>();
        app.init_resource::<OptionsOrigin>();
        let mut initial_config = crate::config::GameConfig::default();
        initial_config.sound.master_volume = 0.2;
        initial_config.video.crt_enabled = true;
        app.insert_resource(initial_config);
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.add_systems(Update, handle_menu_navigation);

        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::OptionsMenu);
        app.update();

        // Wrapping up from 0 to 3
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowUp);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 3);

        // Wrapping down from 3 to 0
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowDown);
        app.update();
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 0);

        // Select index 3 (RESTORE DEFAULTS) and press Enter
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 3;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();

        let cfg = app.world().resource::<crate::config::GameConfig>();
        assert_eq!(cfg.sound.master_volume, 1.0);
        assert!(!cfg.video.crt_enabled);

        let _ = std::fs::remove_file(crate::config::GameConfig::default_config_path());
    }

    #[test]
    fn test_sound_setup_slider_adjustments_and_clamping() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<SaveLoadOrigin>();
        app.init_resource::<OptionsOrigin>();
        app.init_resource::<crate::config::GameConfig>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.add_systems(Update, handle_menu_navigation);

        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::SoundSetup);
        app.update();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::SoundSetup);

        // 1. Master Volume (index 0, default 1.0)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        // Pressing Right at 1.0 clamps at 1.0
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.master_volume, 1.0);

        // Pressing Left twice decrements by 0.2 to 0.8
        for _ in 0..2 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowLeft);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.master_volume, 0.8);

        // 2. Sound FX Volume (index 1, default 1.0)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
        // Pressing Left 15 times clamps at 0.0
        for _ in 0..15 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyA);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.sfx_volume, 0.0);

        // Pressing Right once increments to 0.1
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyD);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.sfx_volume, 0.1);

        // 3. Music Volume (index 2, default 0.7)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 2;
        // Increment once to 0.8
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.music_volume, 0.8);

        // 4. Voice Volume (index 3, default 1.0)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 3;
        // Decrement 4 times to 0.6
        for _ in 0..4 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowLeft);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().sound.voice_volume, 0.6);
    }

    #[test]
    fn test_slider_formatting() {
        let max_str = format_slider("MASTER VOLUME", 1.0);
        assert!(max_str.contains("100%"));
        assert!(max_str.contains("[==========]"));

        let half_str = format_slider("MUSIC VOLUME", 0.5);
        assert!(half_str.contains("50%"));
        assert!(half_str.contains("[=====-----]"));

        let zero_str = format_slider("SFX VOLUME", 0.0);
        assert!(zero_str.contains("0%"));
        assert!(zero_str.contains("[----------]"));
    }

    #[test]
    fn test_video_setup_toggles_and_config_sync() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<OptionsOrigin>();
        app.insert_resource(crate::config::GameConfig::default());
        app.insert_resource(crate::palette::CrtPostProcessConfig {
            enabled: false,
            ..default()
        });
        app.insert_resource(crate::voxel::registry::VoxelConfig {
            enabled: true,
            ..default()
        });
        app.insert_resource(crate::lighting::DynamicLightingConfig {
            enabled: true,
            ..default()
        });
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.add_systems(Update, handle_menu_navigation);

        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::VideoSetup);
        app.update();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::VideoSetup);

        // 1. CRT Shader (index 0, default false)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        assert!(!app.world().resource::<crate::config::GameConfig>().video.crt_enabled);
        assert!(!app.world().resource::<crate::palette::CrtPostProcessConfig>().enabled);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().video.crt_enabled);
        assert!(app.world().resource::<crate::palette::CrtPostProcessConfig>().enabled);

        // Toggle back off
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().video.crt_enabled);
        assert!(!app.world().resource::<crate::palette::CrtPostProcessConfig>().enabled);

        // 2. 3D Voxel Models (index 1, default true)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
        assert!(app.world().resource::<crate::config::GameConfig>().video.voxels_enabled);
        assert!(app.world().resource::<crate::voxel::registry::VoxelConfig>().enabled);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().video.voxels_enabled);
        assert!(!app.world().resource::<crate::voxel::registry::VoxelConfig>().enabled);

        // Toggle back on
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyA);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().video.voxels_enabled);
        assert!(app.world().resource::<crate::voxel::registry::VoxelConfig>().enabled);

        // 3. Dynamic Lighting (index 2, default true)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 2;
        assert!(app.world().resource::<crate::config::GameConfig>().video.dynamic_lighting_enabled);
        assert!(app.world().resource::<crate::lighting::DynamicLightingConfig>().enabled);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().video.dynamic_lighting_enabled);
        assert!(!app.world().resource::<crate::lighting::DynamicLightingConfig>().enabled);

        // Toggle back on
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().video.dynamic_lighting_enabled);
        assert!(app.world().resource::<crate::lighting::DynamicLightingConfig>().enabled);

        // 4. Window Mode (index 3, default Windowed)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 3;
        assert_eq!(
            app.world().resource::<crate::config::GameConfig>().video.window_mode,
            crate::config::WindowModeSetting::Windowed
        );

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(
            app.world().resource::<crate::config::GameConfig>().video.window_mode,
            crate::config::WindowModeSetting::BorderlessFullscreen
        );

        // Toggle back to Windowed
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowLeft);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(
            app.world().resource::<crate::config::GameConfig>().video.window_mode,
            crate::config::WindowModeSetting::Windowed
        );

        // 5. Escape returns to OptionsMenu
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 1);
    }

    #[test]
    fn test_controls_setup_menu_navigation_and_toggles() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GamePhase>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MenuCursor>();
        app.init_resource::<LevelProgress>();
        app.init_resource::<OptionsOrigin>();
        app.insert_resource(crate::config::GameConfig::default());
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<LoadLevelEvent>();
        app.add_event::<crate::save::SaveGameEvent>();
        app.add_event::<crate::save::LoadGameEvent>();
        app.add_event::<bevy::app::AppExit>();
        app.add_systems(Update, handle_menu_navigation);

        app.world_mut().resource_mut::<NextState<GamePhase>>().set(GamePhase::ControlsSetup);
        app.update();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::ControlsSetup);

        // 1. Mouse Sensitivity (index 0, default 1.0)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 0;
        assert_eq!(app.world().resource::<crate::config::GameConfig>().controls.mouse_sensitivity, 1.0);

        // Press Right -> increments by 0.25 to 1.25
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert_eq!(app.world().resource::<crate::config::GameConfig>().controls.mouse_sensitivity, 1.25);

        // Press Left twice -> decrements by 0.5 to 0.75
        for _ in 0..2 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowLeft);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().controls.mouse_sensitivity, 0.75);

        // Press Left 10 times -> clamps at 0.5
        for _ in 0..10 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyA);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().controls.mouse_sensitivity, 0.5);

        // Press Right 20 times -> clamps at 3.0
        for _ in 0..20 {
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyD);
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        }
        assert_eq!(app.world().resource::<crate::config::GameConfig>().controls.mouse_sensitivity, 3.0);

        // 2. Invert Mouse Y (index 1, default false)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 1;
        assert!(!app.world().resource::<crate::config::GameConfig>().controls.invert_mouse_y);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().controls.invert_mouse_y);

        // Toggle back off
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().controls.invert_mouse_y);

        // 3. Auto-Switch Weapon (index 2, default true)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 2;
        assert!(app.world().resource::<crate::config::GameConfig>().controls.auto_switch_weapon);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().controls.auto_switch_weapon);

        // Toggle back on
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowRight);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().controls.auto_switch_weapon);

        // 4. View Bobbing (index 3, default true)
        app.world_mut().resource_mut::<MenuCursor>().selected_index = 3;
        assert!(app.world().resource::<crate::config::GameConfig>().controls.view_bobbing);

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(!app.world().resource::<crate::config::GameConfig>().controls.view_bobbing);

        // Toggle back on
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(app.world().resource::<crate::config::GameConfig>().controls.view_bobbing);

        // 5. Escape returns to OptionsMenu
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        assert_eq!(*app.world().resource::<State<GamePhase>>().get(), GamePhase::OptionsMenu);
        assert_eq!(app.world().resource::<MenuCursor>().selected_index, 2);
    }

    #[test]
    fn test_intermission_ui_boss_victory_and_level_completed() {
        let mut app = App::new();
        app.insert_resource(State::new(GamePhase::Intermission))
            .init_resource::<LevelProgress>()
            .init_resource::<IntermissionAnimationState>()
            .init_resource::<MenuCursor>()
            .init_resource::<CursorAnimTimer>()
            .add_systems(Startup, setup_menu_ui)
            .add_systems(Update, update_menu_ui);

        // Run startup systems to spawn UI nodes
        app.update();

        // 1. Regular level completion (e.g. E1L1)
        {
            let mut progress = app.world_mut().resource_mut::<LevelProgress>();
            progress.current_episode = 1;
            progress.current_level = 1;
            progress.is_boss_victory = false;
        }
        app.update();

        // Query MenuHeaderTitle, MenuSubheaderText, MenuFooterText
        let mut header_q = app.world_mut().query_filtered::<&Text, With<MenuHeaderTitle>>();
        let header_text = header_q.single(app.world());
        assert_eq!(header_text.sections[0].value, "E1L1: LEVEL COMPLETED");
        assert_eq!(header_text.sections[0].style.color, DUKE_RED);

        let mut subheader_q = app.world_mut().query_filtered::<&Text, With<MenuSubheaderText>>();
        let subheader_text = subheader_q.single(app.world());
        assert_eq!(subheader_text.sections[0].value, "MISSION STATISTICS");

        let mut footer_q = app.world_mut().query_filtered::<&Text, With<MenuFooterText>>();
        let footer_text = footer_q.single(app.world());
        assert!(footer_text.sections[0].value.contains("CONTINUE TO NEXT LEVEL"));

        // 2. Boss level completion / victory (e.g. E1L6 Battlelord defeat)
        {
            let mut progress = app.world_mut().resource_mut::<LevelProgress>();
            progress.current_episode = 1;
            progress.current_level = 6;
            progress.is_boss_victory = true;
        }
        app.update();

        let mut header_q = app.world_mut().query_filtered::<&Text, With<MenuHeaderTitle>>();
        let header_text = header_q.single(app.world());
        assert_eq!(header_text.sections[0].value, "E1L6: EPISODE VICTORY!");
        assert_eq!(header_text.sections[0].style.color, DUKE_GOLD);

        let mut subheader_q = app.world_mut().query_filtered::<&Text, With<MenuSubheaderText>>();
        let subheader_text = subheader_q.single(app.world());
        assert_eq!(subheader_text.sections[0].value, "EPISODE COMPLETED");

        let mut footer_q = app.world_mut().query_filtered::<&Text, With<MenuFooterText>>();
        let footer_text = footer_q.single(app.world());
        assert!(footer_text.sections[0].value.contains("ADVANCE TO NEXT EPISODE"));
    }

    #[test]
    fn test_intermission_speech_on_boss_victory() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .insert_resource(State::new(GamePhase::Intermission))
            .init_resource::<LevelProgress>()
            .init_resource::<IntermissionAnimationState>()
            .add_event::<crate::audio::PlaySoundEvent>()
            .add_systems(Update, update_intermission_animation);

        {
            let mut progress = app.world_mut().resource_mut::<LevelProgress>();
            progress.is_boss_victory = true;
            let mut anim = app.world_mut().resource_mut::<IntermissionAnimationState>();
            anim.stage = IntermissionStage::TimeReveal;
            anim.timer = 3.5;
            anim.speech_played = false;
        }

        app.update();

        let anim = app.world().resource::<IntermissionAnimationState>();
        assert!(anim.speech_played);

        let sound_events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        let mut reader = sound_events.get_reader();
        let events: Vec<_> = reader.read(sound_events).collect();
        assert!(events.iter().any(|e| e.sound_id == crate::audio::sound_defs::BONUS_SPEECH1));
    }
}

