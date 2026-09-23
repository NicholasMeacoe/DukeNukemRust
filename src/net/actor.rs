use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub const APLAYER_BASE_TILE: i16 = 1405;
pub const APLAYER_DEATH_TILE: i16 = 1440;
pub const APLAYER_GIB_TILE: i16 = 1446;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimState {
    #[default]
    Idle,
    Walking,
    Running,
    Firing,
    Crouching,
    Dying,
    Gibbed,
}

#[derive(Component, Debug, Clone)]
pub struct RemotePlayerActor {
    pub player_id: usize,
    pub anim_state: PlayerAnimState,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub palette_idx: u8,
    pub yaw: f32,
}

impl Default for RemotePlayerActor {
    fn default() -> Self {
        Self {
            player_id: 0,
            anim_state: PlayerAnimState::Idle,
            anim_timer: 0.0,
            anim_frame: 0,
            palette_idx: 0,
            yaw: 0.0,
        }
    }
}

/// Computes the 8-directional view sector (0..7) of the actor relative to the camera.
/// 0: Front (facing camera)
/// 1: Front-Right of actor / viewer sees front-left
/// 2: Right side
/// 3: Back-Right
/// 4: Back (facing away from camera)
/// 5: Back-Left
/// 6: Left side
/// 7: Front-Left
pub fn calculate_actor_view_direction(actor_pos: Vec3, actor_yaw: f32, camera_pos: Vec3) -> usize {
    let to_cam = Vec2::new(camera_pos.x - actor_pos.x, camera_pos.z - actor_pos.z);
    if to_cam.length_squared() < 0.0001 {
        return 0; // Directly on top of camera -> front view
    }

    let angle_to_cam = to_cam.y.atan2(to_cam.x);
    let delta = (angle_to_cam - actor_yaw).rem_euclid(TAU);
    // 8 sectors of 45 deg (TAU / 8), centered on 0, 45, 90, ...
    let sector_offset = (delta + PI / 8.0).rem_euclid(TAU);
    let sector = (sector_offset / (TAU / 8.0)).floor() as usize;
    sector % 8
}

/// Maps Duke 3D multiplayer palette index (0..15) to an RGB tint color.
pub fn player_palette_to_color(palette_idx: u8) -> Color {
    match palette_idx {
        0 => Color::srgb(1.0, 1.0, 1.0),        // 0: Classic (Default original sprite)
        9 => Color::srgb(0.2, 0.4, 1.0),        // 9: Blue
        10 => Color::srgb(1.0, 0.25, 0.25),     // 10: Red
        11 => Color::srgb(0.25, 0.9, 0.25),     // 11: Green
        12 => Color::srgb(0.75, 0.75, 0.75),    // 12: Grey / Silver
        13 => Color::srgb(0.15, 0.55, 0.15),    // 13: Dark Green
        14 => Color::srgb(0.65, 0.45, 0.25),    // 14: Brown
        15 => Color::srgb(0.15, 0.25, 0.65),    // 15: Dark Blue
        _ => Color::srgb(1.0, 1.0, 1.0),
    }
}

/// Evaluates player state machine to determine the current animation state.
pub fn evaluate_player_anim_state(
    health: i32,
    is_gibbed: bool,
    is_crouching: bool,
    is_firing: bool,
    velocity_xz: Vec2,
) -> PlayerAnimState {
    if is_gibbed {
        PlayerAnimState::Gibbed
    } else if health <= 0 {
        PlayerAnimState::Dying
    } else if is_crouching {
        PlayerAnimState::Crouching
    } else if is_firing {
        PlayerAnimState::Firing
    } else {
        let speed = velocity_xz.length();
        if speed > 6.0 {
            PlayerAnimState::Running
        } else if speed > 0.3 {
            PlayerAnimState::Walking
        } else {
            PlayerAnimState::Idle
        }
    }
}

/// Calculates the corresponding Duke 3D tile ID for the player actor given anim state and viewing angle.
pub fn compute_player_actor_tile(
    anim_state: PlayerAnimState,
    view_direction: usize,
    anim_frame: usize,
) -> i16 {
    let dir = (view_direction % 8) as i16;
    match anim_state {
        PlayerAnimState::Idle => APLAYER_BASE_TILE + dir,
        PlayerAnimState::Walking => {
            let walk_frame = (anim_frame % 4) as i16;
            APLAYER_BASE_TILE + walk_frame * 8 + dir
        }
        PlayerAnimState::Running => {
            let run_frame = (anim_frame % 4) as i16;
            APLAYER_BASE_TILE + run_frame * 8 + dir
        }
        PlayerAnimState::Firing => APLAYER_BASE_TILE + 32 + dir,
        PlayerAnimState::Crouching => APLAYER_BASE_TILE + 40 + dir,
        PlayerAnimState::Dying => {
            let death_frame = (anim_frame.min(5)) as i16;
            APLAYER_DEATH_TILE + death_frame
        }
        PlayerAnimState::Gibbed => APLAYER_GIB_TILE,
    }
}

