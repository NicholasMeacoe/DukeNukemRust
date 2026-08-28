#![allow(dead_code)]

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillLevel {
    PieceOfCake = 0,
    LetsRock = 1,
    ComeGetSome = 2,
    DamnImGood = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Episode {
    LAMeltdown = 1,
    LunarApocalypse = 2,
    ShrapnelCity = 3,
    ThePlutoniumPak = 4,
}

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
        }
    }
}

impl LevelProgress {
    pub fn current_map_filename(&self) -> String {
        format!("E{}L{}.MAP", self.current_episode, self.current_level)
    }

    pub fn advance_to_next_level(&mut self) -> bool {
        self.current_level += 1;
        self.kills_count = 0;
        self.secrets_found = 0;
        self.level_time_seconds = 0.0;
        self.is_level_completed = false;

        // Episode 1 has 7 standard levels (E1L1..E1L7: Faces of Death)
        if self.current_level > 7 {
            self.current_level = 1;
            true // Episode completed!
        } else {
            false
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct LevelCompletedEvent;
