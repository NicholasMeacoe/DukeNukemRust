#![allow(dead_code)]

use bevy::prelude::*;
use crate::game_flow::state::GamePhase;
use crate::player::types::PlayerController;

#[derive(Resource, Debug, Clone, Default)]
pub struct CheatState {
    pub buffer: String,
    pub god_mode: bool,
    pub no_clip: bool,
    pub show_all_map: bool,
    pub cashman: bool,
    pub show_coords: bool,
    pub show_fps: bool,
    pub monsters_disabled: bool,
}

pub struct CheatsPlugin;

impl Plugin for CheatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CheatState>()
            .add_systems(Update, handle_cheat_input.run_if(in_state(GamePhase::Playing)));
    }
}

pub fn handle_cheat_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut cheat_state: ResMut<CheatState>,
    mut player_query: Query<&mut PlayerController, With<crate::Player>>,
    mut progress: ResMut<crate::game_flow::LevelProgress>,
    mut sbar_query: Query<&mut crate::hud::StatusbarState>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    let Ok(mut player) = player_query.get_single_mut() else { return; };

    for &key in keys.get_just_pressed() {
        if let Some(ch) = key_to_char(key) {
            cheat_state.buffer.push(ch);
            if cheat_state.buffer.len() > 15 {
                cheat_state.buffer.remove(0);
            }

            // Check matching cheats
            let buf = cheat_state.buffer.clone();
            if let Some(msg) = evaluate_cheats(&buf, &mut cheat_state, &mut player, &mut progress) {
                for mut sbar in sbar_query.iter_mut() {
                    sbar.message_text = msg.to_string();
                    sbar.message_timer = 3.0;
                }
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 }); // Pistol click confirmation
                cheat_state.buffer.clear();
            }
        }
    }
}

pub fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 | KeyCode::Numpad0 => Some('0'),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some('1'),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some('2'),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some('3'),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some('4'),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some('5'),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some('6'),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some('7'),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some('8'),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some('9'),
        _ => None,
    }
}

