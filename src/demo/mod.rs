#![allow(dead_code)]

pub mod attract;
pub mod format;
pub mod player;
pub mod recorder;

pub use attract::*;
pub use format::*;
pub use player::*;
pub use recorder::*;

use bevy::prelude::*;

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttractModeTimer>()
            .add_systems(Update, update_attract_mode_timer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_gametic_input_packing_and_unpacking() {
        let input = GameticInput {
            avel: -45,
            horz: 12,
            fvel: 250,
            svel: -120,
            bits: demo_bits::JUMP | demo_bits::FIRE | demo_bits::QUICK_KICK,
        };

        let bytes = input.to_bytes();
        assert_eq!(bytes.len(), 10);

        let unpacked = GameticInput::from_bytes(&bytes).expect("Failed to unpack GameticInput");
        assert_eq!(unpacked.avel, -45);
        assert_eq!(unpacked.horz, 12);
        assert_eq!(unpacked.fvel, 250);
        assert_eq!(unpacked.svel, -120);
        assert_eq!(unpacked.bits & demo_bits::JUMP, demo_bits::JUMP);
        assert_eq!(unpacked.bits & demo_bits::FIRE, demo_bits::FIRE);
        assert_eq!(unpacked.bits & demo_bits::QUICK_KICK, demo_bits::QUICK_KICK);
        assert_eq!(unpacked.bits & demo_bits::CROUCH, 0);
    }

    #[test]
    fn test_demo_header_packing() {
        let header = DemoHeader {
            total_tics: 1800, // 60 seconds @ 30 Hz
            version: DEMO_BYTEVERSION,
            episode: 1,
            level: 3,
            skill: 2,
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 8);

        let unpacked = DemoHeader::from_bytes(&bytes).expect("Failed to unpack DemoHeader");
        assert_eq!(unpacked.total_tics, 1800);
        assert_eq!(unpacked.version, DEMO_BYTEVERSION);
        assert_eq!(unpacked.episode, 1);
        assert_eq!(unpacked.level, 3);
        assert_eq!(unpacked.skill, 2);
    }

    #[test]
    fn test_demo_recorder_and_player_pipeline() {
        let mut recorder = DemoRecorder::default();
        recorder.start(1, 1, 1);
        assert!(recorder.is_recording);

        recorder.record_frame(GameticInput {
            avel: 10,
            horz: 0,
            fvel: 100,
            svel: 0,
            bits: demo_bits::RUN,
        });
        recorder.record_frame(GameticInput {
            avel: 20,
            horz: 5,
            fvel: 100,
            svel: 0,
            bits: demo_bits::FIRE,
        });
        recorder.record_frame(GameticInput {
            avel: 0,
            horz: 0,
            fvel: 0,
            svel: 0,
            bits: demo_bits::JUMP,
        });

        assert_eq!(recorder.inputs.len(), 3);

        let mut player = DemoPlayer::default();
        player.is_playing = true;
        player.inputs = recorder.inputs.clone();
        player.current_tic = 0;

        let frame1 = player.next_input().unwrap();
        assert_eq!(frame1.avel, 10);
        assert_eq!(frame1.bits, demo_bits::RUN);

        let frame2 = player.next_input().unwrap();
        assert_eq!(frame2.avel, 20);
        assert_eq!(frame2.bits, demo_bits::FIRE);

        let frame3 = player.next_input().unwrap();
        assert_eq!(frame3.bits, demo_bits::JUMP);

        assert_eq!(player.next_input(), None);
        assert!(!player.is_playing);
    }
}
