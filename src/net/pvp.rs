use crate::net::protocol::NetMode;
use crate::net::scoreboard::DukematchState;
use crate::player::types::{PlayerController, PlayerId};
use bevy::prelude::*;

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct PvpDamageEvent {
    pub attacker_id: usize,
    pub target_player_id: usize,
    pub damage: i32,
    pub weapon_type: u8,
}

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct PlayerFragEvent {
    pub killer_id: usize,
    pub victim_id: usize,
    pub weapon_type: u8,
}

#[derive(Component, Debug, Clone)]
pub struct SpawnInvulnerability {
    pub timer: f32,
}

impl Default for SpawnInvulnerability {
    fn default() -> Self {
        Self { timer: 3.0 }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MultiplayerSpawnPoint {
    pub spawn_idx: usize,
    pub position: Vec3,
    pub yaw: f32,
}

/// Applies PvP damage between players, respecting spawn invulnerability and sending frag events upon fatality.
pub fn apply_pvp_damage(
    mut damage_events: EventReader<PvpDamageEvent>,
    mut frag_events: EventWriter<PlayerFragEvent>,
    mut players: Query<(
        Entity,
        &mut PlayerController,
        &PlayerId,
        Option<&SpawnInvulnerability>,
    )>,
    net_state: Option<Res<DukematchState>>,
    coop_config: Option<Res<crate::net::coop::CoopConfig>>,
) {
    let mode = net_state.as_ref().map_or(NetMode::SinglePlayer, |s| s.mode);
    let ff_enabled = coop_config.as_ref().map_or(false, |c| c.friendly_fire);

    for ev in damage_events.read() {
        for (_entity, mut player, p_id, invuln) in players.iter_mut() {
            if p_id.0 != ev.target_player_id {
                continue;
            }

            // Spawn invulnerability check
            if let Some(inv) = invuln {
                if inv.timer > 0.0 {
                    continue; // Immune to damage during spawn protection
                }
            }

            if player.god_mode || player.health <= 0 {
                continue;
            }

            // In Cooperative mode, friendly fire can be disabled
            let is_friendly_fire = ev.attacker_id != ev.target_player_id;
            if mode == NetMode::Cooperative && is_friendly_fire && !ff_enabled {
                // If friendly fire disabled in Co-op, ignore
                continue;
            }

            // Apply armor absorption (authentic Build engine: 1/3 to armor, 2/3 to health)
            let mut remaining_dmg = ev.damage;
            if player.armor > 0 {
                let absorbed = (remaining_dmg / 3).min(player.armor);
                player.armor -= absorbed;
                remaining_dmg -= absorbed;
            }

            player.health -= remaining_dmg;

            if player.health <= 0 {
                player.health = 0;
                player.death_timer = 3.0;
                frag_events.send(PlayerFragEvent {
                    killer_id: ev.attacker_id,
                    victim_id: ev.target_player_id,
                    weapon_type: ev.weapon_type,
                });
            }
        }
    }
}

/// Updates spawn invulnerability timers.
pub fn update_spawn_invulnerability(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpawnInvulnerability)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut invuln) in query.iter_mut() {
        invuln.timer -= dt;
        if invuln.timer <= 0.0 {
            commands.entity(entity).remove::<SpawnInvulnerability>();
        }
    }
}

/// Processes frag events, updating DukematchState score matrix, sound, and HUD announcements.
pub fn handle_frag_events(
    mut frag_events: EventReader<PlayerFragEvent>,
    mut net_state: ResMut<DukematchState>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
    mut statusbar_query: Query<&mut crate::hud::StatusbarState>,
) {
    for ev in frag_events.read() {
        net_state.record_frag(ev.killer_id, ev.victim_id);
        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD

        let killer_name = if ev.killer_id < net_state.player_names.len() {
            net_state.player_names[ev.killer_id].clone()
        } else {
            format!("PLAYER {}", ev.killer_id + 1)
        };
        let victim_name = if ev.victim_id < net_state.player_names.len() {
            net_state.player_names[ev.victim_id].clone()
        } else {
            format!("PLAYER {}", ev.victim_id + 1)
        };
        let weapon_name = match ev.weapon_type {
            0 => "MIGHTY BOOT",
            1 => "PISTOL",
            2 => "SHOTGUN",
            3 => "CHAINGUN",
            4 => "RPG",
            5 => "PIPEBOMB",
            6 => "SHRINKER",
            7 => "DEVASTATOR",
            8 => "FREEZETHROWER",
            9 => "EXPANDER",
            _ => "WEAPON",
        };
        let announcement = format_frag_announcement(
            &killer_name,
            &victim_name,
            ev.killer_id == ev.victim_id,
            weapon_name,
        );
        for mut sbar in statusbar_query.iter_mut() {
            sbar.message_text = announcement.clone();
            sbar.message_timer = 4.0;
        }
    }
}

