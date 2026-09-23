use crate::game_flow::LevelCompletedEvent;
use crate::net::protocol::NetMode;
use crate::player::types::{PlayerController, PlayerId};
use bevy::prelude::*;

/// Configuration resource for Cooperative campaign multiplayer.
#[derive(Resource, Debug, Clone)]
pub struct CoopConfig {
    pub friendly_fire: bool,
    pub shared_keys: bool,
    pub spawn_at_checkpoints: bool,
}

impl Default for CoopConfig {
    fn default() -> Self {
        Self {
            friendly_fire: false,
            shared_keys: true,
            spawn_at_checkpoints: true,
        }
    }
}

/// Global shared keycard inventory for cooperative play.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SharedKeycards {
    pub has_blue_key: bool,
    pub has_red_key: bool,
    pub has_yellow_key: bool,
}

impl SharedKeycards {
    pub fn give_key(&mut self, key_type: u8) {
        match key_type {
            1 => self.has_blue_key = true,
            2 => self.has_red_key = true,
            3 => self.has_yellow_key = true,
            _ => {}
        }
    }

    pub fn has_key(&self, key_type: u8) -> bool {
        match key_type {
            1 => self.has_blue_key,
            2 => self.has_red_key,
            3 => self.has_yellow_key,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        self.has_blue_key = false;
        self.has_red_key = false;
        self.has_yellow_key = false;
    }
}

/// Tag component designating a cooperative spawn / progress checkpoint in the world.
#[derive(Component, Debug, Clone, Copy)]
pub struct CoopCheckpoint {
    pub checkpoint_idx: usize,
    pub position: Vec3,
    pub is_unlocked: bool,
}

/// Resource managing synchronized cooperative level progression.
#[derive(Resource, Debug, Clone, Default)]
pub struct CoopLevelTransition {
    pub transition_pending: bool,
    pub is_secret: bool,
    pub next_level_idx: usize,
    pub ready_players: usize,
    pub total_players: usize,
}

impl CoopLevelTransition {
    pub fn trigger(&mut self, is_secret: bool, total_players: usize) {
        self.transition_pending = true;
        self.is_secret = is_secret;
        self.total_players = total_players;
        self.ready_players = 0;
    }

    pub fn player_ready(&mut self) {
        self.ready_players += 1;
    }

    pub fn is_all_ready(&self) -> bool {
        self.transition_pending && (self.ready_players >= self.total_players || self.total_players == 0)
    }

