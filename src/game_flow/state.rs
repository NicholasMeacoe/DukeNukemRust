use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GamePhase {
    #[default]
    MainMenu,
    EpisodeSelect,
    SkillSelect,
    Playing,
    Intermission,
    Paused,
    SaveMenu,
    LoadMenu,
    OptionsMenu,
    SoundSetup,
    VideoSetup,
    ControlsSetup,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveLoadOrigin(pub GamePhase);

impl Default for SaveLoadOrigin {
    fn default() -> Self {
        Self(GamePhase::Paused)
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionsOrigin(pub GamePhase);

impl Default for OptionsOrigin {
    fn default() -> Self {
        Self(GamePhase::MainMenu)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SkillLevel {
    PieceOfCake = 0,
    LetsRock = 1,
    ComeGetSome = 2,
    DamnImGood = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Episode {
    LAMeltdown = 1,
    LunarApocalypse = 2,
    ShrapnelCity = 3,
    ThePlutoniumPak = 4,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct FoundSecretSectors(pub std::collections::HashSet<usize>);

#[derive(Resource, Debug, Clone)]
pub struct LevelProgress {
    pub current_episode: usize,
    pub current_level: usize,
    pub skill: SkillLevel,
    pub kills_count: i32,
    pub total_monsters: i32,
    pub secrets_found: i32,
    pub total_secrets: i32,
    pub level_time_seconds: f32,
    pub par_time_seconds: f32,
    pub is_level_completed: bool,
    pub is_boss_victory: bool,
    pub is_secret_exit: bool,
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            current_episode: 1,
            current_level: 1,
            skill: SkillLevel::LetsRock,
            kills_count: 0,
            total_monsters: 50,
            secrets_found: 0,
            total_secrets: 4,
            level_time_seconds: 0.0,
            par_time_seconds: 180.0, // 3 minutes standard par
            is_level_completed: false,
            is_boss_victory: false,
            is_secret_exit: false,
        }
    }
}

impl LevelProgress {
    pub fn current_map_filename(&self) -> String {
        format!("E{}L{}.MAP", self.current_episode, self.current_level)
    }

    pub fn advance_to_next_level(&mut self) -> bool {
        let is_boss_victory = self.is_boss_victory;
        let is_secret_exit = self.is_secret_exit;
        let current_ep = self.current_episode;
        let current_lvl = self.current_level;

        // Reset per-level stats
        self.kills_count = 0;
        self.secrets_found = 0;
        self.level_time_seconds = 0.0;
        self.is_level_completed = false;
        self.is_boss_victory = false;
        self.is_secret_exit = false;

        // 1. Secret Level Branching
        if is_secret_exit {
            if let Some(map_info) = crate::campaign::episodes::find_campaign_map(current_ep, current_lvl) {
                if let Some(dest) = map_info.secret_destination {
                    self.current_level = dest;
                    return false;
                }
            }
        }

        // 2. Returning from a Secret Level
        let is_current_secret = crate::campaign::episodes::find_campaign_map(current_ep, current_lvl)
            .map(|m| m.is_secret)
            .unwrap_or(false);

        if is_current_secret {
            let return_level = match (current_ep, current_lvl) {
                (1, 8) => 4,   // E1L8 -> E1L4
                (2, 10) => 6,  // E2L10 -> E2L6
                (2, 11) => 9,  // E2L11 -> E2L9
                (3, 10) => 6,  // E3L10 -> E3L6
                (3, 11) => 9,  // E3L11 -> E3L9
                (4, 11) => 6,  // E4L11 -> E4L6
                _ => current_lvl + 1,
            };
            self.current_level = return_level;
            return false;
        }

        // 3. Boss Victory or Standard Campaign Progression
        let max_level = match current_ep {
            1 => 7,
            2 => 9,
            3 => 9,
            4 => 10,
            _ => 7,
        };

        if is_boss_victory || current_lvl >= max_level {
            self.current_level = 1;
            true // Episode completed!
        } else {
            self.current_level += 1;
            false
        }
    }
}

#[derive(Event, Debug, Clone, Default, PartialEq)]
pub struct LevelCompletedEvent {
    pub is_secret: bool,
    pub is_boss_victory: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LevelEntity;

#[derive(Event, Debug, Clone)]
pub struct LoadLevelEvent {
    pub episode: usize,
    pub level: usize,
}

pub fn update_secret_detection(
    player_query: Query<&crate::sector_map::CurrentSector, With<crate::player::PlayerController>>,
    sector_map: Option<Res<crate::sector_map::SectorMap>>,
    mut progress: ResMut<LevelProgress>,
    mut found_secrets: Option<ResMut<FoundSecretSectors>>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
    mut voice_events: EventWriter<crate::audio::PlayDukeVoiceEvent>,
    mut tint: Option<ResMut<crate::hud::ScreenTintState>>,
    mut sbar_query: Query<&mut crate::hud::StatusbarState>,
) {
    let Ok(current_sector) = player_query.get_single() else {
        return;
    };
    if current_sector.0 < 0 {
        return;
    }
    let sector_idx = current_sector.0 as usize;

    let Some(ref sm) = sector_map else {
        return;
    };
    if sm.get_sector_lotag(sector_idx) == 32767 {
        if let Some(ref mut found) = found_secrets {
            if found.0.insert(sector_idx) {
                progress.secrets_found += 1;
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 88 }); // SECRET_AREA
                voice_events.send(crate::audio::PlayDukeVoiceEvent {
                    name: Some("COOL01".into()),
                });
                if let Some(ref mut t) = tint {
                    t.target_color = Color::srgba(1.0, 1.0, 1.0, 0.6); // Flash screen white
                }
                if let Ok(mut sb) = sbar_query.get_single_mut() {
                    sb.message_text = "SECRET AREA FOUND!".to_string();
                    sb.message_timer = 3.0;
                }
            }
        }
    }
}
