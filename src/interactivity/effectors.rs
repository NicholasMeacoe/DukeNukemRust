#![allow(dead_code)]

use bevy::prelude::*;
use crate::interactivity::types::*;

#[derive(Component, Debug, Clone)]
pub struct DynamicSectorMesh {
    pub sector_idx: usize,
    pub orig_translation: Vec3,
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
                        *auto_return_timer = Some(5.0); // 5-second return
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
                } else if *current_ang < dest {
                    *current_ang += step;
                } else {
                    *current_ang -= step;
                }

                let rot_ang = *current_ang;
                let piv = *pivot;
                // Apply rotation around pivot point (Px, Pz)
                for (dyn_mesh, mut transform) in sector_meshes.iter_mut() {
                    if dyn_mesh.sector_idx == sector_idx {
                        let pivot_3d = Vec3::new(piv.x, dyn_mesh.orig_translation.y, piv.y);
                        let rel_pos = dyn_mesh.orig_translation - pivot_3d;
                        transform.translation = pivot_3d + Quat::from_rotation_y(rot_ang) * rel_pos;
                        transform.rotation = Quat::from_rotation_y(rot_ang);
                    }
                }
            }

            EffectorKind::SlidingDoor {
                open_offset,
                progress,
                speed,
                auto_close_timer,
                is_open,
                ..
            } => {
                // Check auto-close countdown
                if *is_open {
                    if let Some(ref mut timer) = auto_close_timer {
                        *timer -= dt;
                        if *timer <= 0.0 {
                            *is_open = false;
                            *auto_close_timer = None;
                        }
                    }
                }

                let target_prog = if *is_open { 1.0 } else { 0.0 };
                let step = *speed * dt;
                if (*progress - target_prog).abs() <= step {
                    *progress = target_prog;
                    if !*is_open && *progress == 0.0 {
                        should_deactivate = true;
                    }
                } else if *progress < target_prog {
                    *progress += step;
                } else {
                    *progress -= step;
                }

                let cur_offset = *open_offset * *progress;
                for (dyn_mesh, mut transform) in sector_meshes.iter_mut() {
                    if dyn_mesh.sector_idx == sector_idx {
                        transform.translation = dyn_mesh.orig_translation
                            + Vec3::new(cur_offset.x, 0.0, cur_offset.y);
                    }
                }
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
                if let Some(ref mut timer) = auto_return_timer {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        *is_at_top = false;
                        *auto_return_timer = None;
                    }
                }

                let target_z = if *is_at_top { *target_floor_z } else { *orig_floor_z };
                let step = (*speed as f32 * dt * 1000.0) as i32;
                if (*current_floor_z - target_z).abs() <= step.max(1) {
                    *current_floor_z = target_z;
                    if !*is_at_top && *current_floor_z == *orig_floor_z {
                        should_deactivate = true;
                    }
                } else if *current_floor_z < target_z {
                    *current_floor_z += step.max(1);
                } else {
                    *current_floor_z -= step.max(1);
                }

                // Convert Build Z change to World Y
                let delta_y = -((*current_floor_z - *orig_floor_z) as f32) / (1024.0 * 16.0);
                for (dyn_mesh, mut transform) in sector_meshes.iter_mut() {
                    if dyn_mesh.sector_idx == sector_idx {
                        transform.translation = dyn_mesh.orig_translation + Vec3::new(0.0, delta_y, 0.0);
                    }
                }
            }

            EffectorKind::LightStrobe {
                timer,
                rate,
                ..
            } => {
                *timer += dt * *rate;
            }

            _ => {}
        }

        if should_deactivate {
            effector.active = false;
        }
    }
}
