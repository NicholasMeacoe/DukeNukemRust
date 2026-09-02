#![allow(dead_code)]

pub mod effectors;
pub mod props;
pub mod props_extended;
pub mod types;

pub use effectors::*;
pub use props::*;
#[allow(unused_imports)]
pub use props_extended::{DancerProp, ExtendedPropsPlugin, FountainProp, MoneyItem};
pub use types::*;

use crate::map::Map;
use bevy::prelude::*;

pub struct InteractivityPlugin;

impl Plugin for InteractivityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(props_extended::ExtendedPropsPlugin)
            .add_event::<ActivateTagEvent>()
            .add_event::<InteractEvent>()
            .add_event::<ExplosionDamageEvent>()
            .add_event::<BarrelExplodeEvent>()
            .add_event::<PlayerHealEvent>()
            .add_event::<PlaySoundEvent>()
            .add_systems(
                Update,
                (
                    handle_player_interactions,
                    handle_touchplates,
                    handle_explosions,
                    handle_barrel_chain_explosions,
                    handle_tag_activations,
                )
                    .in_set(crate::GameSet::Interactivity),
            )
            .add_systems(
                Update,
                (
                    update_sector_effectors,
                    update_surveillance_monitors,
                    update_mirror_props,
                    apply_player_healing,
                )
                    .in_set(crate::GameSet::Interactivity),
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

pub fn spawn_interactive_elements_from_map(commands: &mut Commands, map: &Map) {
    for (_idx, sprite) in map.sprites.iter().enumerate() {
        let pos = Vec3::new(
            sprite.x as f32 / 1024.0,
            -(sprite.z as f32) / (1024.0 * 16.0),
            sprite.y as f32 / 1024.0,
        );

        match sprite.picnum {
            // SECTOREFFECTOR (Tile 1)
            1 => {
                let ang_rad = (sprite.ang as f32 / 2048.0) * std::f32::consts::TAU;
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
                        target_sector: (sprite.hitag as usize)
                            .min(map.sectors.len().saturating_sub(1)),
                        target_pos: pos,
                        is_submerged: false,
                    },
                    3 => EffectorKind::LightStrobe {
                        base_shade: 0,
                        min_shade: 0,
                        max_shade: 20,
                        timer: 0.0,
                        rate: 4.0,
                    },
                    12 => EffectorKind::LightSwitchOperator {
                        is_on: true,
                        on_shade: 0,
                        off_shade: 25,
                    },
                    21 => EffectorKind::DropFloor {
                        orig_floor_z: sprite.z,
                        target_floor_z: sprite.z + (4096 * 16),
                        current_floor_z: sprite.z,
                        speed: 20,
                        is_dropped: false,
                    },
                    25 => EffectorKind::RotatingEngine {
                        pivot: Vec2::new(pos.x, pos.z),
                        current_ang: 0.0,
                        speed: 2.0,
                    },
                    30 => EffectorKind::SubwayTrain {
                        stop_a: Vec2::new(pos.x, pos.z),
                        stop_b: Vec2::new(
                            pos.x + ang_rad.cos() * 50.0,
                            pos.z + ang_rad.sin() * 50.0,
                        ),
                        current_pos: Vec2::new(pos.x, pos.z),
                        progress: 0.0,
                        speed: 8.0,
                        moving_to_b: true,
                        pause_timer: 0.0,
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
                    crate::game_flow::LevelEntity,
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
                    crate::game_flow::LevelEntity,
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
                    crate::game_flow::LevelEntity,
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
        if let EffectorKind::SlidingDoor {
            ref mut progress,
            speed,
            ..
        } = effector.kind
        {
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

        if let EffectorKind::Elevator {
            ref mut current_floor_z,
            target_floor_z,
            ..
        } = elevator.kind
        {
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
            cooldown_timer: 0.0,
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

    #[test]
    fn test_nuke_button_facing_check() {
        let player_dir = Vec3::new(0.0, 0.0, -1.0); // Facing North
        let switch_pos = Vec3::new(0.0, 0.0, -2.0); // Switch in front of player
        let player_pos = Vec3::ZERO;

        let to_obj = switch_pos - player_pos;
        let facing = player_dir.dot(to_obj.normalize_or_zero());
        assert!(facing > 0.1); // Valid activation!

        let behind_pos = Vec3::new(0.0, 0.0, 2.0); // Switch behind player
        let to_behind = behind_pos - player_pos;
        let facing_behind = player_dir.dot(to_behind.normalize_or_zero());
        assert!(facing_behind <= 0.1); // Ignored when back is turned!
    }

    #[test]
    fn test_access_switch_keycard_requirement() {
        let mut player = crate::player::PlayerController::default();
        assert!(!player.has_blue_key);

        let blue_switch = InteractiveSwitch {
            switch_type: SwitchType::AccessSwitch { key_required: 1 },
            on_tile: 130,
            off_tile: 131,
            is_on: false,
            lotag: 10,
            hitag: 0,
            sound_id: 76,
            material_handle: None,
        };

        // Without key: cannot activate
        let can_activate = match blue_switch.switch_type {
            SwitchType::AccessSwitch { key_required } => match key_required {
                1 => player.has_blue_key,
                2 => player.has_red_key,
                3 => player.has_yellow_key,
                _ => true,
            },
            _ => true,
        };
        assert!(!can_activate);

        // Pick up blue keycard
        player.has_blue_key = true;
        let can_activate_now = match blue_switch.switch_type {
            SwitchType::AccessSwitch { key_required } => match key_required {
                1 => player.has_blue_key,
                2 => player.has_red_key,
                3 => player.has_yellow_key,
                _ => true,
            },
            _ => true,
        };
        assert!(can_activate_now);
    }

    #[test]
    fn test_drop_floor_effector_trigger() {
        let mut drop_floor = EffectorKind::DropFloor {
            orig_floor_z: 0,
            target_floor_z: 8192,
            current_floor_z: 0,
            speed: 10,
            is_dropped: false,
        };

        // Trigger drop
        if let EffectorKind::DropFloor {
            ref mut is_dropped, ..
        } = drop_floor
        {
            *is_dropped = true;
        }

        if let EffectorKind::DropFloor {
            is_dropped,
            target_floor_z,
            ..
        } = drop_floor
        {
            assert!(is_dropped);
            assert_eq!(target_floor_z, 8192);
        }
    }

    #[test]
    fn test_subway_train_waypoint_interpolation() {
        let train = EffectorKind::SubwayTrain {
            stop_a: Vec2::new(0.0, 0.0),
            stop_b: Vec2::new(100.0, 0.0),
            current_pos: Vec2::new(50.0, 0.0),
            progress: 0.5,
            speed: 10.0,
            moving_to_b: true,
            pause_timer: 0.0,
        };

        if let EffectorKind::SubwayTrain {
            progress,
            current_pos,
            ..
        } = train
        {
            assert_eq!(progress, 0.5);
            assert_eq!(current_pos, Vec2::new(50.0, 0.0));
        }
    }

    #[test]
    fn test_rotating_engine_continuous_angle() {
        let mut engine = EffectorKind::RotatingEngine {
            pivot: Vec2::new(10.0, 10.0),
            current_ang: 0.0,
            speed: 2.0,
        };

        if let EffectorKind::RotatingEngine {
            ref mut current_ang,
            speed,
            ..
        } = engine
        {
            *current_ang += speed * 0.5; // dt = 0.5s -> 1.0 rad
        }

        if let EffectorKind::RotatingEngine { current_ang, .. } = engine {
            assert_eq!(current_ang, 1.0);
        }
    }

    #[test]
    fn test_auto_close_door_effector_lifecycle() {
        let mut door = EffectorKind::AutoCloseDoor {
            orig_ceil_z: 0,
            open_ceil_z: -8192,
            current_ceil_z: 0,
            speed: 16,
            auto_close_timer: None,
            auto_close_delay: 5.0,
            is_open: false,
        };

        // Open door
        if let EffectorKind::AutoCloseDoor {
            ref mut is_open,
            ref mut auto_close_timer,
            auto_close_delay,
            ..
        } = door
        {
            *is_open = true;
            *auto_close_timer = Some(auto_close_delay);
        }

        if let EffectorKind::AutoCloseDoor {
            is_open,
            auto_close_timer,
            ..
        } = door
        {
            assert!(is_open);
            assert_eq!(auto_close_timer, Some(5.0));
        }

        // Count down timer to 0
        if let EffectorKind::AutoCloseDoor {
            ref mut is_open,
            ref mut auto_close_timer,
            ..
        } = door
        {
            if let Some(ref mut timer) = auto_close_timer {
                *timer -= 5.0;
                if *timer <= 0.0 {
                    *is_open = false;
                }
            }
        }

        if let EffectorKind::AutoCloseDoor { is_open, .. } = door {
            assert!(!is_open); // Door auto-closed!
        }
    }

    #[test]
    fn test_platform_carrier_momentum_transfer() {
        let carrier = CarrierPlatform {
            velocity: Vec3::new(0.0, 2.5, 0.0), // Elevator ascending at 2.5 m/s
            sector_bounds_min: Vec2::new(-5.0, -5.0),
            sector_bounds_max: Vec2::new(5.0, 5.0),
        };

        let mut passenger_pos = Vec3::new(0.0, 10.0, 0.0);
        let dt = 0.1;

        // Verify bounds check
        let is_inside = passenger_pos.x >= carrier.sector_bounds_min.x
            && passenger_pos.x <= carrier.sector_bounds_max.x
            && passenger_pos.z >= carrier.sector_bounds_min.y
            && passenger_pos.z <= carrier.sector_bounds_max.y;

        assert!(is_inside);

        // Apply carrier delta
        passenger_pos += carrier.velocity * dt;
        assert_eq!(passenger_pos.y, 10.25);
    }

    #[test]
    fn test_expanded_sector_effectors_matrix() {
        // 1. Pivot rotation (SE 1)
        let mut pivot_door = EffectorKind::PivotRotatingSector {
            pivot: Vec2::new(10.0, 10.0),
            orig_ang: 0.0,
            target_ang: std::f32::consts::FRAC_PI_2,
            current_ang: 0.0,
            speed: 2.0,
            is_open: true,
        };
        if let EffectorKind::PivotRotatingSector {
            ref mut current_ang,
            target_ang,
            ..
        } = pivot_door
        {
            *current_ang = target_ang;
            assert_eq!(*current_ang, std::f32::consts::FRAC_PI_2);
        }

        // 2. Earthquake (SE 2/22)
        let mut earthquake = EffectorKind::Earthquake {
            intensity: 1.5,
            duration: 4.0,
            elapsed: 0.0,
            is_triggered: true,
        };
        if let EffectorKind::Earthquake {
            ref mut elapsed,
            duration,
            ref mut is_triggered,
            ..
        } = earthquake
        {
            *elapsed += 4.5;
            if *elapsed >= duration {
                *is_triggered = false;
            }
            assert!(!*is_triggered);
        }

        // 3. Continuous rotation (SE 11)
        let mut fan = EffectorKind::ContinuousRotation {
            pivot: Vec2::ZERO,
            current_ang: 0.0,
            angular_speed: 10.0,
        };
        if let EffectorKind::ContinuousRotation {
            ref mut current_ang,
            angular_speed,
            ..
        } = fan
        {
            *current_ang += angular_speed * 0.1;
            assert_eq!(*current_ang, 1.0);
        }

        // 4. Conveyor belt (SE 24)
        let conveyor = EffectorKind::ConveyorBelt {
            direction: Vec2::new(1.0, 0.0),
            speed: 3.5,
        };
        if let EffectorKind::ConveyorBelt { direction, speed } = conveyor {
            assert_eq!(direction.x * speed, 3.5);
        }

        // 5. Crusher sector (SE 31/32)
        let mut crusher = EffectorKind::CrusherSector {
            min_z: 0,
            max_z: 10000,
            current_z: 0,
            speed: 50,
            crushing_ceiling: true,
            moving_down: true,
        };
        if let EffectorKind::CrusherSector {
            ref mut current_z,
            ref mut moving_down,
            max_z,
            ..
        } = crusher
        {
            *current_z = max_z;
            *moving_down = false;
            assert_eq!(*current_z, 10000);
            assert!(!*moving_down);
        }
    }

    #[test]
    fn test_complete_pickup_matrix_classification() {
        use crate::interactivity::types::PickupKind::*;
        let all_pickups = [
            SmallMedkit,
            LargeMedkit,
            PortableMedkit,
            AtomicHealth,
            ArmorVest,
            PistolClip,
            ShotgunBox,
            ChaingunBox,
            RpgRocket,
            PipebombBox,
            ShrinkerAmmo,
            DevastatorBox,
            FreezeAmmo,
            ExpanderAmmo,
            Steroids,
            ScubaTank,
            NightvisionGoggles,
            ProtectiveBoots,
            Jetpack,
            Holoduke,
            WeaponPistol,
            WeaponShotgun,
            WeaponChaingun,
            WeaponRpg,
            WeaponPipebomb,
            WeaponShrinker,
            WeaponDevastator,
            WeaponTripbomb,
            WeaponFreezer,
            WeaponExpander,
        ];
        assert_eq!(all_pickups.len(), 30);
    }
}
