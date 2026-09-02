#![allow(dead_code)]

pub mod protocol;
pub mod rng;
pub mod scoreboard;

pub use protocol::*;
pub use rng::*;
pub use scoreboard::*;

use bevy::prelude::*;

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DukematchState>()
            .init_resource::<DeterministicRng>()
            .add_systems(
                Update,
                (toggle_scoreboard, update_dukematch_timer).in_set(crate::GameSet::Input),
            );
    }
}

pub fn toggle_scoreboard(keys: Res<ButtonInput<KeyCode>>, mut net_state: ResMut<DukematchState>) {
    if keys.just_pressed(KeyCode::F7) {
        net_state.show_scoreboard = !net_state.show_scoreboard;
    }
}

pub fn update_dukematch_timer(time: Res<Time>, mut net_state: ResMut<DukematchState>) {
    if net_state.mode != NetMode::SinglePlayer {
        net_state.match_timer += time.delta_seconds();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_packet_serialization_roundtrip() {
        // 1. Connect
        let pkt1 = NetPacket::Connect {
            player_id: 2,
            name: "DukeNukem".to_string(),
            color_pal: 9,
        };
        let bytes1 = pkt1.to_bytes();
        let decoded1 = NetPacket::from_bytes(&bytes1).unwrap();
        assert_eq!(pkt1, decoded1);

        // 2. InputSync
        let pkt2 = NetPacket::InputSync {
            player_id: 1,
            gametic: 12450,
            forward: 100,
            strafe: -50,
            yaw: 512,
            pitch: 0,
            actions: 0b101,
            weapon: 2,
        };
        let bytes2 = pkt2.to_bytes();
        let decoded2 = NetPacket::from_bytes(&bytes2).unwrap();
        assert_eq!(pkt2, decoded2);

        // 3. PlayerStateSync
        let pkt3 = NetPacket::PlayerStateSync {
            player_id: 0,
            x: 10.5,
            y: 1.0,
            z: -20.25,
            yaw: 1.57,
            pitch: -0.2,
            health: 100,
            armor: 50,
        };
        let bytes3 = pkt3.to_bytes();
        let decoded3 = NetPacket::from_bytes(&bytes3).unwrap();
        assert_eq!(pkt3, decoded3);

        // 4. FragEvent
        let pkt4 = NetPacket::FragEvent {
            killer_id: 0,
            victim_id: 1,
            weapon_type: 2,
        };
        let bytes4 = pkt4.to_bytes();
        let decoded4 = NetPacket::from_bytes(&bytes4).unwrap();
        assert_eq!(pkt4, decoded4);

        // 5. ChatMessage
        let pkt5 = NetPacket::ChatMessage {
            sender_id: 0,
            message: "Hail to the king, baby!".to_string(),
        };
        let bytes5 = pkt5.to_bytes();
        let decoded5 = NetPacket::from_bytes(&bytes5).unwrap();
        assert_eq!(pkt5, decoded5);
    }

    #[test]
    fn test_dukematch_frag_matrix_and_scoreboard() {
        let mut dm = DukematchState::default();
        dm.record_frag(0, 1);
        dm.record_frag(0, 2);
        dm.record_frag(1, 0);
        dm.record_frag(0, 0); // Player 0 suicide

        assert_eq!(dm.get_total_frags(0), 1); // 2 kills - 1 suicide = 1
        assert_eq!(dm.get_total_frags(1), 1); // 1 kill
        assert_eq!(dm.get_deaths(1), 1); // 1 death
        assert_eq!(dm.get_leader(), 0);
    }
}