/// Checks if Dukematch kill limit or time limit has been reached, returning the winning player ID if over.
pub fn check_dukematch_winner(net_state: &DukematchState) -> Option<usize> {
    if net_state.mode != NetMode::Dukematch {
        return None;
    }

    // Check kill limit
    for i in 0..8 {
        if net_state.get_total_frags(i) >= net_state.kill_limit {
            return Some(i);
        }
    }

    // Check time limit
    if net_state.match_timer >= net_state.time_limit_sec && net_state.time_limit_sec > 0.0 {
        return Some(net_state.get_leader());
    }

    None
}

/// Formats the Dukematch match header summary.
pub fn format_dukematch_header(net_state: &DukematchState) -> String {
    let cur_min = (net_state.match_timer / 60.0).floor() as u32;
    let cur_sec = (net_state.match_timer % 60.0).floor() as u32;
    let max_min = (net_state.time_limit_sec / 60.0).floor() as u32;
    let max_sec = (net_state.time_limit_sec % 60.0).floor() as u32;

    format!(
        "DUKEMATCH - TIME: {:02}:{:02} / {:02}:{:02} - KILL LIMIT: {}",
        cur_min, cur_sec, max_min, max_sec, net_state.kill_limit
    )
}

/// Formats a scoreboard line for a given player.
pub fn format_player_scoreboard_row(net_state: &DukematchState, player_id: usize) -> String {
    let name = &net_state.player_names[player_id];
    let frags = net_state.get_total_frags(player_id);
    let deaths = net_state.get_deaths(player_id);
    let ping = net_state.ping_ms[player_id];
    format!(
        "{:<10} | FRAGS: {:>3} | DEATHS: {:>3} | PING: {:>3}ms",
        name, frags, deaths, ping
    )
}

/// Formats a frag announcement string.
pub fn format_frag_announcement(
    killer_name: &str,
    victim_name: &str,
    is_suicide: bool,
    weapon_name: &str,
) -> String {
    if is_suicide {
        format!("{} committed suicide", victim_name)
    } else {
        format!("{} fragged {} with {}", killer_name, victim_name, weapon_name)
    }
}

#[derive(Component, Debug, Clone)]
pub struct ScoreboardRoot;

#[derive(Component, Debug, Clone)]
pub struct ScoreboardHeaderText;

#[derive(Component, Debug, Clone)]
pub struct ScoreboardContentText;

/// Spawns the F7 scoreboard UI hierarchy.
pub fn setup_scoreboard_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(15.0),
                    top: Val::Percent(15.0),
                    width: Val::Percent(70.0),
                    height: Val::Percent(70.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.05, 0.1, 0.88).into(),
                border_color: Color::srgb(0.9, 0.6, 0.1).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            ScoreboardRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "DUKEMATCH SCOREBOARD",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(1.0, 0.8, 0.2),
                        ..default()
                    },
                ),
                ScoreboardHeaderText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ScoreboardContentText,
            ));
        });
}