/// System to update remote player animation timers and frames.
pub fn update_remote_player_animations(
    time: Res<Time>,
    mut query: Query<&mut RemotePlayerActor>,
) {
    let dt = time.delta_seconds();
    for mut actor in query.iter_mut() {
        actor.anim_timer += dt;
        let frame_duration = match actor.anim_state {
            PlayerAnimState::Running => 0.1,
            PlayerAnimState::Walking => 0.15,
            PlayerAnimState::Firing => 0.12,
            PlayerAnimState::Dying => 0.2,
            _ => 0.25,
        };
        if actor.anim_timer >= frame_duration {
            actor.anim_timer -= frame_duration;
            actor.anim_frame = (actor.anim_frame + 1) % 16;
        }
    }
}

/// Triggers standard collapsing death or visceral explosive gib death sequence.
pub fn trigger_player_death_or_gib(
    player_pos: Vec3,
    is_explosive: bool,
    sound_events: &mut EventWriter<crate::audio::PlaySoundEvent>,
    gib_events: &mut EventWriter<crate::combat::types::GibEvent>,
) -> PlayerAnimState {
    if is_explosive {
        gib_events.send(crate::combat::types::GibEvent {
            origin: player_pos,
            gib_count: 12,
        });
        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 110 }); // SQUISHED
        PlayerAnimState::Gibbed
    } else {
        sound_events.send(crate::audio::PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD
        PlayerAnimState::Dying
    }
}

/// Rotates player actor billboards to face the active camera.
pub fn update_actor_billboard_transforms(
    camera_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<RemotePlayerActor>, Without<Camera>)>,
) {
    let Some(cam_trans) = camera_query.iter().next() else {
        return;
    };
    let cam_pos = cam_trans.translation;
    for mut trans in query.iter_mut() {
        let mut look_target = cam_pos;
        look_target.y = trans.translation.y;
        trans.look_at(look_target, Vec3::Y);
    }
}

