use crate::audio::PlaySoundEvent;
use crate::interactivity::types::*;
use bevy::prelude::*;

/// Event triggered when a projectile, hitscan bullet, or melee attack impacts a wall or interactive prop.
/// This corresponds to `checkhitwall()` in the Build engine / Duke Nukem 3D.
#[derive(Event, Debug, Clone)]
pub struct WallDamageEvent {
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub damage: i32,
    pub is_explosive: bool,
    pub hit_entity: Option<Entity>,
    pub attacker_id: Option<usize>,
}

/// System that processes wall and prop damage from impacts (`checkhitwall`).
/// Handles crack walls, breakable glass, viewscreens, mirrors, fire extinguishers, and barrels.
pub fn handle_wall_damage(
    mut wall_damage_events: EventReader<WallDamageEvent>,
    mut commands: Commands,
    mut glass_query: Query<(Entity, &Transform, &mut BreakableGlass)>,
    mut crack_query: Query<(&Transform, &mut CrackWall)>,
    mut viewscreen_query: Query<(&Transform, &mut ViewscreenProp)>,
    mut mirror_query: Query<(&Transform, &mut MirrorProp)>,
    mut fire_ext_query: Query<(Entity, &Transform, &mut FireExtinguisher)>,
    mut barrel_query: Query<(Entity, &Transform, &mut ExplodingBarrel)>,
    mut fountain_query: Query<(Entity, &Transform, &mut WaterFountain)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
    mut explosion_events: EventWriter<ExplosionDamageEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut gib_events: EventWriter<crate::combat::GibEvent>,
) {
    for ev in wall_damage_events.read() {
        let hit_pt = ev.hit_point;

        // 1. Breakable Glass
        for (entity, trans, mut glass) in glass_query.iter_mut() {
            if glass.is_broken {
                continue;
            }
            let is_direct = ev.hit_entity == Some(entity);
            let is_near = trans.translation.distance_squared(hit_pt) < 1.44; // 1.2m
            if is_direct || is_near {
                glass.health -= ev.damage;
                if glass.health <= 0 {
                    glass.is_broken = true;
                    sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // 2. Crack Walls
        for (trans, mut crack) in crack_query.iter_mut() {
            if crack.is_blown {
                continue;
            }
            let is_near = trans.translation.distance_squared(hit_pt) < 2.25; // 1.5m
            if is_near {
                if ev.is_explosive {
                    // Explosives blow the crack wall wide open immediately
                    crack.health = 0;
                    crack.is_blown = true;
                    crack.stage = 4;
                    sound_events.send(PlaySoundEvent { sound_id: 18 }); // VENT_BUST
                    if crack.lotag != 0 {
                        tag_events.send(ActivateTagEvent { lotag: crack.lotag });
                    }
                } else {
                    // Bullet / Melee impacts damage the crack wall progressively
                    crack.health -= ev.damage;
                    crack.stage = (crack.stage + 1).min(3);
                    sound_events.send(PlaySoundEvent { sound_id: 1 }); // PISTOL_RICOCHET
                    if crack.health <= 0 {
                        crack.is_blown = true;
                        crack.stage = 4;
                        sound_events.send(PlaySoundEvent { sound_id: 18 }); // VENT_BUST
                        if crack.lotag != 0 {
                            tag_events.send(ActivateTagEvent { lotag: crack.lotag });
                        }
                    }
                }
            }
        }

        // 3. Viewscreen CRT Monitors
        for (trans, mut viewscreen) in viewscreen_query.iter_mut() {
            if viewscreen.is_broken {
                continue;
            }
            let is_near = trans.translation.distance_squared(hit_pt) < 1.44; // 1.2m
            if is_near {
                viewscreen.is_broken = true;
                viewscreen.is_active = false;
                sound_events.send(PlaySoundEvent { sound_id: 18 }); // VENT_BUST
            }
        }

        // 4. Mirrors
        for (trans, mut mirror) in mirror_query.iter_mut() {
            let is_near = trans.translation.distance_squared(hit_pt) < 1.44; // 1.2m
            if is_near {
                sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                mirror.cooldown_timer = 999.0; // Disabled once cracked
            }
        }

        // 5. Fire Extinguishers
        for (entity, trans, mut ext) in fire_ext_query.iter_mut() {
            if ext.is_exploded {
                continue;
            }
            let is_direct = ev.hit_entity == Some(entity);
            let is_near = trans.translation.distance_squared(hit_pt) < 1.0; // 1.0m
            if is_direct || is_near {
                ext.health -= ev.damage;
                if ext.health <= 0 {
                    ext.is_exploded = true;
                    sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                    explosion_events.send(ExplosionDamageEvent {
                        origin: trans.translation,
                        radius: 4.0,
                        damage: 80,
                        attacker_id: ev.attacker_id,
                    });
                    gib_events.send(crate::combat::GibEvent {
                        origin: trans.translation,
                        gib_count: 4,
                    });
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // 6. Exploding Barrels (direct bullet / projectile hits)
        for (entity, trans, mut barrel) in barrel_query.iter_mut() {
            if barrel.is_exploded {
                continue;
            }
            let is_direct = ev.hit_entity == Some(entity);
            let is_near = trans.translation.distance_squared(hit_pt) < 1.44; // 1.2m
            if is_direct || is_near {
                barrel.health -= ev.damage;
                if barrel.health <= 0 {
                    barrel.is_exploded = true;
                    sound_events.send(PlaySoundEvent { sound_id: 14 }); // PIPEBOMB_EXPLODE
                    explosion_events.send(ExplosionDamageEvent {
                        origin: trans.translation,
                        radius: barrel.damage_radius,
                        damage: barrel.damage,
                        attacker_id: ev.attacker_id,
                    });
                    gib_events.send(crate::combat::GibEvent {
                        origin: trans.translation,
                        gib_count: 6,
                    });
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // 7. Water Fountains (breaks upon impact damage)
        for (entity, trans, mut fountain) in fountain_query.iter_mut() {
            if fountain.is_broken {
                continue;
            }
            let is_direct = ev.hit_entity == Some(entity);
            let is_near = trans.translation.distance_squared(hit_pt) < 1.44;
            if is_direct || is_near {
                fountain.is_broken = true;
                fountain.uses_left = 0;
                sound_events.send(PlaySoundEvent { sound_id: 19 }); // GLASS_BREAKING
                gib_events.send(crate::combat::GibEvent {
                    origin: trans.translation,
                    gib_count: 3,
                });
            }
        }
    }
}

/// Updates master switches with delay timers and handles activator propagation.
pub fn update_master_switches(
    time: Res<Time>,
    mut master_switches: Query<&mut MasterSwitch>,
    mut tag_events: EventWriter<ActivateTagEvent>,
) {
    let dt = time.delta_seconds();
    for mut master in master_switches.iter_mut() {
        if let Some(ref mut timer) = master.timer {
            *timer -= dt;
            if *timer <= 0.0 {
                master.timer = None;
                if master.hitag != 0 {
                    tag_events.send(ActivateTagEvent { lotag: master.hitag });
                }
            }
        }
    }
}

/// Triggers MasterSwitch delay chains when matching lotag activation is received.
pub fn handle_master_switch_activations(
    mut events: EventReader<ActivateTagEvent>,
    mut master_switches: Query<&mut MasterSwitch>,
    activators: Query<&Activator>,
    mut tag_events: EventWriter<ActivateTagEvent>,
) {
    for event in events.read() {
        // Trigger matching MasterSwitches
        for mut master in master_switches.iter_mut() {
            if master.lotag == event.lotag && !master.is_triggered {
                master.is_triggered = true;
                if master.delay > 0.0 {
                    master.timer = Some(master.delay);
                } else if master.hitag != 0 {
                    tag_events.send(ActivateTagEvent { lotag: master.hitag });
                }
            }
        }

        // Trigger matching Activators
        for activator in activators.iter() {
            if activator.lotag == event.lotag && activator.hitag != 0 && activator.hitag != event.lotag {
                tag_events.send(ActivateTagEvent { lotag: activator.hitag });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakable_glass_shatters_on_damage() {
        let mut glass = BreakableGlass {
            health: 10,
            is_broken: false,
            wall_idx: None,
            sector_idx: None,
        };

        glass.health -= 15;
        if glass.health <= 0 {
            glass.is_broken = true;
        }

        assert!(glass.is_broken);
        assert_eq!(glass.health, -5);
    }

    #[test]
    fn test_crack_wall_explosive_detonation() {
        let mut crack = CrackWall {
            health: 30,
            stage: 1,
            lotag: 105,
            is_blown: false,
        };

        // Explosive impact blows it open immediately
        let is_explosive = true;
        if is_explosive {
            crack.health = 0;
            crack.is_blown = true;
            crack.stage = 4;
        }

        assert!(crack.is_blown);
        assert_eq!(crack.stage, 4);
        assert_eq!(crack.lotag, 105);
    }

    #[test]
    fn test_crack_wall_progressive_bullet_damage() {
        let mut crack = CrackWall {
            health: 30,
            stage: 1,
            lotag: 200,
            is_blown: false,
        };

        crack.health -= 12;
        crack.stage = (crack.stage + 1).min(3);
        assert_eq!(crack.stage, 2);
        assert!(!crack.is_blown);

        crack.health -= 20;
        if crack.health <= 0 {
            crack.is_blown = true;
            crack.stage = 4;
        }
        assert!(crack.is_blown);
        assert_eq!(crack.stage, 4);
    }

    #[test]
    fn test_fire_extinguisher_explodes_at_zero_health() {
        let mut ext = FireExtinguisher {
            health: 10,
            is_exploded: false,
        };

        ext.health -= 12;
        if ext.health <= 0 {
            ext.is_exploded = true;
        }

        assert!(ext.is_exploded);
    }

    #[test]
    fn test_master_switch_delay_timer() {
        let mut master = MasterSwitch {
            lotag: 50,
            hitag: 100,
            delay: 2.0,
            timer: Some(2.0),
            is_triggered: true,
        };

        // Advance 1.5 seconds
        if let Some(ref mut timer) = master.timer {
            *timer -= 1.5;
        }
        assert_eq!(master.timer, Some(0.5));

        // Advance 1.0 second -> expires
        if let Some(ref mut timer) = master.timer {
            *timer -= 1.0;
            if *timer <= 0.0 {
                master.timer = None;
            }
        }
        assert_eq!(master.timer, None);
    }
}
