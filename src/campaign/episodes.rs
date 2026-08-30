#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampaignMapInfo {
    pub episode: usize,
    pub level: usize,
    pub title: &'static str,
    pub map_filename: &'static str,
    pub music_filename: &'static str,
    pub par_time_seconds: f32,
    pub is_secret: bool,
    pub secret_destination: Option<usize>,
    pub is_boss_level: bool,
}

pub const ALL_CAMPAIGN_MAPS: &[CampaignMapInfo] = &[
    // --- Episode 1: L.A. Meltdown ---
    CampaignMapInfo {
        episode: 1, level: 1, title: "Hollywood Holocaust", map_filename: "E1L1.MAP",
        music_filename: "STALKER.MID", par_time_seconds: 90.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 1, level: 2, title: "Red Light District", map_filename: "E1L2.MAP",
        music_filename: "DETHTOLL.MID", par_time_seconds: 190.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 1, level: 3, title: "Death Row", map_filename: "E1L3.MAP",
        music_filename: "STREETS.MID", par_time_seconds: 225.0, is_secret: false,
        secret_destination: Some(8), is_boss_level: false, // Secret exit to E1L8
    },
    CampaignMapInfo {
        episode: 1, level: 4, title: "Toxic Dump", map_filename: "E1L4.MAP",
        music_filename: "WATRWRD1.MID", par_time_seconds: 255.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 1, level: 5, title: "The Abyss", map_filename: "E1L5.MAP",
        music_filename: "SNAKE1.MID", par_time_seconds: 310.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 1, level: 6, title: "Launch Facility", map_filename: "E1L6.MAP",
        music_filename: "LORDCOFO.MID", par_time_seconds: 240.0, is_secret: false,
        secret_destination: None, is_boss_level: true, // Battlelord Finale
    },
    CampaignMapInfo {
        episode: 1, level: 7, title: "Faces of Death", map_filename: "E1L7.MAP",
        music_filename: "STALKER.MID", par_time_seconds: 120.0, is_secret: false,
        secret_destination: None, is_boss_level: false, // Multiplayer Arena
    },
    CampaignMapInfo {
        episode: 1, level: 8, title: "Launch Facility (Secret)", map_filename: "E1L8.MAP",
        music_filename: "LORDCOFO.MID", par_time_seconds: 210.0, is_secret: true,
        secret_destination: None, is_boss_level: false, // Returns to E1L4
    },

    // --- Episode 2: Lunar Apocalypse ---
    CampaignMapInfo {
        episode: 2, level: 1, title: "Spaceport", map_filename: "E2L1.MAP",
        music_filename: "ALF.MID", par_time_seconds: 165.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 2, title: "Incubator", map_filename: "E2L2.MAP",
        music_filename: "SPACE.MID", par_time_seconds: 220.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 3, title: "Warp Factor", map_filename: "E2L3.MAP",
        music_filename: "GUTWREN.MID", par_time_seconds: 210.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 4, title: "Fusion Station", map_filename: "E2L4.MAP",
        music_filename: "INVADER.MID", par_time_seconds: 180.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 5, title: "Occupied Territory", map_filename: "E2L5.MAP",
        music_filename: "GOTH.MID", par_time_seconds: 170.0, is_secret: false,
        secret_destination: Some(10), is_boss_level: false, // Secret exit to E2L10
    },
    CampaignMapInfo {
        episode: 2, level: 6, title: "Tiberius Station", map_filename: "E2L6.MAP",
        music_filename: "2048.MID", par_time_seconds: 195.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 7, title: "Lunar Reactor", map_filename: "E2L7.MAP",
        music_filename: "THECALL.MID", par_time_seconds: 215.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 8, title: "Dark Side", map_filename: "E2L8.MAP",
        music_filename: "WHOM.MID", par_time_seconds: 240.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 2, level: 9, title: "Overlord", map_filename: "E2L9.MAP",
        music_filename: "LORDCOFO.MID", par_time_seconds: 270.0, is_secret: false,
        secret_destination: None, is_boss_level: true, // Overlord Boss Finale
    },
    CampaignMapInfo {
        episode: 2, level: 10, title: "Spin Cycle (Secret)", map_filename: "E2L10.MAP",
        music_filename: "INVADER.MID", par_time_seconds: 120.0, is_secret: true,
        secret_destination: None, is_boss_level: false, // Returns to E2L6
    },
    CampaignMapInfo {
        episode: 2, level: 11, title: "Lunatic Fringe (Secret)", map_filename: "E2L11.MAP",
        music_filename: "SPACE.MID", par_time_seconds: 135.0, is_secret: true,
        secret_destination: None, is_boss_level: false,
    },

    // --- Episode 3: Shrapnel City ---
    CampaignMapInfo {
        episode: 3, level: 1, title: "Raw Meat", map_filename: "E3L1.MAP",
        music_filename: "STALKER.MID", par_time_seconds: 120.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 2, title: "Bank Roll", map_filename: "E3L2.MAP",
        music_filename: "DETHTOLL.MID", par_time_seconds: 180.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 3, title: "Flood Zone", map_filename: "E3L3.MAP",
        music_filename: "STREETS.MID", par_time_seconds: 220.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 4, title: "L.A. Rumble", map_filename: "E3L4.MAP",
        music_filename: "WATRWRD1.MID", par_time_seconds: 190.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 5, title: "Movie Set", map_filename: "E3L5.MAP",
        music_filename: "SNAKE1.MID", par_time_seconds: 160.0, is_secret: false,
        secret_destination: Some(10), is_boss_level: false, // Secret exit to E3L10
    },
    CampaignMapInfo {
        episode: 3, level: 6, title: "Rabid Transit", map_filename: "E3L6.MAP",
        music_filename: "LORDCOFO.MID", par_time_seconds: 195.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 7, title: "Fahrenheit", map_filename: "E3L7.MAP",
        music_filename: "ALF.MID", par_time_seconds: 210.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 8, title: "Hotel Hell", map_filename: "E3L8.MAP",
        music_filename: "SPACE.MID", par_time_seconds: 225.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 3, level: 9, title: "Stadium", map_filename: "E3L9.MAP",
        music_filename: "GUTWREN.MID", par_time_seconds: 150.0, is_secret: false,
        secret_destination: None, is_boss_level: true, // Cycloid Emperor Finale
    },
    CampaignMapInfo {
        episode: 3, level: 10, title: "Tier Drops (Secret)", map_filename: "E3L10.MAP",
        music_filename: "STREETS.MID", par_time_seconds: 165.0, is_secret: true,
        secret_destination: None, is_boss_level: false, // Returns to E3L6
    },
    CampaignMapInfo {
        episode: 3, level: 11, title: "Freeway (Secret)", map_filename: "E3L11.MAP",
        music_filename: "WATRWRD1.MID", par_time_seconds: 135.0, is_secret: true,
        secret_destination: None, is_boss_level: false,
    },

    // --- Episode 4: The Birth (Atomic Edition) ---
    CampaignMapInfo {
        episode: 4, level: 1, title: "It's Impossible", map_filename: "E4L1.MAP",
        music_filename: "GOINGDN.MID", par_time_seconds: 165.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 2, title: "Duke-Burger", map_filename: "E4L2.MAP",
        music_filename: "BRIEFING.MID", par_time_seconds: 210.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 3, title: "Shop-N-Bag", map_filename: "E4L3.MAP",
        music_filename: "WAREHSE.MID", par_time_seconds: 195.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 4, title: "Babe Land", map_filename: "E4L4.MAP",
        music_filename: "BABES.MID", par_time_seconds: 270.0, is_secret: false,
        secret_destination: Some(11), is_boss_level: false, // Secret exit to E4L11
    },
    CampaignMapInfo {
        episode: 4, level: 5, title: "Pigsty", map_filename: "E4L5.MAP",
        music_filename: "PIGSTY.MID", par_time_seconds: 200.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 6, title: "Going Postal", map_filename: "E4L6.MAP",
        music_filename: "POSTAL.MID", par_time_seconds: 230.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 7, title: "XXX-Stacy", map_filename: "E4L7.MAP",
        music_filename: "STACY.MID", par_time_seconds: 220.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 8, title: "Critical Mass", map_filename: "E4L8.MAP",
        music_filename: "CRITICAL.MID", par_time_seconds: 240.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 9, title: "Derelict", map_filename: "E4L9.MAP",
        music_filename: "DERELICT.MID", par_time_seconds: 255.0, is_secret: false,
        secret_destination: None, is_boss_level: false,
    },
    CampaignMapInfo {
        episode: 4, level: 10, title: "The Queen", map_filename: "E4L10.MAP",
        music_filename: "QUEEN.MID", par_time_seconds: 285.0, is_secret: false,
        secret_destination: None, is_boss_level: true, // Queen Boss Finale
    },
    CampaignMapInfo {
        episode: 4, level: 11, title: "Area 51 (Secret)", map_filename: "E4L11.MAP",
        music_filename: "GOINGDN.MID", par_time_seconds: 210.0, is_secret: true,
        secret_destination: None, is_boss_level: false, // Returns to E4L5
    },
];

