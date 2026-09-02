#![allow(dead_code)]

use crate::net::protocol::NetMode;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct DukematchState {
    pub mode: NetMode,
    pub local_player_id: u8,
    pub player_names: [String; 8],
    pub player_colors: [u8; 8],
    pub frags: [[i32; 8]; 8],
    pub ping_ms: [u32; 8],
    pub kill_limit: i32,
    pub time_limit_sec: f32,
    pub match_timer: f32,
    pub show_scoreboard: bool,
}

impl Default for DukematchState {
    fn default() -> Self {
        Self {
            mode: NetMode::SinglePlayer,
            local_player_id: 0,
            player_names: [
                "DUKE".to_string(),
                "PLAYER 2".to_string(),
                "PLAYER 3".to_string(),
                "PLAYER 4".to_string(),
                "PLAYER 5".to_string(),
                "PLAYER 6".to_string(),
                "PLAYER 7".to_string(),
                "PLAYER 8".to_string(),
            ],
            player_colors: [0, 9, 10, 11, 12, 13, 14, 15],
            frags: [[0; 8]; 8],
            ping_ms: [0; 8],
            kill_limit: 25,
            time_limit_sec: 900.0, // 15 minutes
            match_timer: 0.0,
            show_scoreboard: false,
        }
    }
}

impl DukematchState {
    pub fn record_frag(&mut self, killer_id: usize, victim_id: usize) {
        if killer_id < 8 && victim_id < 8 {
            if killer_id == victim_id {
                // Suicide -> -1 frag
                self.frags[killer_id][victim_id] -= 1;
            } else {
                self.frags[killer_id][victim_id] += 1;
            }
        }
    }

    pub fn get_total_frags(&self, player_id: usize) -> i32 {
        if player_id >= 8 {
            return 0;
        }
        self.frags[player_id].iter().sum()
    }

    pub fn get_deaths(&self, player_id: usize) -> i32 {
        if player_id >= 8 {
            return 0;
        }
        let mut deaths = 0;
        for k in 0..8 {
            deaths += self.frags[k][player_id].abs();
        }
        deaths
    }

    pub fn get_leader(&self) -> usize {
        let mut leader = 0;
        let mut max_frags = i32::MIN;
        for i in 0..8 {
            let frags = self.get_total_frags(i);
            if frags > max_frags {
                max_frags = frags;
                leader = i;
            }
        }
        leader
    }
}