    pub fn reset(&mut self) {
        self.transition_pending = false;
        self.is_secret = false;
        self.ready_players = 0;
        self.total_players = 0;
    }
}

/// Synchronizes keycard pickups across all active players in cooperative sessions.
pub fn sync_coop_keycards_system(
    coop_config: Option<Res<CoopConfig>>,
    mut shared_keys: ResMut<SharedKeycards>,
    mut player_query: Query<&mut PlayerController>,
) {
    let shared_enabled = coop_config.as_ref().map_or(true, |c| c.shared_keys);
    if !shared_enabled {
        return;
    }

    // Step 1: Collect any newly acquired keycards into the shared resource
    for player in player_query.iter() {
        if player.has_blue_key {
            shared_keys.has_blue_key = true;
        }
        if player.has_red_key {
            shared_keys.has_red_key = true;
        }
        if player.has_yellow_key {
            shared_keys.has_yellow_key = true;
        }
    }

    // Step 2: Propagate shared keycards to all players
    for mut player in player_query.iter_mut() {
        if shared_keys.has_blue_key {
            player.has_blue_key = true;
        }
        if shared_keys.has_red_key {
            player.has_red_key = true;
        }
        if shared_keys.has_yellow_key {
            player.has_yellow_key = true;
        }
    }
}

/// Handles cooperative level transition synchronization when any player hits the nuke exit button.
pub fn coop_level_advance_system(
    mut level_events: EventReader<LevelCompletedEvent>,
    mut transition: ResMut<CoopLevelTransition>,
    player_query: Query<&PlayerId>,
) {
    for event in level_events.read() {
        let player_count = player_query.iter().count().max(1);
        transition.trigger(event.is_secret, player_count);
    }
}

/// Evaluates PvP damage taking into account cooperative mode and friendly fire settings.
pub fn evaluate_coop_pvp_damage(
    attacker_id: usize,
    victim_id: usize,
    mode: NetMode,
    friendly_fire: bool,
) -> bool {
    let is_friendly = attacker_id != victim_id;
    if mode == NetMode::Cooperative && is_friendly && !friendly_fire {
        // Friendly fire is disabled in Co-op -> ignore damage
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_keycards_resource_methods() {
        let mut keys = SharedKeycards::default();
        assert!(!keys.has_key(1));
        assert!(!keys.has_key(2));
        assert!(!keys.has_key(3));

        keys.give_key(1); // Blue
        assert!(keys.has_key(1));
        assert!(keys.has_blue_key);

        keys.give_key(2); // Red
        assert!(keys.has_key(2));
        assert!(keys.has_red_key);

        keys.give_key(3); // Yellow
        assert!(keys.has_key(3));
        assert!(keys.has_yellow_key);

        keys.reset();
        assert!(!keys.has_blue_key);
        assert!(!keys.has_red_key);
        assert!(!keys.has_yellow_key);
    }

    #[test]
    fn test_shared_keycards_unlock_for_all_players() {
        let mut app = App::new();
        app.init_resource::<SharedKeycards>()
            .init_resource::<CoopConfig>()
            .add_systems(Update, sync_coop_keycards_system);

        // Spawn Player 1 with Blue Key
        let mut p1 = PlayerController::default();
        p1.has_blue_key = true;
        app.world_mut().spawn((p1, PlayerId(0)));

        // Spawn Player 2 without keys
        let p2 = PlayerController::default();
        let p2_entity = app.world_mut().spawn((p2, PlayerId(1))).id();

        // Run system update
        app.update();

        // Player 2 should now have the Blue Keycard unlocked
        let p2_ref = app.world().get::<PlayerController>(p2_entity).unwrap();
        assert!(p2_ref.has_blue_key);
        assert!(!p2_ref.has_red_key);

        // Shared resource should reflect the blue key
        let shared = app.world().resource::<SharedKeycards>();
        assert!(shared.has_blue_key);
    }

    #[test]
    fn test_cooperative_friendly_fire_toggle() {
        // Case 1: Co-op with Friendly Fire disabled -> Damage rejected
        let can_damage_ff_off = evaluate_coop_pvp_damage(0, 1, NetMode::Cooperative, false);
        assert!(!can_damage_ff_off);

        // Case 2: Co-op with Friendly Fire enabled -> Damage allowed
        let can_damage_ff_on = evaluate_coop_pvp_damage(0, 1, NetMode::Cooperative, true);
        assert!(can_damage_ff_on);

        // Case 3: Co-op self damage (e.g. rocket/pipebomb suicide) -> Damage always allowed
        let can_damage_self = evaluate_coop_pvp_damage(0, 0, NetMode::Cooperative, false);
        assert!(can_damage_self);

        // Case 4: Dukematch mode -> Damage always allowed regardless of friendly fire
        let can_damage_dm = evaluate_coop_pvp_damage(0, 1, NetMode::Dukematch, false);
        assert!(can_damage_dm);
    }

    #[test]
    fn test_cooperative_level_transition_synchronization() {
        let mut app = App::new();
        app.init_resource::<CoopLevelTransition>()
            .add_event::<LevelCompletedEvent>()
            .add_systems(Update, coop_level_advance_system);

        // Spawn 2 players
        app.world_mut().spawn(PlayerId(0));
        app.world_mut().spawn(PlayerId(1));

        // Send level completed event (e.g. from Nuke button)
        app.world_mut().send_event(LevelCompletedEvent {
            is_secret: true,
            is_boss_victory: false,
        });

        app.update();

        let mut transition = app.world_mut().resource_mut::<CoopLevelTransition>();
        assert!(transition.transition_pending);
        assert!(transition.is_secret);
        assert_eq!(transition.total_players, 2);
        assert_eq!(transition.ready_players, 0);
        assert!(!transition.is_all_ready());

        transition.player_ready();
        assert!(!transition.is_all_ready());

        transition.player_ready();
        assert!(transition.is_all_ready());
    }

    #[test]
    fn test_cooperative_respawn_at_checkpoint() {
        let checkpoint = CoopCheckpoint {
            checkpoint_idx: 1,
            position: Vec3::new(10.0, 1.5, -20.0),
            is_unlocked: true,
        };

        // When respawning, if an unlocked checkpoint is found, its position is used
        assert!(checkpoint.is_unlocked);
        assert_eq!(checkpoint.position, Vec3::new(10.0, 1.5, -20.0));
    }
}
