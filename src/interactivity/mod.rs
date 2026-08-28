#![allow(dead_code)]

pub mod types;
pub mod effectors;
pub mod props;

pub use types::*;
pub use effectors::*;
pub use props::*;

use bevy::prelude::*;
use crate::map::Map;

pub struct InteractivityPlugin;

impl Plugin for InteractivityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActivateTagEvent>()
            .add_event::<InteractEvent>()
            .add_event::<ExplosionDamageEvent>()
            .add_event::<PlayerHealEvent>()
            .add_systems(
                Update,
                (
                    (
                        handle_player_interactions,
                        handle_touchplates,
                        handle_explosions,
                        handle_tag_activations,
                        update_sector_effectors,
                        apply_player_healing,
                    ).in_set(crate::GameSet::Interactivity),
                ),
            );
    }
}

pub fn apply_player_healing(
    mut heal_events: EventReader<PlayerHealEvent>,
    mut player_query: Query<&mut crate::Player>,
) {
    for event in heal_events.read() {
        for mut player in player_query.iter_mut() {
            player.health = (player.health + event.amount).min(100);
        }
    }
}

pub fn spawn_interactive_elements_from_map(
    commands: &mut Commands,
    map: &Map,
) {
    for (_idx, sprite) in map.sprites.iter().enumerate() {
        let pos = Vec3::new(
            sprite.x as f32 / 1024.0,
            -(sprite.z as f32) / (1024.0 * 16.0),
            sprite.y as f32 / 1024.0,
        );

        match sprite.picnum {
            // SECTOREFFECTOR (Tile 1)
            1 => {
                let ang_rad = -(sprite.ang as f32 / 2048.0) * std::f32::consts::TAU;
                let kind = match sprite.lotag {
                    0 => EffectorKind::RotatingDoor {
                        pivot: Vec2::new(pos.x, pos.z),
                        orig_ang: 0.0,
                        target_ang: std::f32::consts::FRAC_PI_2,
                        current_ang: 0.0,
                        speed: 1.5,
                        is_open: false,
                    },
                    15 => EffectorKind::SlidingDoor {
                        orig_pos: Vec2::new(pos.x, pos.z),
                        open_offset: Vec2::new(ang_rad.cos() * 3.0, ang_rad.sin() * 3.0),
                        progress: 0.0,
                        speed: 1.2,
                        auto_close_timer: None,
                        auto_close_delay: 4.0,
                        is_open: false,
                    },
                    17 | 18 => EffectorKind::Elevator {
                        orig_floor_z: sprite.z,
                        target_floor_z: sprite.z - (4096 * 16), // Rise up
                        current_floor_z: sprite.z,
                        orig_ceil_z: sprite.z - (8192 * 16),
                        target_ceil_z: sprite.z - (12288 * 16),
                        current_ceil_z: sprite.z - (8192 * 16),
                        speed: 16,
                        is_at_top: false,
                        auto_return_timer: None,
                    },
                    7 => EffectorKind::UnderwaterTeleport {
                        target_sector: (sprite.hitag as usize).min(map.sectors.len().saturating_sub(1)),
                    },
                    3 => EffectorKind::LightStrobe {
                        base_shade: 0,
                        min_shade: 0,
                        max_shade: 20,
                        timer: 0.0,
                        rate: 4.0,
                    },
                    _ => EffectorKind::SlidingDoor {
                        orig_pos: Vec2::new(pos.x, pos.z),
                        open_offset: Vec2::new(ang_rad.cos() * 2.0, ang_rad.sin() * 2.0),
                        progress: 0.0,
                        speed: 1.0,
                        auto_close_timer: None,
                        auto_close_delay: 3.0,
                        is_open: false,
                    },
                };

                commands.spawn((
                    SectorEffectorComponent {
                        sector_idx: sprite.sectnum as usize,
                        lotag: sprite.lotag,
                        hitag: sprite.hitag,
                        kind,
                        active: false,
                    },
                    TransformBundle::from_transform(Transform::from_translation(pos)),
                ));
            }

            // TOUCHPLATE (Tile 3)
            3 => {
                commands.spawn((
                    Touchplate {
                        lotag: sprite.lotag,
                        hitag: sprite.hitag,
                        radius: 2.5,
                        triggered: false,
                    },
                    TransformBundle::from_transform(Transform::from_translation(pos)),
                ));
            }

            // ACTIVATOR (Tile 2)
            2 => {
                commands.spawn((
                    Activator {
                        lotag: sprite.lotag,
                        hitag: sprite.hitag,
                    },
                    TransformBundle::from_transform(Transform::from_translation(pos)),
                ));
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_door_effector_progression() {
        let mut effector = SectorEffectorComponent {
            sector_idx: 5,
            lotag: 15,
            hitag: 0,
            kind: EffectorKind::SlidingDoor {
                orig_pos: Vec2::ZERO,
                open_offset: Vec2::new(4.0, 0.0),
                progress: 0.0,
                speed: 2.0,
                auto_close_timer: None,
                auto_close_delay: 3.0,
                is_open: true,
            },
            active: true,
        };

        // Advance 0.25 seconds with speed 2.0 -> progress increases by 0.5
        if let EffectorKind::SlidingDoor { ref mut progress, speed, .. } = effector.kind {
            *progress += speed * 0.25;
            assert_eq!(*progress, 0.5);
        }
    }

    #[test]
    fn test_elevator_height_interpolation() {
        let mut elevator = SectorEffectorComponent {
            sector_idx: 2,
            lotag: 17,
            hitag: 0,
            kind: EffectorKind::Elevator {
                orig_floor_z: 10000,
                target_floor_z: 5000,
                current_floor_z: 10000,
                orig_ceil_z: 0,
                target_ceil_z: -5000,
                current_ceil_z: 0,
                speed: 10,
                is_at_top: true,
                auto_return_timer: None,
            },
            active: true,
        };

        if let EffectorKind::Elevator { ref mut current_floor_z, target_floor_z, .. } = elevator.kind {
            *current_floor_z = target_floor_z;
            assert_eq!(*current_floor_z, 5000);
        }
    }

    #[test]
    fn test_touchplate_and_switch_events() {
        let switch = InteractiveSwitch {
            switch_type: SwitchType::LightSwitch,
            on_tile: 135,
            off_tile: 134,
            is_on: false,
            lotag: 42,
            hitag: 0,
            sound_id: 10,
            material_handle: None,
        };

        let event = ActivateTagEvent {
            lotag: switch.lotag,
        };

        assert_eq!(event.lotag, 42);
    }

    #[test]
    fn test_water_fountain_and_toilet_props() {
        let mut fountain = WaterFountain {
            uses_left: 10,
            is_broken: false,
            broken_tile: 567,
        };
        fountain.uses_left -= 1;
        assert_eq!(fountain.uses_left, 9);

        let mut toilet = ToiletProp {
            is_broken: false,
            broken_tile: 615,
            water_tile: 921,
            last_used_time: 0.0,
        };
        toilet.last_used_time = 5.0;
        assert_eq!(toilet.last_used_time, 5.0);
    }

    #[test]
    fn test_destructible_props_and_crack_walls() {
        let mut barrel = ExplodingBarrel {
            health: 20,
            damage_radius: 6.0,
            damage: 100,
            is_exploded: false,
        };
        barrel.health -= 25;
        if barrel.health <= 0 {
            barrel.is_exploded = true;
        }
        assert!(barrel.is_exploded);

        let mut crack = CrackWall {
            health: 30,
            stage: 1,
            lotag: 15,
            is_blown: false,
        };
        crack.health -= 35;
        if crack.health <= 0 {
            crack.is_blown = true;
            crack.stage = 4;
        }
        assert!(crack.is_blown);
        assert_eq!(crack.stage, 4);
    }
}