pub fn get_campaign_map(episode: usize, level: usize) -> Option<&'static CampaignMapInfo> {
    ALL_CAMPAIGN_MAPS
        .iter()
        .find(|m| m.episode == episode && m.level == level)
}

pub fn get_episode_level_count(episode: usize) -> usize {
    match episode {
        1 => 6,  // E1L1..E1L6 (Finale)
        2 => 9,  // E2L1..E2L9 (Finale)
        3 => 9,  // E3L1..E3L9 (Finale)
        4 => 10, // E4L1..E4L10 (Finale)
        _ => 6,
    }
}

use bevy::prelude::*;
use crate::scripting::types::*;
use crate::scripting::compiler::CompiledScript;

#[derive(Resource, Debug, Clone, Default)]
pub struct CampaignRegistry {
    pub volumes: Vec<DynamicVolumeDef>,
    pub skills: Vec<DynamicSkillDef>,
    pub levels: Vec<DynamicLevelDef>,
    pub quotes: std::collections::HashMap<i32, String>,
}

impl CampaignRegistry {
    pub fn from_compiled_script(script: &CompiledScript) -> Self {
        Self {
            volumes: script.volumes.clone(),
            skills: script.skills.clone(),
            levels: script.levels.clone(),
            quotes: script.quotes.clone(),
        }
    }

    pub fn get_level_info(&self, volume: usize, level: usize) -> Option<&DynamicLevelDef> {
        self.levels.iter().find(|l| l.volume == volume && l.level == level)
    }

    pub fn get_volume_title(&self, volume: usize) -> Option<&str> {
        self.volumes.iter().find(|v| v.volume_id == volume).map(|v| v.title.as_str())
    }

    pub fn get_skill_title(&self, skill_id: usize) -> Option<&str> {
        self.skills.iter().find(|s| s.skill_id == skill_id).map(|s| s.title.as_str())
    }

    pub fn get_level_map_filename(&self, volume: usize, level: usize) -> Option<&str> {
        self.get_level_info(volume, level).map(|l| l.filename.as_str())
    }

    pub fn get_quote(&self, quote_id: i32) -> Option<&str> {
        self.quotes.get(&quote_id).map(|s| s.as_str())
    }
}