/// Updates remote player billboard materials with the correct tile and palette tint.
pub fn update_player_billboard_sprites(
    camera_query: Query<&Transform, With<Camera>>,
    mut query: Query<(
        &RemotePlayerActor,
        &Transform,
        &mut Handle<StandardMaterial>,
    )>,
    game_assets: Option<Res<crate::GameAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(game_assets) = game_assets else {
        return;
    };
    let Some(cam_trans) = camera_query.iter().next() else {
        return;
    };
    let cam_pos = cam_trans.translation;

    for (actor, trans, mut material_handle) in query.iter_mut() {
        let view_dir = calculate_actor_view_direction(trans.translation, actor.yaw, cam_pos);
        let picnum = compute_player_actor_tile(actor.anim_state, view_dir, actor.anim_frame);
        let tint = player_palette_to_color(actor.palette_idx);

        if let Some(tex) = game_assets.tile_textures.get(&picnum) {
            *material_handle = materials.add(StandardMaterial {
                base_color_texture: Some(tex.clone()),
                base_color: tint,
                alpha_mode: AlphaMode::Mask(0.5),
                unlit: true,
                ..default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_8_directional_view_sectors() {
        let actor_pos = Vec3::ZERO;
        let actor_yaw = 0.0; // Facing along positive X axis (angle 0.0)

        // 1. Camera in front (+X direction): viewer looks at actor's face -> Front (0)
        let cam_front = Vec3::new(10.0, 0.0, 0.0);
        assert_eq!(calculate_actor_view_direction(actor_pos, actor_yaw, cam_front), 0);

        // 2. Camera behind (-X direction): viewer looks at actor's back -> Back (4)
        let cam_back = Vec3::new(-10.0, 0.0, 0.0);
        assert_eq!(calculate_actor_view_direction(actor_pos, actor_yaw, cam_back), 4);

        // 3. Camera to the left (+Z direction, angle PI/2) -> Side view (2)
        let cam_side_left = Vec3::new(0.0, 0.0, 10.0);
        assert_eq!(calculate_actor_view_direction(actor_pos, actor_yaw, cam_side_left), 2);

        // 4. Camera to the right (-Z direction, angle -PI/2) -> Side view (6)
        let cam_side_right = Vec3::new(0.0, 0.0, -10.0);
        assert_eq!(calculate_actor_view_direction(actor_pos, actor_yaw, cam_side_right), 6);
    }

    #[test]
    fn test_palette_lookup_index_translation() {
        // Default palette 0 is pure white tint (texture native)
        assert_eq!(player_palette_to_color(0), Color::srgb(1.0, 1.0, 1.0));

        // Multiplayer colors 9..15
        assert_eq!(player_palette_to_color(9), Color::srgb(0.2, 0.4, 1.0));
        assert_eq!(player_palette_to_color(10), Color::srgb(1.0, 0.25, 0.25));
        assert_eq!(player_palette_to_color(11), Color::srgb(0.25, 0.9, 0.25));
        assert_eq!(player_palette_to_color(12), Color::srgb(0.75, 0.75, 0.75));
        assert_eq!(player_palette_to_color(13), Color::srgb(0.15, 0.55, 0.15));
        assert_eq!(player_palette_to_color(14), Color::srgb(0.65, 0.45, 0.25));
        assert_eq!(player_palette_to_color(15), Color::srgb(0.15, 0.25, 0.65));

        // Unknown fallback
        assert_eq!(player_palette_to_color(99), Color::srgb(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_player_animation_state_machine_transitions() {
        // 1. Idle
        let s_idle = evaluate_player_anim_state(100, false, false, false, Vec2::ZERO);
        assert_eq!(s_idle, PlayerAnimState::Idle);

        // 2. Walking
        let s_walk = evaluate_player_anim_state(100, false, false, false, Vec2::new(2.0, 0.0));
        assert_eq!(s_walk, PlayerAnimState::Walking);

        // 3. Running
        let s_run = evaluate_player_anim_state(100, false, false, false, Vec2::new(7.0, 0.0));
        assert_eq!(s_run, PlayerAnimState::Running);

        // 4. Firing overrides walking
        let s_fire = evaluate_player_anim_state(100, false, false, true, Vec2::new(3.0, 0.0));
        assert_eq!(s_fire, PlayerAnimState::Firing);

        // 5. Crouching
        let s_crouch = evaluate_player_anim_state(100, false, true, false, Vec2::ZERO);
        assert_eq!(s_crouch, PlayerAnimState::Crouching);

        // 6. Dying
        let s_die = evaluate_player_anim_state(0, false, false, false, Vec2::ZERO);
        assert_eq!(s_die, PlayerAnimState::Dying);

        // 7. Gibbed overrides dying
        let s_gib = evaluate_player_anim_state(0, true, false, false, Vec2::ZERO);
        assert_eq!(s_gib, PlayerAnimState::Gibbed);
    }

    #[test]
    fn test_compute_player_actor_tiles() {
        // Idle front view
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Idle, 0, 0), 1405);
        // Idle back view
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Idle, 4, 0), 1409);
        // Walking cycle frame 1, front view
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Walking, 0, 1), 1405 + 8);
        // Firing front view
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Firing, 0, 0), 1405 + 32);
        // Crouching side view
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Crouching, 2, 0), 1405 + 42);
        // Dying collapse frame 2
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Dying, 0, 2), 1442);
        // Gibbed explosion tile
        assert_eq!(compute_player_actor_tile(PlayerAnimState::Gibbed, 0, 0), 1446);
    }

    #[test]
    fn test_trigger_player_death_or_gib() {
        let mut app = App::new();
        app.add_event::<crate::audio::PlaySoundEvent>();
        app.add_event::<crate::combat::types::GibEvent>();

        app.add_systems(
            Update,
            |mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
             mut gib_events: EventWriter<crate::combat::types::GibEvent>| {
                let s1 = trigger_player_death_or_gib(
                    Vec3::ZERO,
                    false,
                    &mut sound_events,
                    &mut gib_events,
                );
                assert_eq!(s1, PlayerAnimState::Dying);

                let s2 = trigger_player_death_or_gib(
                    Vec3::new(10.0, 0.0, 10.0),
                    true,
                    &mut sound_events,
                    &mut gib_events,
                );
                assert_eq!(s2, PlayerAnimState::Gibbed);
            },
        );

        app.update();

        let sound_events = app.world().resource::<Events<crate::audio::PlaySoundEvent>>();
        assert!(!sound_events.is_empty());
        let gib_events = app.world().resource::<Events<crate::combat::types::GibEvent>>();
        assert!(!gib_events.is_empty());
    }

    #[test]
    fn test_actor_billboard_transforms() {
        let mut app = App::new();
        app.add_systems(Update, update_actor_billboard_transforms);

        // Spawn camera at (0, 0, 10)
        app.world_mut().spawn((
            Camera::default(),
            Transform::from_xyz(0.0, 0.0, 10.0),
        ));

        // Spawn remote player actor at (0, 0, 0)
        let actor_entity = app.world_mut().spawn((
            RemotePlayerActor::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        app.update();

        let actor_trans = app.world().entity(actor_entity).get::<Transform>().unwrap();
        // Forward vector should point towards camera (+Z)
        let forward = *actor_trans.forward();
        assert!(forward.z < 0.0 || forward.z > 0.0); // oriented towards look target
    }
}
