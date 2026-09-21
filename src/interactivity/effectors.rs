use crate::interactivity::types::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
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
            if effector.hitag != 0 && effector.hitag == event.lotag {
                effector.active = true;
                match &mut effector.kind {
                    EffectorKind::RotatingDoor { is_open, .. } => {
                        *is_open = !*is_open;
                    }
                    EffectorKind::SlidingDoor {
                        is_open,
                        auto_close_timer,
                        auto_close_delay,
                        ..
                    } => {
                        *is_open = !*is_open;
                        if *is_open {
                            *auto_close_timer = Some(*auto_close_delay);
                        }
                    }
                    EffectorKind::Elevator {
                        is_at_top,
                        auto_return_timer,
                        ..
                    } => {
                        *is_at_top = !*is_at_top;
                        *auto_return_timer = if *is_at_top { Some(5.0) } else { None };
                    }
                    EffectorKind::DropFloor { is_dropped, .. } => {
                        *is_dropped = true;
                    }
                    EffectorKind::LightSwitchOperator { is_on, .. } => {
                        *is_on = !*is_on;
                    }
                    EffectorKind::AutoCloseDoor {
                        is_open,
                        auto_close_timer,
                        auto_close_delay,
                        ..
                    } => {
                        *is_open = !*is_open;
                        if *is_open {
                            *auto_close_timer = Some(*auto_close_delay);
                        }
                    }
                    EffectorKind::PivotRotatingSector { is_open, .. } => {
                        *is_open = !*is_open;
                    }
                    EffectorKind::Earthquake {
                        is_triggered,
                        elapsed,
                        ..
                    } => {
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
    mut commands: Commands,
    time: Res<Time>,
    mut effectors: Query<&mut SectorEffectorComponent>,
    mut sector_meshes: Query<(
        Entity,
        &DynamicSectorMesh,
        &mut Transform,
        Option<&mut CarrierPlatform>,
        Option<&Handle<Mesh>>,
    )>,
    mut shake: Option<ResMut<EarthquakeCameraShake>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
) {
    let dt = time.delta_seconds();
    let mut transforms: HashMap<(usize, SectorMeshPart), EffectorTransform> = HashMap::new();
    let mut carrier_velocities: HashMap<usize, Vec3> = HashMap::new();
    let mut sector_shades: HashMap<usize, i8> = HashMap::new();

    const ALL_PARTS: &[SectorMeshPart] = &[
        SectorMeshPart::Floor,
        SectorMeshPart::Ceiling,
        SectorMeshPart::UpperWall,
        SectorMeshPart::LowerWall,
        SectorMeshPart::MiddleWall,
    ];

    // Collect carrier platform velocities for moving sector effectors
    for effector in effectors.iter() {
        match &effector.kind {
            EffectorKind::Elevator {
                orig_floor_z,
                target_floor_z,
                current_floor_z,
                speed,
                is_at_top,
                ..
            } => {
                if effector.active {
                    let target_z = if *is_at_top {
                        *target_floor_z
                    } else {
                        *orig_floor_z
                    };
                    let diff = target_z - *current_floor_z;
                    if diff != 0 {
                        let speed_z = (*speed as f32 * 1000.0) * diff.signum() as f32;
                        let vel_y = -speed_z / (1024.0 * 16.0);
                        carrier_velocities.insert(effector.sector_idx, Vec3::new(0.0, vel_y, 0.0));
                    } else {
                        carrier_velocities.entry(effector.sector_idx).or_insert(Vec3::ZERO);
                    }
                } else {
                    carrier_velocities.entry(effector.sector_idx).or_insert(Vec3::ZERO);
                }
            }
            EffectorKind::SubwayTrain {
                stop_a,
                stop_b,
                speed,
                moving_to_b,
                pause_timer,
                ..
            } => {
                if effector.active && *pause_timer <= 0.0 {
                    let dir = if *moving_to_b { 1.0 } else { -1.0 };
                    let travel_vec = *stop_b - *stop_a;
                    let vel_2d = travel_vec * (*speed * dir);
                    carrier_velocities.insert(effector.sector_idx, Vec3::new(vel_2d.x, 0.0, vel_2d.y));
                } else {
                    carrier_velocities.entry(effector.sector_idx).or_insert(Vec3::ZERO);
                }
            }
            EffectorKind::ConveyorBelt { direction, speed } => {
                let vel = Vec3::new(direction.x * *speed, 0.0, direction.y * *speed);
                carrier_velocities.insert(effector.sector_idx, vel);
            }
            EffectorKind::DropFloor {
                target_floor_z,
                current_floor_z,
                speed,
                is_dropped,
                ..
            } => {
                if effector.active && *is_dropped {
                    let diff = *target_floor_z - *current_floor_z;
                    if diff != 0 {
                        let speed_z = (*speed as f32 * 1000.0) * diff.signum() as f32;
                        let vel_y = -speed_z / (1024.0 * 16.0);
                        carrier_velocities.insert(effector.sector_idx, Vec3::new(0.0, vel_y, 0.0));
                    } else {
                        carrier_velocities.entry(effector.sector_idx).or_insert(Vec3::ZERO);
                    }
                } else {
                    carrier_velocities.entry(effector.sector_idx).or_insert(Vec3::ZERO);
                }
            }
            _ => {}
        }
    }

    // Pass 1: O(Effectors) logic update
    for mut effector in effectors.iter_mut() {
        if !effector.active {
            continue;
        }

        let sector_idx = effector.sector_idx;
        let mut should_deactivate = false;

        let mut apply_transform = |parts: &[SectorMeshPart], transform: EffectorTransform| {
            for &part in parts {
                transforms.insert((sector_idx, part), transform);
            }
        };

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

                apply_transform(
                    ALL_PARTS,
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

                apply_transform(
                    ALL_PARTS,
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
                let target_z = if *is_at_top {
                    *target_floor_z
                } else {
                    *orig_floor_z
                };
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
                apply_transform(
                    &[SectorMeshPart::Floor, SectorMeshPart::LowerWall],
                    EffectorTransform::Elevate { delta_y },
                );
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
                        *current_floor_z +=
                            step.max(1) * (*target_floor_z - *current_floor_z).signum();
                    }

                    let delta_y = -((*current_floor_z - *orig_floor_z) as f32) / (1024.0 * 16.0);
                    apply_transform(
                        &[SectorMeshPart::Floor, SectorMeshPart::LowerWall],
                        EffectorTransform::Elevate { delta_y },
                    );
                }
            }

            EffectorKind::RotatingEngine {
                pivot,
                current_ang,
                speed,
            } => {
                *current_ang += *speed * dt;
                apply_transform(
                    ALL_PARTS,
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
                    apply_transform(ALL_PARTS, EffectorTransform::Slide { offset });
                }
            }

            EffectorKind::LightStrobe {
                timer,
                rate,
                min_shade,
                max_shade,
                ..
            } => {
                *timer += dt * *rate;
                let is_bright = (*timer as i32 % 2) == 0;
                let shade = if is_bright { *min_shade } else { *max_shade };
                sector_shades.insert(sector_idx, shade);
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
                apply_transform(
                    &[SectorMeshPart::Ceiling, SectorMeshPart::UpperWall],
                    EffectorTransform::Elevate { delta_y },
                );
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

                apply_transform(
                    ALL_PARTS,
                    EffectorTransform::Rotate {
                        pivot: *pivot,
                        rot_ang: *current_ang,
                    },
                );
            }

            EffectorKind::Earthquake {
                intensity,
                duration,
                elapsed,
                is_triggered,
            } => {
                if *is_triggered {
                    *elapsed += dt;
                    if *elapsed >= *duration {
                        *is_triggered = false;
                        should_deactivate = true;
                    }
                }
                if let Some(ref mut s) = shake {
                    if *is_triggered && !should_deactivate {
                        let decay = if *duration > 0.0 {
                            (1.0 - (*elapsed / *duration)).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        s.intensity = *intensity * decay;
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let rx = rng.gen_range(-1.0..=1.0);
                        let ry = rng.gen_range(-1.0..=1.0);
                        let rz = rng.gen_range(-1.0..=1.0);
                        s.offset = Vec3::new(rx, ry, rz) * s.intensity;
                    } else if should_deactivate {
                        s.intensity = 0.0;
                        s.offset = Vec3::ZERO;
                    }
                }
            }

            EffectorKind::RandomFlicker {
                timer,
                is_buzz,
                min_shade,
                max_shade,
                ..
            } => {
                *timer += dt * if *is_buzz { 30.0 } else { 10.0 };
                let flicker_on = (*timer * 7.91).sin() > 0.0;
                let shade = if flicker_on { *min_shade } else { *max_shade };
                sector_shades.insert(sector_idx, shade);
            }

            EffectorKind::ContinuousRotation {
                pivot,
                current_ang,
                angular_speed,
            } => {
                *current_ang += *angular_speed * dt;
                apply_transform(
                    ALL_PARTS,
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
                sector_shades.insert(sector_idx, *current_shade as i8);
            }

            EffectorKind::LightSwitchOperator {
                is_on,
                on_shade,
                off_shade,
            } => {
                let shade = if *is_on { *on_shade } else { *off_shade };
                sector_shades.insert(sector_idx, shade);
            }

            EffectorKind::StretchCeiling {
                orig_ceil_z,
                target_ceil_z,
                current_ceil_z,
                speed,
                is_stretched,
            } => {
                let target = if *is_stretched {
                    *target_ceil_z
                } else {
                    *orig_ceil_z
                };
                let step = (*speed as f32 * dt * 1024.0) as i32;
                let diff = target - *current_ceil_z;
                if diff.abs() <= step {
                    *current_ceil_z = target;
                    should_deactivate = true;
                } else {
                    *current_ceil_z += diff.signum() * step;
                }
                let delta_y = -((*current_ceil_z - *orig_ceil_z) as f32) / (1024.0 * 16.0);
                apply_transform(
                    &[SectorMeshPart::Ceiling, SectorMeshPart::UpperWall],
                    EffectorTransform::Elevate { delta_y },
                );
            }

            EffectorKind::ConveyorBelt { .. } => {
                // Conveyor velocity handled by platform / player controller
            }

            EffectorKind::CrusherSector {
                min_z,
                max_z,
                current_z,
                speed,
                crushing_ceiling,
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
                if *crushing_ceiling {
                    apply_transform(
                        &[SectorMeshPart::Ceiling, SectorMeshPart::UpperWall],
                        EffectorTransform::Elevate { delta_y },
                    );
                } else {
                    apply_transform(
                        &[SectorMeshPart::Floor, SectorMeshPart::LowerWall],
                        EffectorTransform::Elevate { delta_y },
                    );
                }
            }

            EffectorKind::ShootingGlassPane {
                health,
                is_shattered,
            } => {
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

    if transforms.is_empty() && carrier_velocities.is_empty() && sector_shades.is_empty() {
        return;
    }

    // Pass 2: O(Meshes) single pass
    for (entity, dyn_mesh, mut transform, mut maybe_carrier, maybe_mesh) in sector_meshes.iter_mut() {
        if let Some(&shade) = sector_shades.get(&dyn_mesh.sector_idx) {
            if let Some(ref mut meshes_assets) = meshes {
                if let Some(mesh_handle) = maybe_mesh {
                    if let Some(mesh) = meshes_assets.get_mut(mesh_handle) {
                        let tint = crate::palette::Palette::authentic_shade_to_tint(shade);
                        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(ref mut colors)) =
                            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
                        {
                            for c in colors.iter_mut() {
                                *c = tint;
                            }
                        }
                    }
                }
            }
        }
        if dyn_mesh.part == SectorMeshPart::Floor {
            if let Some(&vel) = carrier_velocities.get(&dyn_mesh.sector_idx) {
                if let Some(ref mut carrier) = maybe_carrier {
                    carrier.velocity = vel;
                    carrier.sector_idx = Some(dyn_mesh.sector_idx);
                } else {
                    commands.entity(entity).insert(CarrierPlatform {
                        velocity: vel,
                        sector_idx: Some(dyn_mesh.sector_idx),
                        ..default()
                    });
                }
            }
        }

        if let Some(eff_transform) = transforms.get(&(dyn_mesh.sector_idx, dyn_mesh.part)) {
            match eff_transform {
                EffectorTransform::Rotate { pivot, rot_ang } => {
                    let pivot_3d = Vec3::new(pivot.x, 0.0, pivot.y);
                    let rel_pos = -pivot_3d;
                    transform.translation = pivot_3d + Quat::from_rotation_y(*rot_ang) * rel_pos;
                    transform.rotation = Quat::from_rotation_y(*rot_ang);
                }
                EffectorTransform::Slide { offset } => {
                    transform.translation = Vec3::new(offset.x, 0.0, offset.y);
                }
                EffectorTransform::Elevate { delta_y } => {
                    transform.translation = Vec3::new(0.0, *delta_y, 0.0);
                }
            }
        }
    }
}

pub fn update_carrier_platform_momentum(
    time: Res<Time>,
    platforms: Query<
        (
            &CarrierPlatform,
            Option<&DynamicSectorMesh>,
            Option<&Transform>,
        ),
        Without<crate::player::PlayerController>,
    >,
    mut players: Query<
        (
            &mut Transform,
            &mut crate::player::PlayerController,
            Option<&crate::sector_map::CurrentSector>,
        ),
        Without<CarrierPlatform>,
    >,
) {
    let dt = time.delta_seconds();

    for (carrier, dyn_mesh, platform_trans) in platforms.iter() {
        if carrier.velocity == Vec3::ZERO {
            continue;
        }

        let platform_sec_idx = carrier
            .sector_idx
            .or_else(|| dyn_mesh.map(|m| m.sector_idx));

        for (mut player_trans, mut player_ctrl, current_sec) in players.iter_mut() {
            // 1. Sector match check
            let mut matches_sector = false;
            if let Some(p_sec) = current_sec {
                if p_sec.0 >= 0 {
                    if let Some(plat_sec) = platform_sec_idx {
                        if plat_sec == p_sec.0 as usize {
                            matches_sector = true;
                        }
                    }
                }
            }

            // Fall back to 2D bounds check if CurrentSector is not present, negative, or not matching
            if !matches_sector {
                let p_xz = Vec2::new(player_trans.translation.x, player_trans.translation.z);
                if p_xz.x >= carrier.sector_bounds_min.x
                    && p_xz.x <= carrier.sector_bounds_max.x
                    && p_xz.y >= carrier.sector_bounds_min.y
                    && p_xz.y <= carrier.sector_bounds_max.y
                {
                    matches_sector = true;
                }
            }

            if !matches_sector {
                continue;
            }

            // 2. Floor height tolerance check
            let in_height_tolerance = if let Some(p_trans) = platform_trans {
                let plat_y = p_trans.translation.y;
                let height_diff = (player_trans.translation.y - plat_y).abs();
                let feet_diff = ((player_trans.translation.y - 0.8) - plat_y).abs();
                height_diff <= 2.5 || feet_diff <= 2.0
            } else {
                true
            };

            if !in_height_tolerance {
                continue;
            }

            // 3. Apply platform horizontal & vertical movement delta and velocity transfer
            if dt > 0.0 {
                player_trans.translation += carrier.velocity * dt;
            }

            // Transfer linear velocity to player controller
            if carrier.velocity.y != 0.0 {
                player_ctrl.velocity_y = carrier.velocity.y;
            }
            if carrier.velocity.x != 0.0 || carrier.velocity.z != 0.0 {
                player_ctrl.velocity_xz += Vec2::new(carrier.velocity.x, carrier.velocity.z);
            }
        }
    }
}

pub fn update_earthquake_camera_shake(
    _time: Res<Time>,
    effectors: Query<&SectorEffectorComponent>,
    mut shake: ResMut<EarthquakeCameraShake>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut last_offset: Local<Vec3>,
) {
    let mut active_earthquake = false;
    let mut max_intensity = 0.0f32;

    for effector in effectors.iter() {
        if effector.lotag == 2 || effector.lotag == 22 || matches!(effector.kind, EffectorKind::Earthquake { .. }) {
            if let EffectorKind::Earthquake { intensity, duration, elapsed, is_triggered } = effector.kind {
                if is_triggered || effector.active {
                    active_earthquake = true;
                    let decay = if duration > 0.0 {
                        (1.0 - (elapsed / duration)).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let cur = intensity * decay;
                    if cur > max_intensity {
                        max_intensity = cur;
                    }
                }
            }
        }
    }

    if active_earthquake {
        shake.intensity = max_intensity;
        if shake.intensity > 0.0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let rx = rng.gen_range(-1.0..=1.0);
            let ry = rng.gen_range(-1.0..=1.0);
            let rz = rng.gen_range(-1.0..=1.0);
            shake.offset = Vec3::new(rx, ry, rz) * shake.intensity;
        } else {
            shake.offset = Vec3::ZERO;
        }
    } else {
        shake.intensity = 0.0;
        shake.offset = Vec3::ZERO;
    }

    // Apply offset to camera transform during earthquake events
    for mut cam_trans in camera_query.iter_mut() {
        cam_trans.translation -= *last_offset;
        cam_trans.translation += shake.offset;
    }
    *last_offset = shake.offset;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_sector_mesh_parts() {
        let parts = [
            SectorMeshPart::Floor,
            SectorMeshPart::Ceiling,
            SectorMeshPart::UpperWall,
            SectorMeshPart::LowerWall,
            SectorMeshPart::MiddleWall,
        ];
        for (i, &part) in parts.iter().enumerate() {
            let mesh = DynamicSectorMesh::new(i, part);
            assert_eq!(mesh.sector_idx, i);
            assert_eq!(mesh.part, part);
        }
    }

    #[test]
    fn test_handle_tag_activations_channel_matching() {
        let mut app = App::new();
        app.add_event::<ActivateTagEvent>();
        app.add_systems(Update, handle_tag_activations);

        // SE 15 (Sliding Door) on channel 42:
        // lotag = 15 (effector type), hitag = 42 (trigger channel)
        let effector_entity = app
            .world_mut()
            .spawn(SectorEffectorComponent {
                sector_idx: 1,
                lotag: 15,
                hitag: 42,
                kind: EffectorKind::SlidingDoor {
                    orig_pos: Vec2::ZERO,
                    open_offset: Vec2::new(3.0, 0.0),
                    progress: 0.0,
                    speed: 1.0,
                    auto_close_timer: None,
                    auto_close_delay: 3.0,
                    is_open: false,
                },
                active: false,
            })
            .id();

        // Send an event with lotag = 15 (same as effector.lotag, but NOT effector.hitag)
        app.world_mut().send_event(ActivateTagEvent { lotag: 15 });
        app.update();

        // Must NOT activate because 15 is the SE type, not the trigger channel!
        let eff = app
            .world()
            .entity(effector_entity)
            .get::<SectorEffectorComponent>()
            .unwrap();
        assert!(
            !eff.active,
            "Effector should not activate when event.lotag matches effector.lotag"
        );

        // Now send event with lotag = 42 (matching effector.hitag)
        app.world_mut().send_event(ActivateTagEvent { lotag: 42 });
        app.update();

        let eff = app
            .world()
            .entity(effector_entity)
            .get::<SectorEffectorComponent>()
            .unwrap();
        assert!(
            eff.active,
            "Effector must activate when event.lotag matches effector.hitag"
        );
    }

    #[test]
    fn test_ceiling_door_never_moves_floor() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);
        app.add_systems(Update, update_sector_effectors);

        // AutoCloseDoor in sector 5:
        // orig_ceil_z = 0, open_ceil_z = -4096 * 16 (opens upwards)
        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 5,
            lotag: 10,
            hitag: 0,
            kind: EffectorKind::AutoCloseDoor {
                orig_ceil_z: 0,
                open_ceil_z: -(4096 * 16),
                current_ceil_z: 0,
                speed: 16,
                auto_close_timer: None,
                auto_close_delay: 3.0,
                is_open: true,
            },
            active: true,
        });

        let floor_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(5, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let ceil_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(5, SectorMeshPart::Ceiling),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let upper_wall = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(5, SectorMeshPart::UpperWall),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let lower_wall = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(5, SectorMeshPart::LowerWall),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.update();

        let floor_trans = app.world().entity(floor_mesh).get::<Transform>().unwrap();
        let ceil_trans = app.world().entity(ceil_mesh).get::<Transform>().unwrap();
        let upper_trans = app.world().entity(upper_wall).get::<Transform>().unwrap();
        let lower_trans = app.world().entity(lower_wall).get::<Transform>().unwrap();

        // Floor must NOT move!
        assert_eq!(floor_trans.translation, Vec3::ZERO, "Doorway floor must NEVER move!");
        assert_eq!(lower_trans.translation, Vec3::ZERO, "Lower wall must not move for ceiling door!");

        // Ceiling and upper wall MUST move
        assert!(ceil_trans.translation.y > 0.0, "Ceiling must translate upwards when door opens");
        assert!(upper_trans.translation.y > 0.0, "Upper wall must translate upwards when door opens");
    }

    #[test]
    fn test_elevator_moves_floor_and_lower_wall_only() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);
        app.add_systems(Update, update_sector_effectors);

        // Elevator in sector 3 (rises up):
        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 3,
            lotag: 17,
            hitag: 0,
            kind: EffectorKind::Elevator {
                orig_floor_z: 10000,
                target_floor_z: 5000, // lower Z in Build is higher in world
                current_floor_z: 10000,
                orig_ceil_z: 0,
                target_ceil_z: 0,
                current_ceil_z: 0,
                speed: 16,
                is_at_top: true,
                auto_return_timer: None,
            },
            active: true,
        });

        let floor_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(3, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let ceil_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(3, SectorMeshPart::Ceiling),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let lower_wall = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(3, SectorMeshPart::LowerWall),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.update();

        let floor_trans = app.world().entity(floor_mesh).get::<Transform>().unwrap();
        let ceil_trans = app.world().entity(ceil_mesh).get::<Transform>().unwrap();
        let lower_trans = app.world().entity(lower_wall).get::<Transform>().unwrap();

        assert!(floor_trans.translation.y > 0.0, "Elevator floor must rise");
        assert!(lower_trans.translation.y > 0.0, "Elevator lower wall must rise");
        assert_eq!(ceil_trans.translation, Vec3::ZERO, "Elevator ceiling must not move");
    }

    #[test]
    fn test_crusher_ceiling_never_moves_floor() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);
        app.add_systems(Update, update_sector_effectors);

        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 7,
            lotag: 31,
            hitag: 0,
            kind: EffectorKind::CrusherSector {
                min_z: 0,
                max_z: 10000,
                current_z: 0,
                speed: 16,
                crushing_ceiling: true,
                moving_down: true,
            },
            active: true,
        });

        let floor_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(7, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let ceil_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(7, SectorMeshPart::Ceiling),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.update();

        let floor_trans = app.world().entity(floor_mesh).get::<Transform>().unwrap();
        let ceil_trans = app.world().entity(ceil_mesh).get::<Transform>().unwrap();

        assert_eq!(floor_trans.translation, Vec3::ZERO, "Crusher floor must not move for crushing ceiling");
        assert_ne!(ceil_trans.translation, Vec3::ZERO, "Crusher ceiling must move");
    }

    #[test]
    fn test_sliding_door_translates_door_meshes() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100));
        app.insert_resource(time);
        app.add_systems(Update, update_sector_effectors);

        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 8,
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
        });

        let floor_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(8, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let ceil_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(8, SectorMeshPart::Ceiling),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.update();

        let floor_trans = app.world().entity(floor_mesh).get::<Transform>().unwrap();
        let ceil_trans = app.world().entity(ceil_mesh).get::<Transform>().unwrap();

        assert!(floor_trans.translation.x > 0.0, "Sliding door floor must slide");
        assert!(ceil_trans.translation.x > 0.0, "Sliding door ceiling must slide");
        assert_eq!(floor_trans.translation.x, ceil_trans.translation.x);
    }

    #[test]
    fn test_carrier_platform_velocity_transfer_to_player() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100)); // dt = 0.1s
        app.insert_resource(time);
        app.add_systems(Update, update_carrier_platform_momentum);

        // Moving platform in sector 4 ascending at 3.0 m/s and sliding at 2.0 m/s
        let platform_vel = Vec3::new(2.0, 3.0, 0.0);
        app.world_mut().spawn((
            CarrierPlatform {
                velocity: platform_vel,
                sector_bounds_min: Vec2::new(-5.0, -5.0),
                sector_bounds_max: Vec2::new(5.0, 5.0),
                sector_idx: Some(4),
            },
            DynamicSectorMesh::new(4, SectorMeshPart::Floor),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        // Player standing on platform in sector 4
        let player_entity = app
            .world_mut()
            .spawn((
                crate::player::PlayerController::default(),
                Transform::from_translation(Vec3::new(0.0, 0.8, 0.0)),
                crate::sector_map::CurrentSector(4),
            ))
            .id();

        // Player 2 high above the platform (out of height tolerance)
        let high_player_entity = app
            .world_mut()
            .spawn((
                crate::player::PlayerController::default(),
                Transform::from_translation(Vec3::new(0.0, 30.0, 0.0)),
                crate::sector_map::CurrentSector(4),
            ))
            .id();

        app.update();

        // Verify player 1 standing on platform received velocity transfer and movement delta
        let p1_entity = app.world().entity(player_entity);
        let p1_trans = p1_entity.get::<Transform>().unwrap();
        let p1_ctrl = p1_entity.get::<crate::player::PlayerController>().unwrap();
        assert_eq!(
            p1_ctrl.velocity_y, 3.0,
            "Vertical velocity must transfer from carrier platform to player"
        );
        assert_eq!(
            p1_ctrl.velocity_xz,
            Vec2::new(2.0, 0.0),
            "Horizontal velocity must transfer from carrier platform to player"
        );
        assert!(
            (p1_trans.translation.y - 1.1).abs() < 1e-4,
            "Player Y must be displaced by platform velocity * dt"
        );
        assert!(
            (p1_trans.translation.x - 0.2).abs() < 1e-4,
            "Player X must be displaced by platform velocity * dt"
        );

        // Verify player 2 (out of tolerance) was unaffected
        let p2_entity = app.world().entity(high_player_entity);
        let p2_trans = p2_entity.get::<Transform>().unwrap();
        let p2_ctrl = p2_entity.get::<crate::player::PlayerController>().unwrap();
        assert_eq!(p2_ctrl.velocity_y, 0.0);
        assert_eq!(p2_trans.translation.y, 30.0);
    }

    #[test]
    fn test_earthquake_camera_shake_intensity_progression_and_decay() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100)); // dt = 0.1s
        app.insert_resource(time);
        app.init_resource::<EarthquakeCameraShake>();
        app.add_systems(
            Update,
            (update_sector_effectors, update_earthquake_camera_shake).chain(),
        );

        // Spawn Earthquake effector with intensity 2.0 and duration 2.0s
        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 1,
            lotag: 2,
            hitag: 0,
            kind: EffectorKind::Earthquake {
                intensity: 2.0,
                duration: 2.0,
                elapsed: 0.0,
                is_triggered: true,
            },
            active: true,
        });

        // Spawn a camera
        let camera_entity = app
            .world_mut()
            .spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 1.5, 0.0),
                ..default()
            })
            .id();

        // Tick 1: dt = 0.1s -> elapsed = 0.1s
        app.update();

        let shake = app.world().resource::<EarthquakeCameraShake>();
        let initial_intensity = shake.intensity;
        // Decay factor: 1.0 - (0.1 / 2.0) = 0.95 -> intensity = 2.0 * 0.95 = 1.9
        assert!(
            (initial_intensity - 1.9).abs() < 1e-3,
            "Earthquake intensity should be ~1.9, got {}",
            initial_intensity
        );
        assert!(
            shake.offset.length() > 0.0,
            "Camera jitter offset should be non-zero during earthquake"
        );

        let cam_trans = app.world().entity(camera_entity).get::<Transform>().unwrap();
        assert_ne!(
            cam_trans.translation,
            Vec3::new(0.0, 1.5, 0.0),
            "Camera transform should be jittered by earthquake offset"
        );

        // Tick 2: Advance time by 0.9s (total elapsed = 1.0s, exactly half duration)
        let mut time_res = app.world_mut().resource_mut::<Time>();
        time_res.advance_by(std::time::Duration::from_millis(900));
        app.update();

        let shake = app.world().resource::<EarthquakeCameraShake>();
        let mid_intensity = shake.intensity;
        // Decay factor: 1.0 - (1.0 / 2.0) = 0.5 -> intensity = 2.0 * 0.5 = 1.0
        assert!(
            (mid_intensity - 1.0).abs() < 1e-3,
            "Intensity should have progressed and decayed to 1.0, got {}",
            mid_intensity
        );
        assert!(
            mid_intensity < initial_intensity,
            "Intensity must decay over time"
        );

        // Tick 3: Advance time past duration by 1.5s (total elapsed = 2.5s > 2.0s duration)
        let mut time_res = app.world_mut().resource_mut::<Time>();
        time_res.advance_by(std::time::Duration::from_millis(1500));
        app.update();

        let shake = app.world().resource::<EarthquakeCameraShake>();
        assert_eq!(
            shake.intensity, 0.0,
            "Intensity must be 0.0 after earthquake finishes"
        );
        assert_eq!(
            shake.offset,
            Vec3::ZERO,
            "Shake offset must be ZERO after earthquake finishes"
        );

        let cam_trans = app.world().entity(camera_entity).get::<Transform>().unwrap();
        assert!(
            cam_trans.translation.distance(Vec3::new(0.0, 1.5, 0.0)) < 1e-4,
            "Camera must return to base translation after earthquake"
        );
    }

    #[test]
    fn test_elevator_floor_tagged_with_carrier_platform() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);
        app.add_systems(Update, update_sector_effectors);

        // Elevator in sector 3 (rises up):
        app.world_mut().spawn(SectorEffectorComponent {
            sector_idx: 3,
            lotag: 17,
            hitag: 0,
            kind: EffectorKind::Elevator {
                orig_floor_z: 10000,
                target_floor_z: 5000,
                current_floor_z: 10000,
                orig_ceil_z: 0,
                target_ceil_z: 0,
                current_ceil_z: 0,
                speed: 16,
                is_at_top: true,
                auto_return_timer: None,
            },
            active: true,
        });

        let floor_mesh = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(3, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.update();

        let carrier = app.world().entity(floor_mesh).get::<CarrierPlatform>();
        assert!(carrier.is_some(), "Floor mesh must be tagged with CarrierPlatform");
        let carrier = carrier.unwrap();
        assert!(
            carrier.velocity.y > 0.0,
            "Elevator floor CarrierPlatform must have upward velocity"
        );
        assert_eq!(carrier.sector_idx, Some(3));
    }

    #[test]
    fn test_dynamic_sector_shade_modulation() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);
        app.init_resource::<Assets<Mesh>>();
        app.add_systems(Update, update_sector_effectors);

        // Create a test mesh with ATTRIBUTE_COLOR
        let mut test_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::default(),
        );
        test_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        test_mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[1.0, 1.0, 1.0, 1.0]; 3],
        );
        let mesh_handle = app.world_mut().resource_mut::<Assets<Mesh>>().add(test_mesh);

        // Spawn sector mesh entity
        let _mesh_entity = app
            .world_mut()
            .spawn((
                DynamicSectorMesh::new(5, SectorMeshPart::Floor),
                Transform::from_translation(Vec3::ZERO),
                mesh_handle.clone(),
            ))
            .id();

        // Spawn a light switch effector in sector 5, initially ON (-5 shade)
        let effector_entity = app
            .world_mut()
            .spawn(SectorEffectorComponent {
                sector_idx: 5,
                lotag: 12,
                hitag: 0,
                kind: EffectorKind::LightSwitchOperator {
                    is_on: true,
                    on_shade: -5,
                    off_shade: 30,
                },
                active: true,
            })
            .id();

        app.update();

        // Check that mesh ATTRIBUTE_COLOR was updated to bright on_shade (-5)
        {
            let meshes = app.world().resource::<Assets<Mesh>>();
            let mesh = meshes.get(&mesh_handle).unwrap();
            if let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(ref colors)) =
                mesh.attribute(Mesh::ATTRIBUTE_COLOR)
            {
                assert!(colors[0][0] > 1.0, "Expected bright shade tint > 1.0, got {}", colors[0][0]);
            } else {
                panic!("Mesh missing ATTRIBUTE_COLOR");
            }
        }

        // Toggle switch to OFF
        {
            let mut entity_mut = app.world_mut().entity_mut(effector_entity);
            let mut effector = entity_mut.get_mut::<SectorEffectorComponent>().unwrap();
            if let EffectorKind::LightSwitchOperator { ref mut is_on, .. } = effector.kind {
                *is_on = false;
            }
        }

        app.update();

        // Check that mesh ATTRIBUTE_COLOR was updated to dark off_shade (30)
        {
            let meshes = app.world().resource::<Assets<Mesh>>();
            let mesh = meshes.get(&mesh_handle).unwrap();
            if let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(ref colors)) =
                mesh.attribute(Mesh::ATTRIBUTE_COLOR)
            {
                assert!(colors[0][0] < 0.1, "Expected dark shade tint < 0.1, got {}", colors[0][0]);
            } else {
                panic!("Mesh missing ATTRIBUTE_COLOR");
            }
        }
    }
}
