#![allow(dead_code)]

use bevy::prelude::*;
use crate::interactivity::types::*;

pub fn handle_player_interactions(
    mut interact_events: EventReader<InteractEvent>,
    mut switches: Query<(&Transform, &mut InteractiveSwitch)>,
    mut nuke_switches: Query<(&Transform, &mut NukeExitSwitch)>,
    mut fountains: Query<(&Transform, &mut WaterFountain)>,
    mut toilets: Query<(&Transform, &mut ToiletProp)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
    mut heal_events: EventWriter<PlayerHealEvent>,
    mut level_completed_events: EventWriter<crate::game_flow::LevelCompletedEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Option<Res<crate::GameAssets>>,
) {
    for event in interact_events.read() {
        let player_pos = event.player_pos;
        let player_dir = event.player_dir.normalize_or_zero();

        // 1. Check nearby switches with facing angle check
        for (trans, mut switch) in switches.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist = to_obj.length();
            if dist < 2.8 {
                let facing = if dist > 0.1 {
                    player_dir.dot(to_obj / dist)
                } else {
                    1.0
                };

                if facing > 0.1 {
                    switch.is_on = !switch.is_on;
                    let target_tile = if switch.is_on { switch.on_tile } else { switch.off_tile };

                    // Swap visual texture on the switch material
                    if let Some(ref mat_handle) = switch.material_handle {
                        if let Some(ref game_assets) = assets {
                            if let Some(tex_handle) = game_assets.tile_textures.get(&target_tile) {
                                if let Some(mat) = materials.get_mut(mat_handle) {
                                    mat.base_color_texture = Some(tex_handle.clone());
                                }
                            }
                        }
                    }

                    if switch.lotag != 0 {
                        tag_events.send(ActivateTagEvent {
                            lotag: switch.lotag,
                        });
                    }
                }
            }
        }

        // 2. Check drinking fountains
        for (trans, mut fountain) in fountains.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist = to_obj.length();
            if dist < 2.2 && !fountain.is_broken && fountain.uses_left > 0 {
                fountain.uses_left -= 1;
                heal_events.send(PlayerHealEvent { amount: 1 });
            }
        }

        // 3. Check toilets & stalls
        for (trans, mut toilet) in toilets.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist = to_obj.length();
            if dist < 2.2 && !toilet.is_broken {
                if toilet.last_used_time <= 0.0 {
                    toilet.last_used_time = 10.0;
                    heal_events.send(PlayerHealEvent { amount: 10 });
                }
            }
        }

        // 4. Check Nuke Buttons (Level Exit)
        for (trans, mut nuke) in nuke_switches.iter_mut() {
            let to_obj = trans.translation - player_pos;
            let dist = to_obj.length();
            if dist < 2.8 && !nuke.is_activated {
                nuke.is_activated = true;
                level_completed_events.send(crate::game_flow::LevelCompletedEvent);
            }
        }
    }
}

pub fn handle_touchplates(
    player_query: Query<&Transform, With<crate::Player>>,
    mut touchplates: Query<(&Transform, &mut Touchplate)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
) {
    let Ok(player_trans) = player_query.get_single() else { return; };
    let p_pos = player_trans.translation;

    for (trans, mut touchplate) in touchplates.iter_mut() {
        let dist = trans.translation.distance(p_pos);
        if dist <= touchplate.radius {
            if !touchplate.triggered {
                touchplate.triggered = true;
                if touchplate.lotag != 0 {
                    tag_events.send(ActivateTagEvent {
                        lotag: touchplate.lotag,
                    });
                }
            }
        } else {
            touchplate.triggered = false;
        }
    }
}

pub fn handle_explosions(
    mut explosion_events: EventReader<ExplosionDamageEvent>,
    mut barrels: Query<(Entity, &Transform, &mut ExplodingBarrel)>,
    mut crack_walls: Query<(&Transform, &mut CrackWall)>,
    mut glass_windows: Query<(Entity, &Transform, &mut BreakableGlass)>,
    mut tag_events: EventWriter<ActivateTagEvent>,
    mut commands: Commands,
) {
    for exp in explosion_events.read() {
        // 1. Check exploding barrels
        for (entity, trans, mut barrel) in barrels.iter_mut() {
            if !barrel.is_exploded {
                let dist = trans.translation.distance(exp.origin);
                if dist <= exp.radius {
                    barrel.health -= exp.damage;
                    if barrel.health <= 0 {
                        barrel.is_exploded = true;
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }

        // 2. Check crack walls
        for (trans, mut crack) in crack_walls.iter_mut() {
            if !crack.is_blown {
                let dist = trans.translation.distance(exp.origin);
                if dist <= exp.radius {
                    crack.health -= exp.damage;
                    if crack.health <= 0 {
                        crack.is_blown = true;
                        crack.stage = 4;
                        if crack.lotag != 0 {
                            tag_events.send(ActivateTagEvent {
                                lotag: crack.lotag,
                            });
                        }
                    }
                }
            }
        }

        // 3. Check breakable glass
        for (entity, trans, mut glass) in glass_windows.iter_mut() {
            if !glass.is_broken {
                let dist = trans.translation.distance(exp.origin);
                if dist <= exp.radius {
                    glass.is_broken = true;
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
