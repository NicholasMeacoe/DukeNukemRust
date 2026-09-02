#![allow(dead_code)]

use bevy::prelude::*;
use crate::game_flow::state::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntermissionStage {
    #[default]
    BackgroundFadeIn, // 0.0s .. 0.5s
    KillsTally,       // 0.5s .. 1.8s
    SecretsTally,     // 1.8s .. 3.0s
    TimeReveal,       // 3.0s .. 4.2s
    ReadyToProceed,   // 4.2s+
}

#[derive(Resource, Debug, Clone, Default)]
pub struct IntermissionAnimationState {
    pub stage: IntermissionStage,
    pub timer: f32,
    pub displayed_kills: i32,
    pub displayed_secrets: i32,
    pub speech_played: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntermissionStats {
    pub kill_percentage: i32,
    pub secret_percentage: i32,
    pub time_taken_seconds: f32,
    pub par_time_seconds: f32,
}

pub fn update_level_time(
    time: Res<Time>,
    state: Res<State<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
) {
    if *state.get() == GamePhase::Playing {
        progress.level_time_seconds += time.delta_seconds();
    }
}

pub fn handle_level_completed_events(
    mut events: EventReader<LevelCompletedEvent>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
    mut anim_state: ResMut<IntermissionAnimationState>,
) {
    for _ in events.read() {
        progress.is_level_completed = true;
        anim_state.stage = IntermissionStage::BackgroundFadeIn;
        anim_state.timer = 0.0;
        anim_state.displayed_kills = 0;
        anim_state.displayed_secrets = 0;
        anim_state.speech_played = false;
        next_state.set(GamePhase::Intermission);
    }
}

pub fn update_intermission_animation(
    time: Res<Time>,
    state: Res<State<GamePhase>>,
    progress: Res<LevelProgress>,
    mut anim_state: ResMut<IntermissionAnimationState>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    if *state.get() != GamePhase::Intermission {
        return;
    }

    let dt = time.delta_seconds();
    anim_state.timer += dt;

    let stats = progress.compute_stats();

    match anim_state.stage {
        IntermissionStage::BackgroundFadeIn => {
            if anim_state.timer >= 0.5 {
                anim_state.stage = IntermissionStage::KillsTally;
            }
        }
        IntermissionStage::KillsTally => {
            let progress_ratio = ((anim_state.timer - 0.5) / 1.3).clamp(0.0, 1.0);
            anim_state.displayed_kills = (stats.kill_percentage as f32 * progress_ratio) as i32;

            if anim_state.timer >= 1.8 {
                anim_state.displayed_kills = stats.kill_percentage;
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 }); // PISTOL_FIRE tally sound
                anim_state.stage = IntermissionStage::SecretsTally;
            }
        }
        IntermissionStage::SecretsTally => {
            let progress_ratio = ((anim_state.timer - 1.8) / 1.2).clamp(0.0, 1.0);
            anim_state.displayed_secrets = (stats.secret_percentage as f32 * progress_ratio) as i32;

            if anim_state.timer >= 3.0 {
                anim_state.displayed_secrets = stats.secret_percentage;
                sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 });
                anim_state.stage = IntermissionStage::TimeReveal;
            }
        }
        IntermissionStage::TimeReveal => {
            if !anim_state.speech_played && anim_state.timer >= 3.3 {
                anim_state.speech_played = true;
                if stats.time_taken_seconds <= stats.par_time_seconds {
                    // Beat par time -> "Damn, I'm good!"
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 40 });
                } else if stats.secret_percentage >= 100 {
                    // All secrets -> "Groovy!"
                    sound_events.send(crate::audio::PlaySoundEvent { sound_id: 41 });
                }
            }

            if anim_state.timer >= 4.2 {
                anim_state.stage = IntermissionStage::ReadyToProceed;
            }
        }
        IntermissionStage::ReadyToProceed => {}
    }
}

pub fn handle_intermission_input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut progress: ResMut<LevelProgress>,
    mut anim_state: ResMut<IntermissionAnimationState>,
    mut load_level_events: EventWriter<LoadLevelEvent>,
) {
    if *state.get() != GamePhase::Intermission {
        return;
    }

    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        if anim_state.stage != IntermissionStage::ReadyToProceed {
            // Fast forward to end of animation
            let stats = progress.compute_stats();
            anim_state.displayed_kills = stats.kill_percentage;
            anim_state.displayed_secrets = stats.secret_percentage;
            anim_state.stage = IntermissionStage::ReadyToProceed;
            return;
        }

        let episode_finished = progress.advance_to_next_level();
        if episode_finished {
            next_state.set(GamePhase::MainMenu);
        } else {
            load_level_events.send(LoadLevelEvent {
                episode: progress.current_episode,
                level: progress.current_level,
            });
            next_state.set(GamePhase::Playing);
        }
    }
}

impl LevelProgress {
    pub fn compute_stats(&self) -> IntermissionStats {
        let kill_percentage = if self.total_monsters > 0 {
            ((self.kills_count as f32 / self.total_monsters as f32) * 100.0) as i32
        } else {
            100
        };

        let secret_percentage = if self.total_secrets > 0 {
            ((self.secrets_found as f32 / self.total_secrets as f32) * 100.0) as i32
        } else {
            100
        };

        IntermissionStats {
            kill_percentage: kill_percentage.clamp(0, 100),
            secret_percentage: secret_percentage.clamp(0, 100),
            time_taken_seconds: self.level_time_seconds,
            par_time_seconds: self.par_time_seconds,
        }
    }
}