/// Updates the F7 scoreboard visibility and content each frame based on DukematchState.
pub fn update_scoreboard_overlay(
    net_state: Res<DukematchState>,
    mut root_query: Query<&mut Visibility, With<ScoreboardRoot>>,
    mut header_query: Query<&mut Text, (With<ScoreboardHeaderText>, Without<ScoreboardContentText>)>,
    mut content_query: Query<&mut Text, (With<ScoreboardContentText>, Without<ScoreboardHeaderText>)>,
) {
    let Ok(mut vis) = root_query.get_single_mut() else {
        return;
    };
    if net_state.show_scoreboard {
        *vis = Visibility::Inherited;
        if let Ok(mut header) = header_query.get_single_mut() {
            header.sections[0].value = format_dukematch_header(&net_state);
        }
        if let Ok(mut content) = content_query.get_single_mut() {
            let mut rows = Vec::new();
            for i in 0..8 {
                if net_state.get_total_frags(i) != 0 || net_state.get_deaths(i) != 0 || i < 2 {
                    rows.push(format_player_scoreboard_row(&net_state, i));
                }
            }
            content.sections[0].value = rows.join("\n");
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvp_damage_and_frag_attribution() {
        let mut app = App::new();
        app.add_event::<PvpDamageEvent>();
        app.add_event::<PlayerFragEvent>();
        app.init_resource::<DukematchState>();
        app.world_mut().resource_mut::<DukematchState>().mode = NetMode::Dukematch;

        app.add_systems(Update, apply_pvp_damage);

        // Spawn P0 and P1
        let mut p1_ctrl = PlayerController::default();
        p1_ctrl.health = 30;
        p1_ctrl.armor = 0;
        app.world_mut().spawn((p1_ctrl, PlayerId(1)));

        // Send PvpDamageEvent: P0 deals 40 damage to P1
        app.world_mut().send_event(PvpDamageEvent {
            attacker_id: 0,
            target_player_id: 1,
            damage: 40,
            weapon_type: 2, // Shotgun
        });

        app.update();

        let frag_events = app.world().resource::<Events<PlayerFragEvent>>();
        assert_eq!(frag_events.len(), 1);
        let events: Vec<&PlayerFragEvent> = frag_events.iter_current_update_events().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].killer_id, 0);
        assert_eq!(events[0].victim_id, 1);
        assert_eq!(events[0].weapon_type, 2);
    }

    #[test]
    fn test_pvp_spawn_invulnerability_protects_player() {
        let mut app = App::new();
        app.add_event::<PvpDamageEvent>();
        app.add_event::<PlayerFragEvent>();
        app.init_resource::<DukematchState>();
        app.world_mut().resource_mut::<DukematchState>().mode = NetMode::Dukematch;

        app.add_systems(Update, apply_pvp_damage);

        // Spawn P1 with active spawn invulnerability (3.0s)
        let mut p1_ctrl = PlayerController::default();
        p1_ctrl.health = 100;
        let p1_entity = app.world_mut().spawn((
            p1_ctrl,
            PlayerId(1),
            SpawnInvulnerability { timer: 3.0 },
        )).id();

        // P0 attempts to deal 50 damage
        app.world_mut().send_event(PvpDamageEvent {
            attacker_id: 0,
            target_player_id: 1,
            damage: 50,
            weapon_type: 1,
        });

        app.update();

        let player = app.world().entity(p1_entity).get::<PlayerController>().unwrap();
        // Health must remain intact
        assert_eq!(player.health, 100);

        let frag_events = app.world().resource::<Events<PlayerFragEvent>>();
        assert!(frag_events.is_empty());
    }

    #[test]
    fn test_suicide_penalty_scoring() {
        let mut app = App::new();
        app.add_event::<PlayerFragEvent>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.init_resource::<DukematchState>();

        app.add_systems(Update, handle_frag_events);

        // Send suicide event (P1 self-kills)
        app.world_mut().send_event(PlayerFragEvent {
            killer_id: 1,
            victim_id: 1,
            weapon_type: 4, // RPG self-damage
        });

        app.update();

        let dm = app.world().resource::<DukematchState>();
        assert_eq!(dm.get_total_frags(1), -1);
    }

    #[test]
    fn test_dukematch_winner_evaluation() {
        let mut dm = DukematchState::default();
        dm.mode = NetMode::Dukematch;
        dm.kill_limit = 10;
        dm.time_limit_sec = 600.0;

        assert_eq!(check_dukematch_winner(&dm), None);

        // Simulate 10 frags for player 0
        for _ in 0..10 {
            dm.record_frag(0, 1);
        }
        assert_eq!(check_dukematch_winner(&dm), Some(0));

        // Reset and test time limit expiration
        let mut dm2 = DukematchState::default();
        dm2.mode = NetMode::Dukematch;
        dm2.kill_limit = 25;
        dm2.time_limit_sec = 300.0;
        dm2.match_timer = 301.0;
        dm2.record_frag(2, 0); // Player 2 has 1 frag

        assert_eq!(check_dukematch_winner(&dm2), Some(2));
    }

    #[test]
    fn test_scoreboard_and_frag_announcement_formatting() {
        let mut dm = DukematchState::default();
        dm.mode = NetMode::Dukematch;
        dm.match_timer = 125.0; // 02:05
        dm.kill_limit = 20;
        dm.record_frag(0, 1);

        let header = format_dukematch_header(&dm);
        assert!(header.contains("02:05"));
        assert!(header.contains("KILL LIMIT: 20"));

        let row0 = format_player_scoreboard_row(&dm, 0);
        assert!(row0.contains("DUKE"));
        assert!(row0.contains("FRAGS:   1"));

        let row1 = format_player_scoreboard_row(&dm, 1);
        assert!(row1.contains("DEATHS:   1"));

        let kill_announcement = format_frag_announcement("DUKE", "PLAYER 2", false, "SHOTGUN");
        assert_eq!(kill_announcement, "DUKE fragged PLAYER 2 with SHOTGUN");

        let suicide_announcement = format_frag_announcement("PLAYER 2", "PLAYER 2", true, "RPG");
        assert_eq!(suicide_announcement, "PLAYER 2 committed suicide");
    }

    #[test]
    fn test_scoreboard_overlay_visibility_and_content_update() {
        let mut app = App::new();
        app.init_resource::<DukematchState>();
        app.add_systems(Update, update_scoreboard_overlay);

        let root = app.world_mut().spawn((
            ScoreboardRoot,
            Visibility::Hidden,
        )).id();

        let header = app.world_mut().spawn((
            ScoreboardHeaderText,
            Text::from_section("OLD HEADER", TextStyle::default()),
        )).id();

        let content = app.world_mut().spawn((
            ScoreboardContentText,
            Text::from_section("OLD CONTENT", TextStyle::default()),
        )).id();

        // 1. With show_scoreboard = false: stays hidden
        app.update();
        let vis = app.world().entity(root).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Hidden);

        // 2. Set show_scoreboard = true: becomes Inherited and updates text
        {
            let mut dm = app.world_mut().resource_mut::<DukematchState>();
            dm.show_scoreboard = true;
            dm.record_frag(0, 1);
        }
        app.update();

        let vis = app.world().entity(root).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Inherited);

        let header_txt = app.world().entity(header).get::<Text>().unwrap();
        assert!(header_txt.sections[0].value.contains("DUKEMATCH"));

        let content_txt = app.world().entity(content).get::<Text>().unwrap();
        assert!(content_txt.sections[0].value.contains("DUKE"));
    }

    #[test]
    fn test_frag_announcement_updates_statusbar() {
        let mut app = App::new();
        app.add_event::<PlayerFragEvent>();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.init_resource::<DukematchState>();
        app.add_systems(Update, handle_frag_events);

        let sbar_entity = app.world_mut().spawn(crate::hud::StatusbarState::default()).id();

        app.world_mut().send_event(PlayerFragEvent {
            killer_id: 0,
            victim_id: 1,
            weapon_type: 2, // Shotgun
        });

        app.update();

        let sbar = app.world().entity(sbar_entity).get::<crate::hud::StatusbarState>().unwrap();
        assert!(sbar.message_text.contains("fragged"));
        assert!(sbar.message_text.contains("SHOTGUN"));
        assert!(sbar.message_timer > 0.0);
    }

    #[test]
    fn test_multiplayer_spawn_point_respawn_and_invulnerability() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<DukematchState>();
        app.add_systems(Update, crate::player::movement::update_player_movement);

        // Spawn multiplayer spawn point at (20.0, 5.0, -10.0)
        app.world_mut().spawn(MultiplayerSpawnPoint {
            spawn_idx: 0,
            position: Vec3::new(20.0, 5.0, -10.0),
            yaw: 0.0,
        });

        // Spawn dead player with death_timer = 0.01s
        let mut player = PlayerController::default();
        player.health = 0;
        player.death_timer = 0.01;
        player.spawn_position = Vec3::ZERO;

        let player_entity = app.world_mut().spawn((
            player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            bevy_rapier3d::prelude::KinematicCharacterController::default(),
            PlayerId(0),
        )).id();

        // Spawn camera for player
        app.world_mut().spawn((
            Camera3dBundle::default(),
            crate::player::types::PlayerCamera(0),
        ));

        // Advance time to expire death timer
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(50));
        }

        app.update();

        let updated_player = app.world().entity(player_entity).get::<PlayerController>().unwrap();
        assert_eq!(updated_player.health, updated_player.max_health);

        let trans = app.world().entity(player_entity).get::<Transform>().unwrap();
        assert_eq!(trans.translation, Vec3::new(20.0, 5.0, -10.0));

        let invuln = app.world().entity(player_entity).get::<SpawnInvulnerability>();
        assert!(invuln.is_some(), "Respawned player must gain SpawnInvulnerability");
    }
}
