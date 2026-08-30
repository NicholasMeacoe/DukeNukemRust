#![allow(dead_code)]

use bevy::prelude::*;
use crate::campaign::episodes::*;
use crate::game_flow::state::SkillLevel;

#[derive(Resource, Debug, Clone)]
pub struct CampaignProgression {
    pub current_episode: usize,
    pub current_level: usize,
    pub skill: SkillLevel,
    pub from_bonus_level: Option<usize>,
    pub kills_count: i32,
    pub total_monsters: i32,
    pub secrets_found: i32,
    pub total_secrets: i32,
    pub level_time_seconds: f32,
    pub episode_completed: bool,
}

impl Default for CampaignProgression {
    fn default() -> Self {
        Self {
            current_episode: 1,
            current_level: 1,
            skill: SkillLevel::LetsRock,
            from_bonus_level: None,
            kills_count: 0,
            total_monsters: 50,
            secrets_found: 0,
            total_secrets: 4,
            level_time_seconds: 0.0,
            episode_completed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelAdvanceResult {
    NextLevel { episode: usize, level: usize },
    SecretLevel { episode: usize, level: usize },
    ReturnFromSecret { episode: usize, level: usize },
    EpisodeCompleted { episode: usize },
}

impl CampaignProgression {
    pub fn current_map_info(&self) -> Option<&'static CampaignMapInfo> {
        get_campaign_map(self.current_episode, self.current_level)
    }

    pub fn current_map_filename(&self) -> String {
        format!("E{}L{}.MAP", self.current_episode, self.current_level)
    }

    pub fn advance_level(&mut self, is_secret_exit: bool) -> LevelAdvanceResult {
        let current_map = self.current_map_info();
        let is_boss = current_map.map_or(false, |m| m.is_boss_level);

        if is_boss {
            self.episode_completed = true;
            return LevelAdvanceResult::EpisodeCompleted { episode: self.current_episode };
        }

        if is_secret_exit {
            if let Some(map) = current_map {
                if let Some(secret_dest) = map.secret_destination {
                    self.from_bonus_level = Some(self.current_level + 1);
                    self.current_level = secret_dest;
                    self.reset_level_stats();
                    return LevelAdvanceResult::SecretLevel {
                        episode: self.current_episode,
                        level: secret_dest,
                    };
                }
            }
        }

        if let Some(return_level) = self.from_bonus_level.take() {
            self.current_level = return_level;
            self.reset_level_stats();
            return LevelAdvanceResult::ReturnFromSecret {
                episode: self.current_episode,
                level: return_level,
            };
        }

        let max_level = get_episode_level_count(self.current_episode);
        if self.current_level >= max_level {
            self.episode_completed = true;
            LevelAdvanceResult::EpisodeCompleted { episode: self.current_episode }
        } else {
            self.current_level += 1;
            self.reset_level_stats();
            LevelAdvanceResult::NextLevel {
                episode: self.current_episode,
                level: self.current_level,
            }
        }
    }

    pub fn reset_level_stats(&mut self) {
        self.kills_count = 0;
        self.secrets_found = 0;
        self.level_time_seconds = 0.0;
        self.episode_completed = false;
    }
}