pub fn evaluate_cheats(
    buffer: &str,
    cheat_state: &mut CheatState,
    player: &mut PlayerController,
    progress: &mut crate::game_flow::LevelProgress,
) -> Option<&'static str> {
    if buffer.ends_with("dnkroz") || buffer.ends_with("dncornholio") {
        cheat_state.god_mode = !cheat_state.god_mode;
        player.health = 100;
        return Some(if cheat_state.god_mode { "GOD MODE ON" } else { "GOD MODE OFF" });
    }

    if buffer.ends_with("dnstuff") {
        for (i, w) in player.weapons.iter_mut().enumerate() {
            w.is_unlocked = true;
            w.ammo = match i {
                0 => 0,      // Foot
                1 => 200,    // Pistol
                2 => 50,     // Shotgun
                3 => 200,    // Chaingun
                4 => 50,     // RPG
                5 => 50,     // Pipebomb
                6 => 50,     // Shrinker
                7 => 99,     // Devastator
                8 => 10,     // Tripbomb
                9 => 99,     // Freezethrower
                10 => 0,     // HandRemote
                11 => 50,    // Expander
                _ => 50,
            };
        }
        player.armor = 100;
        player.has_blue_key = true;
        player.has_red_key = true;
        player.has_yellow_key = true;
        player.inventory.steroids_amount = 400;
        player.inventory.medkit_amount = 100;
        player.inventory.nightvision_amount = 1200;
        player.inventory.scuba_amount = 6400;
        player.inventory.boots_amount = 200;
        player.inventory.holoduke_amount = 2400;
        player.inventory.jetpack_amount = 1600;
        return Some("ALL WEAPONS, ITEMS, KEYS");
    }

    if buffer.ends_with("dnitems") {
        player.armor = 100;
        player.has_blue_key = true;
        player.has_red_key = true;
        player.has_yellow_key = true;
        player.inventory.steroids_amount = 400;
        player.inventory.medkit_amount = 100;
        player.inventory.nightvision_amount = 1200;
        player.inventory.scuba_amount = 6400;
        player.inventory.boots_amount = 200;
        player.inventory.holoduke_amount = 2400;
        player.inventory.jetpack_amount = 1600;
        return Some("ALL ITEMS AND KEYS");
    }

    if buffer.ends_with("dnclip") {
        cheat_state.no_clip = !cheat_state.no_clip;
        return Some(if cheat_state.no_clip { "NO CLIPPING ON" } else { "NO CLIPPING OFF" });
    }

    if buffer.ends_with("dncashman") {
        cheat_state.cashman = !cheat_state.cashman;
        return Some(if cheat_state.cashman { "CASHMAN ON" } else { "CASHMAN OFF" });
    }

    if buffer.ends_with("dnhyper") {
        player.inventory.steroids_amount = 400;
        player.inventory.steroids_active = true;
        return Some("HYPER STEROIDS ACTIVE");
    }

    if buffer.ends_with("dnshowmap") {
        cheat_state.show_all_map = !cheat_state.show_all_map;
        return Some(if cheat_state.show_all_map { "SHOW ALL MAP ON" } else { "SHOW ALL MAP OFF" });
    }

    if buffer.ends_with("dnmonsters") {
        cheat_state.monsters_disabled = !cheat_state.monsters_disabled;
        return Some(if cheat_state.monsters_disabled { "MONSTERS OFF" } else { "MONSTERS ON" });
    }

    if buffer.ends_with("dnrate") {
        cheat_state.show_fps = !cheat_state.show_fps;
        return Some(if cheat_state.show_fps { "TICK RATE ON" } else { "TICK RATE OFF" });
    }

    if buffer.ends_with("dncoords") {
        cheat_state.show_coords = !cheat_state.show_coords;
        return Some(if cheat_state.show_coords { "SHOW COORDS ON" } else { "SHOW COORDS OFF" });
    }

    if buffer.ends_with("dnkeys") {
        player.has_blue_key = true;
        player.has_red_key = true;
        player.has_yellow_key = true;
        return Some("ALL KEYS GIVEN");
    }

    if buffer.ends_with("dnweapons") {
        for (i, w) in player.weapons.iter_mut().enumerate() {
            w.is_unlocked = true;
            w.ammo = match i {
                0 => 0, 1 => 200, 2 => 50, 3 => 200, 4 => 50,
                5 => 50, 6 => 50, 7 => 99, 8 => 10, 9 => 99,
                10 => 0, 11 => 50, _ => 50,
            };
        }
        return Some("ALL WEAPONS GIVEN");
    }

    if buffer.ends_with("dninventory") {
        player.inventory.steroids_amount = 400;
        player.inventory.medkit_amount = 100;
        player.inventory.nightvision_amount = 1200;
        player.inventory.scuba_amount = 6400;
        player.inventory.boots_amount = 200;
        player.inventory.holoduke_amount = 2400;
        player.inventory.jetpack_amount = 1600;
        return Some("ALL INVENTORY GIVEN");
    }

    // Dynamic wildcard cheats: dnskill1..dnskill4
    if buffer.len() >= 8 && buffer.contains("dnskill") {
        if let Some(digit) = buffer.chars().last().filter(|c| c.is_ascii_digit()) {
            let skill_num = digit.to_digit(10).unwrap_or(1).clamp(1, 4);
            progress.skill = match skill_num {
                1 => crate::game_flow::SkillLevel::PieceOfCake,
                2 => crate::game_flow::SkillLevel::LetsRock,
                3 => crate::game_flow::SkillLevel::ComeGetSome,
                4 => crate::game_flow::SkillLevel::DamnImGood,
                _ => crate::game_flow::SkillLevel::LetsRock,
            };
            return Some("SKILL LEVEL CHANGED");
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cheat_code_parsing_and_evaluation() {
        let mut cheat_state = CheatState::default();
        let mut player = PlayerController::default();
        let mut progress = crate::game_flow::LevelProgress::default();

        // 1. God mode
        let res = evaluate_cheats("dnkroz", &mut cheat_state, &mut player, &mut progress);
        assert_eq!(res, Some("GOD MODE ON"));
        assert!(cheat_state.god_mode);
        assert_eq!(player.health, 100);

        let res2 = evaluate_cheats("dnkroz", &mut cheat_state, &mut player, &mut progress);
        assert_eq!(res2, Some("GOD MODE OFF"));
        assert!(!cheat_state.god_mode);

        // 2. All items/weapons (dnstuff)
        let res3 = evaluate_cheats("dnstuff", &mut cheat_state, &mut player, &mut progress);
        assert_eq!(res3, Some("ALL WEAPONS, ITEMS, KEYS"));
        assert_eq!(player.weapons[1].ammo, 200); // Pistol ammo
        assert_eq!(player.weapons[2].ammo, 50);  // Shotgun ammo
        assert_eq!(player.weapons[4].ammo, 50);  // RPG ammo
        assert!(player.has_blue_key);
        assert!(player.has_red_key);
        assert!(player.has_yellow_key);
        assert_eq!(player.inventory.jetpack_amount, 1600);

        // 3. No clip
        let res4 = evaluate_cheats("dnclip", &mut cheat_state, &mut player, &mut progress);
        assert_eq!(res4, Some("NO CLIPPING ON"));
        assert!(cheat_state.no_clip);

        // 4. Wildcard skill change (dnskill3)
        let res5 = evaluate_cheats("dnskill3", &mut cheat_state, &mut player, &mut progress);
        assert_eq!(res5, Some("SKILL LEVEL CHANGED"));
        assert_eq!(progress.skill, crate::game_flow::SkillLevel::ComeGetSome);
    }
}
