#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;
use crate::interactivity::types::*;

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DynamicSectorMesh {
    pub sector_idx: usize,
    pub orig_translation: Vec3,
}

enum EffectorTransform {
    Rotate { pivot: Vec2, rot_ang: f32 },
    Slide { offset: Vec2 },
    Elevate { delta_y: f32 },
}

pub fn handle_tag_activations(
    mut events: EventReader<ActivateTagEvent>,
    mut effectors: Query<&mut SectorEffectorComponent>,
) {
    for event in events.read() {
        for mut effector in effectors.iter_mut() {
            if effector.lotag == event.lotag || (effector.hitag != 0 && effector.hitag == event.lotag) {
                effector.active = true;
                match &mut effector.kind {
                    EffectorKind::RotatingDoor { is_open, .. } => {
                        *is_open = !*is_open;
                    }
                    EffectorKind::SlidingDoor { is_open, auto_close_timer, auto_close_delay, .. } => {
                        *is_open = !*is_open;
                        if *is_open {
                            *auto_close_timer = Some(*auto_close_delay);
                        }
                    }
                    EffectorKind::Elevator { is_at_top, auto_return_timer, .. } => {
                        *is_at_top = !*is_at_top;
                        *auto_return_timer = if *is_at_top { Some(5.0) } else { None };
                    }
                    EffectorKind::DropFloor { is_dropped, .. } => {
                        *is_dropped = true;
                    }
                    EffectorKind::LightSwitchOperator { is_on, .. } => {
                        *is_on = !*is_on;
                    }
                    EffectorKind::AutoCloseDoor { is_open, auto_close_timer, auto_close_delay, .. } => {
                        *is_open = !*is_open;
                        if *is_open {
                            *auto_close_timer = Some(*auto_close_delay);
                        }
                    }
                    EffectorKind::PivotRotatingSector { is_open, .. } => {
                        *is_open = !*is_open;
                    }
                    EffectorKind::Earthquake { is_triggered, elapsed, .. } => {
                        *is_triggered = true;
                        *elapsed = 0.0;
                    }
                    EffectorKind::StretchCeiling { is_stretched, .. } => {
                        *is_stretched = !*is_stretched;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn update_sector_effectors(
    time: Res<Time>,
    mut effectors: Query<&mut SectorEffectorComponent>,
    mut sector_meshes: Query<(&DynamicSectorMesh, &mut Transform)>,
) {
    let dt = time.delta_seconds();
    let mut transforms = HashMap::new();

    // Pass 1: O(Effectors) logic update
    for mut effector in effectors.iter_mut() {
        if !effector.active {
            continue;
        }

        let sector_idx = effector.sector_idx;
        let mut should_deactivate = false;

        match &mut effector.kind {
            EffectorKind::RotatingDoor {
                pivot,
                orig_ang,
                target_ang,
                current_ang,
                speed,
                is_open,
                ..
            } => {
                let dest = if *is_open { *target_ang } else { *orig_ang };
                let step = *speed * dt;
                if (*current_ang - dest).abs() <= step {
                    *current_ang = dest;
                    should_deactivate = true;
                } else {
                    *current_ang += step * (dest - *current_ang).signum();
                }

                transforms.insert(
                    sector_idx,
                    EffectorTransform::Rotate {
                        pivot: *pivot,
                        rot_ang: *current_ang,
                    },
                );
            }

            EffectorKind::SlidingDoor {
                open_offset,
                progress,
                speed,
                auto_close_timer,
                is_open,
                ..
            } => {
                let target_prog = if *is_open { 1.0 } else { 0.0 };
                let step = *speed * dt;
                if (*progress - target_prog).abs() <= step {
                    *progress = target_prog;
                    if !*is_open {
                        should_deactivate = true;
                    } else if *progress == 1.0 {
                        // Timer ticks only when fully open
                        if let Some(ref mut timer) = auto_close_timer {
                            *timer -= dt;
                            if *timer <= 0.0 {
                                *is_open = false;
                                *auto_close_timer = None;
                            }
                        }
                    }
                } else {
                    *progress += step * (target_prog - *progress).signum();
                }

                transforms.insert(
                    sector_idx,
                    EffectorTransform::Slide {
                        offset: *open_offset * *progress,
                    },
                );
            }

            EffectorKind::Elevator {
                orig_floor_z,
                target_floor_z,
                current_floor_z,
                speed,
                is_at_top,
                auto_return_timer,
                ..
            } => {
                let target_z = if *is_at_top { *target_floor_z } else { *orig_floor_z };
                let step = (*speed as f32 * dt * 1000.0) as i32;
                if (*current_floor_z - target_z).abs() <= step.max(1) {
                    *current_floor_z = target_z;
                    if !*is_at_top && *current_floor_z == *orig_floor_z {
                        should_deactivate = true;
                    } else if *is_at_top {
                        // Return timer ticks only when fully elevated
                        if let Some(ref mut timer) = auto_return_timer {
                            *timer -= dt;
                            if *timer <= 0.0 {
                                *is_at_top = false;
                                *auto_return_timer = None;
                            }
                        } else {
                            should_deactivate = true;
                        }
                    }
                } else {
                    *current_floor_z += step.max(1) * (target_z - *current_floor_z).signum();
                }

                // Convert Build Z change to World Y
                let delta_y = -((*current_floor_z - *orig_floor_z) as f32) / (1024.0 * 16.0);
                transforms.insert(sector_idx, EffectorTransform::Elevate { delta_y });
            }

            EffectorKind::DropFloor {
                orig_floor_z,
                target_floor_z,
                current_floor_z,
                speed,
                is_dropped,
            } => {
                if *is_dropped {
                    let step = (*speed as f32 * dt * 1000.0) as i32;
                    if (*current_floor_z - *target_floor_z).abs() <= step.max(1) {
                        *current_floor_z = *target_floor_z;
                        should_deactivate = true;
                    } else {
                        *current_floor_z += step.max(1) * (*target_floor_z - *current_floor_z).signum();
                    }

                    let delta_y = -((*current_floor_z - *orig_floor_z) as f32) / (1024.0 * 16.0);
                    transforms.insert(sector_idx, EffectorTransform::Elevate { delta_y });
                }
            }

            EffectorKind::RotatingEngine { pivot, current_ang, speed } => {
                *current_ang += *speed * dt;
                transforms.insert(
                    sector_idx,
                    EffectorTransform::Rotate {
                        pivot: *pivot,
                        rot_ang: *current_ang,
                    },
                );
            }

            EffectorKind::SubwayTrain {
                stop_a,
                stop_b,
                current_pos,
                progress,
                speed,
                moving_to_b,
                pause_timer,
            } => {
                if *pause_timer > 0.0 {
                    *pause_timer -= dt;
                } else {
                    let step = *speed * dt;
                    if *moving_to_b {
                        *progress = (*progress + step).min(1.0);
                        if *progress >= 1.0 {
                            *moving_to_b = false;
                            *pause_timer = 4.0; // Wait 4s at station B
                        }
                    } else {
                        *progress = (*progress - step).max(0.0);
                        if *progress <= 0.0 {
                            *moving_to_b = true;
                            *pause_timer = 4.0; // Wait 4s at station A
                        }
                    }

                    let offset = (*stop_b - *stop_a) * *progress;
                    *current_pos = *stop_a + offset;
                    transforms.insert(sector_idx, EffectorTransform::Slide { offset });
                }
            }

            EffectorKind::LightStrobe { timer, rate, .. } => {
                *timer += dt * *rate;
            }

            EffectorKind::AutoCloseDoor {
                orig_ceil_z,
                open_ceil_z,
                current_ceil_z,
                speed,
                auto_close_timer,
                auto_close_delay: _,
                is_open,
            } => {
                let target_z = if *is_open { *open_ceil_z } else { *orig_ceil_z };
                let diff = target_z - *current_ceil_z;
                if diff != 0 {
                    let step = (*speed as f32 * dt * 1024.0) as i32;
                    if diff.abs() <= step {
                        *current_ceil_z = target_z;
                    } else {
                        *current_ceil_z += diff.signum() * step;
                    }
                } else if *is_open {
                    // Count down auto-close timer
                    if let Some(ref mut timer) = auto_close_timer {
                        *timer -= dt;
                        if *timer <= 0.0 {
                            *auto_close_timer = None;
                            *is_open = false; // Auto close door!
                        }
                    }
                } else {
                    should_deactivate = true;
                }

                // Delta Y in meters
                let delta_z = *current_ceil_z - *orig_ceil_z;
                let delta_y = -(delta_z as f32) / (1024.0 * 16.0);
                transforms.insert(sector_idx, EffectorTransform::Elevate { delta_y });
            }

            EffectorKind::PivotRotatingSector {
                pivot,
                orig_ang,
                target_ang,
                current_ang,
                speed,
                is_open,
            } => {
                let dest = if *is_open { *target_ang } else { *orig_ang };
                let step = *speed * dt;
                if (*current_ang - dest).abs() <= step {
                    *current_ang = dest;
                    should_deactivate = true;
                } else {
                    *current_ang += step * (dest - *current_ang).signum();
                }

                transforms.insert(
                    sector_idx,
                    EffectorTransform::Rotate {
                        pivot: *pivot,
                        rot_ang: *current_ang,
                    },
                );
            }

            EffectorKind::Earthquake {
                duration,
                elapsed,
                is_triggered,
                ..
            } => {
                if *is_triggered {
                    *elapsed += dt;
                    if *elapsed >= *duration {
                        *is_triggered = false;
                        should_deactivate = true;
                    }
                }
            }

            EffectorKind::RandomFlicker { timer, is_buzz, .. } => {
                *timer += dt * if *is_buzz { 30.0 } else { 10.0 };
            }

            EffectorKind::ContinuousRotation {
                pivot,
                current_ang,
                angular_speed,
            } => {
                *current_ang += *angular_speed * dt;
                transforms.insert(
                    sector_idx,
                    EffectorTransform::Rotate {
                        pivot: *pivot,
                        rot_ang: *current_ang,
                    },
                );
            }

            EffectorKind::GlowGradient {
                min_shade,
                max_shade,
                current_shade,
                rate,
                increasing,
            } => {
                let step = *rate * dt;
                if *increasing {
                    *current_shade += step;
                    if *current_shade >= *max_shade as f32 {
                        *current_shade = *max_shade as f32;
                        *increasing = false;
                    }
                } else {
                    *current_shade -= step;
                    if *current_shade <= *min_shade as f32 {
                        *current_shade = *min_shade as f32;
                        *increasing = true;
                    }
                }
            }

            EffectorKind::StretchCeiling {
                orig_ceil_z,
                target_ceil_z,
                current_ceil_z,
                speed,
                is_stretched,
            } => {
                let target = if *is_stretched { *target_ceil_z } else { *orig_ceil_z };
                let step = (*speed as f32 * dt * 1024.0) as i32;
                let diff = target - *current_ceil_z;
                if diff.abs() <= step {
                    *current_ceil_z = target;
                    should_deactivate = true;
                } else {
                    *current_ceil_z += diff.signum() * step;
                }
                let delta_y = -((*current_ceil_z - *orig_ceil_z) as f32) / (1024.0 * 16.0);
                transforms.insert(sector_idx, EffectorTransform::Elevate { delta_y });
            }

            EffectorKind::ConveyorBelt { .. } => {
                // Conveyor velocity handled by platform / player controller
            }

            EffectorKind::CrusherSector {
                min_z,
                max_z,
                current_z,
                speed,
                moving_down,
                ..
            } => {
                let step = (*speed as f32 * dt * 1024.0) as i32;
                if *moving_down {
                    *current_z += step;
                    if *current_z >= *max_z {
                        *current_z = *max_z;
                        *moving_down = false;
                    }
                } else {
                    *current_z -= step;
                    if *current_z <= *min_z {
                        *current_z = *min_z;
                        *moving_down = true;
                    }
                }
                let delta_y = -((*current_z - *min_z) as f32) / (1024.0 * 16.0);
                transforms.insert(sector_idx, EffectorTransform::Elevate { delta_y });
            }

            EffectorKind::ShootingGlassPane { health, is_shattered } => {
                if *health <= 0 && !*is_shattered {
                    *is_shattered = true;
                    should_deactivate = true;
                }
            }

            _ => {}
        }

        if should_deactivate {
            effector.active = false;
        }
    }

    if transforms.is_empty() {
        return;
    }

    // Pass 2: O(Meshes) single pass
    for (dyn_mesh, mut transform) in sector_meshes.iter_mut() {
        if let Some(eff_transform) = transforms.get(&dyn_mesh.sector_idx) {
            match eff_transform {
                EffectorTransform::Rotate { pivot, rot_ang } => {
                    let pivot_3d = Vec3::new(pivot.x, dyn_mesh.orig_translation.y, pivot.y);
                    let rel_pos = dyn_mesh.orig_translation - pivot_3d;
                    transform.translation = pivot_3d + Quat::from_rotation_y(*rot_ang) * rel_pos;
                    transform.rotation = Quat::from_rotation_y(*rot_ang);
                }
                EffectorTransform::Slide { offset } => {
                    transform.translation = dyn_mesh.orig_translation + Vec3::new(offset.x, 0.0, offset.y);
                }
                EffectorTransform::Elevate { delta_y } => {
                    transform.translation = dyn_mesh.orig_translation + Vec3::new(0.0, *delta_y, 0.0);
                }
            }
        }
    }
}
