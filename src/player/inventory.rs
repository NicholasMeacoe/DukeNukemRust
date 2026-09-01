#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::*;
use crate::audio::PlaySoundEvent;

pub fn handle_inventory_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut PlayerController)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
    holoduke_query: Query<Entity, With<HoloDukeDecoy>>,
) {
    let Ok((trans, mut player)) = query.get_single_mut() else { return; };

    // 1. Steroids (Key 'U')
    if keys.just_pressed(KeyCode::KeyU) && player.inventory.steroids_amount > 0 {
        player.inventory.steroids_active = true;
        // Instant unshrink if shrunk!
        player.shrink_timer = 0.0;
        sound_events.send(PlaySoundEvent { sound_id: 723 }); // DUKE_TAKEPILLS
    }

    // 2. Medkit (Key 'M')
    if keys.just_pressed(KeyCode::KeyM) && player.inventory.medkit_amount > 0 && player.health < player.max_health {
        let needed = player.max_health - player.health;
        let use_amount = needed.min(player.inventory.medkit_amount);
        player.health += use_amount;
        player.inventory.medkit_amount -= use_amount;
        sound_events.send(PlaySoundEvent { sound_id: 722 }); // DUKE_USEMEDKIT
    }

    // 3. Nightvision (Key 'N')
    if keys.just_pressed(KeyCode::KeyN) && player.inventory.nightvision_amount > 0 {
        player.inventory.nightvision_active = !player.inventory.nightvision_active;
        sound_events.send(PlaySoundEvent { sound_id: 649 }); // NITEVISION_ONOFF
    }

    // 4. Jetpack (Key 'J')
    if keys.just_pressed(KeyCode::KeyJ) && player.inventory.jetpack_amount > 0 {
        player.inventory.jetpack_active = !player.inventory.jetpack_active;
        if player.inventory.jetpack_active {
            sound_events.send(PlaySoundEvent { sound_id: 49 }); // DUKE_JETPACK_ON
        } else {
            sound_events.send(PlaySoundEvent { sound_id: 51 }); // DUKE_JETPACK_OFF
        }
    }

    // 5. Holoduke (Key 'H')
    if keys.just_pressed(KeyCode::KeyH) && player.inventory.holoduke_amount > 0 {
        player.inventory.holoduke_active = !player.inventory.holoduke_active;
        sound_events.send(PlaySoundEvent { sound_id: 70 }); // TELEPORT
        if player.inventory.holoduke_active {
            // Spawn HoloDukeDecoy entity
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(trans.translation),
                    ..default()
                },
                HoloDukeDecoy { lifetime: 30.0 },
                crate::game_flow::LevelEntity,
            ));
        } else {
            for entity in holoduke_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn update_inventory_timers(
    time: Res<Time>,
    mut query: Query<&mut PlayerController>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
    holoduke_query: Query<Entity, With<HoloDukeDecoy>>,
) {
    let dt = time.delta_seconds();
    let Ok(mut player) = query.get_single_mut() else { return; };

    player.inventory.inventory_accumulator += dt;
    let tick_interval = 0.1; // Tick 10 times per second
    while player.inventory.inventory_accumulator >= tick_interval {
        player.inventory.inventory_accumulator -= tick_interval;

        // Steroids countdown (20 / sec = 2 per 0.1s tick)
        if player.inventory.steroids_active {
            player.inventory.steroids_amount = player.inventory.steroids_amount.saturating_sub(2);
            if player.inventory.steroids_amount == 0 {
                player.inventory.steroids_active = false;
            }
        }

        // Nightvision countdown (10 / sec = 1 per tick)
        if player.inventory.nightvision_active {
            player.inventory.nightvision_amount = player.inventory.nightvision_amount.saturating_sub(1);
            if player.inventory.nightvision_amount == 0 {
                player.inventory.nightvision_active = false;
            }
        }

        // Jetpack countdown (15 / sec = ~1.5 per tick)
        if player.inventory.jetpack_active {
            player.inventory.jetpack_amount = player.inventory.jetpack_amount.saturating_sub(1);
            if player.inventory.jetpack_amount == 0 {
                player.inventory.jetpack_active = false;
            }
        }

        // Holoduke countdown (10 / sec = 1 per tick)
        if player.inventory.holoduke_active {
            player.inventory.holoduke_amount = player.inventory.holoduke_amount.saturating_sub(1);
            if player.inventory.holoduke_amount == 0 {
                player.inventory.holoduke_active = false;
                for entity in holoduke_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // Scuba consumption while underwater
        let is_underwater = matches!(player.movement_mode, PlayerMovementMode::Swimming | PlayerMovementMode::Diving);
        if is_underwater && player.inventory.scuba_amount > 0 {
            player.inventory.scuba_amount = player.inventory.scuba_amount.saturating_sub(1);
        }
    }

    // Scuba & Underwater Air Supply / Drowning
    let is_underwater = matches!(player.movement_mode, PlayerMovementMode::Swimming | PlayerMovementMode::Diving);
    if is_underwater {
        if player.inventory.scuba_amount > 0 {
            // Scuba prevents air loss
            player.inventory.air_supply = 100.0;
        } else {
            // Air supply depletes
            player.inventory.air_supply -= dt * 10.0;
            if player.inventory.air_supply <= 0.0 {
                player.inventory.air_supply = 0.0;
                player.inventory.drowning_damage_timer += dt;
                if player.inventory.drowning_damage_timer >= 1.0 {
                    player.inventory.drowning_damage_timer = 0.0;
                    player.health = player.health.saturating_sub(10);
                    sound_events.send(PlaySoundEvent { sound_id: 39 }); // DUKE_PAIN
                }
            }
        }
    } else {
        // Recover air rapidly when outside water
        player.inventory.air_supply = (player.inventory.air_supply + dt * 50.0).min(100.0);
        player.inventory.drowning_damage_timer = 0.0;
    }

    // Status effect countdowns (Shrink & Freeze)
    if player.shrink_timer > 0.0 {
        player.shrink_timer -= dt;
    }
    if player.freeze_timer > 0.0 {
        player.freeze_timer -= dt;
    }
}

pub fn update_player_pickups(
    mut player_query: Query<(&Transform, &mut PlayerController), Without<crate::interactivity::ItemPickup>>,
    pickup_query: Query<(Entity, &Transform, &crate::interactivity::ItemPickup), Without<PlayerController>>,
    mut sbar_query: Query<&mut crate::hud::StatusbarState>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut voice_events: EventWriter<crate::audio::PlayDukeVoiceEvent>,
    mut commands: Commands,
) {
    let Ok((p_trans, mut player)) = player_query.get_single_mut() else { return; };
    let mut sbar = sbar_query.get_single_mut().ok();

    for (pickup_entity, item_trans, item) in pickup_query.iter() {
        if p_trans.translation.distance_squared(item_trans.translation) < 4.0 { // 2.0m radius
            use crate::interactivity::PickupKind::*;
            let mut collected = false;
            let mut message = "";

            match item.kind {
                SmallMedkit => {
                    if player.health < 100 {
                        player.health = (player.health + 10).min(100);
                        collected = true;
                        message = "SMALL MEDKIT (+10 HEALTH)";
                    }
                }
                LargeMedkit => {
                    if player.health < 100 {
                        player.health = (player.health + 30).min(100);
                        collected = true;
                        message = "MEDKIT PACK (+30 HEALTH)";
                    }
                }
                PortableMedkit => {
                    player.inventory.medkit_amount = (player.inventory.medkit_amount + 100).min(100);
                    collected = true;
                    message = "PORTABLE MEDKIT";
                }
                AtomicHealth => {
                    if player.health < 200 {
                        player.health = (player.health + 50).min(200);
                        collected = true;
                        message = "ATOMIC HEALTH (+50 HEALTH)";
                        voice_events.send(crate::audio::PlayDukeVoiceEvent { name: Some("GETSOM01.VOC".into()) });
                    }
                }
                ArmorVest => {
                    if player.armor < 100 {
                        player.armor = 100;
                        collected = true;
                        message = "BODY ARMOR";
                    }
                }
                PistolClip => {
                    player.weapons[WeaponType::Pistol as usize].ammo = (player.weapons[WeaponType::Pistol as usize].ammo + 12).min(200);
                    collected = true;
                    message = "PISTOL CLIP (+12)";
                }
                ShotgunBox => {
                    player.weapons[WeaponType::Shotgun as usize].ammo = (player.weapons[WeaponType::Shotgun as usize].ammo + 10).min(50);
                    player.weapons[WeaponType::Shotgun as usize].is_unlocked = true;
                    collected = true;
                    message = "SHOTGUN SHELLS (+10)";
                }
                ChaingunBox => {
                    player.weapons[WeaponType::Chaingun as usize].ammo = (player.weapons[WeaponType::Chaingun as usize].ammo + 50).min(400);
                    player.weapons[WeaponType::Chaingun as usize].is_unlocked = true;
                    collected = true;
                    message = "CHAINGUN AMMO BOX (+50)";
                }
                RpgRocket => {
                    player.weapons[WeaponType::Rpg as usize].ammo = (player.weapons[WeaponType::Rpg as usize].ammo + 5).min(50);
                    player.weapons[WeaponType::Rpg as usize].is_unlocked = true;
                    collected = true;
                    message = "RPG ROCKETS (+5)";
                }
                PipebombBox => {
                    player.weapons[WeaponType::Pipebomb as usize].ammo = (player.weapons[WeaponType::Pipebomb as usize].ammo + 5).min(50);
                    player.weapons[WeaponType::Pipebomb as usize].is_unlocked = true;
                    collected = true;
                    message = "PIPEBOMBS (+5)";
                }
                ShrinkerAmmo => {
                    player.weapons[WeaponType::Shrinker as usize].ammo = (player.weapons[WeaponType::Shrinker as usize].ammo + 5).min(50);
                    player.weapons[WeaponType::Shrinker as usize].is_unlocked = true;
                    collected = true;
                    message = "SHRINKER CRYSTALS (+5)";
                }
                DevastatorBox => {
                    player.weapons[WeaponType::Devastator as usize].ammo = (player.weapons[WeaponType::Devastator as usize].ammo + 15).min(99);
                    player.weapons[WeaponType::Devastator as usize].is_unlocked = true;
                    collected = true;
                    message = "DEVASTATOR ROCKETS (+15)";
                }
                FreezeAmmo => {
                    player.weapons[WeaponType::Freezethrower as usize].ammo = (player.weapons[WeaponType::Freezethrower as usize].ammo + 25).min(99);
                    player.weapons[WeaponType::Freezethrower as usize].is_unlocked = true;
                    collected = true;
                    message = "FREEZETHROWER AMMO (+25)";
                }
                ExpanderAmmo => {
                    player.weapons[WeaponType::Expander as usize].ammo = (player.weapons[WeaponType::Expander as usize].ammo + 20).min(99);
                    player.weapons[WeaponType::Expander as usize].is_unlocked = true;
                    collected = true;
                    message = "EXPANDER AMMO (+20)";
                }
                Steroids => {
                    player.inventory.steroids_amount = (player.inventory.steroids_amount + 400).min(400);
                    collected = true;
                    message = "STEROIDS";
                }
                ScubaTank => {
                    player.inventory.scuba_amount = (player.inventory.scuba_amount + 100).min(100);
                    collected = true;
                    message = "SCUBA GEAR";
                }
                NightvisionGoggles => {
                    player.inventory.nightvision_amount = (player.inventory.nightvision_amount + 100).min(100);
                    collected = true;
                    message = "NIGHTVISION GOGGLES";
                }
                ProtectiveBoots => {
                    player.inventory.boots_amount = (player.inventory.boots_amount + 100).min(100);
                    collected = true;
                    message = "PROTECTIVE BOOTS";
                }
                Jetpack => {
                    player.inventory.jetpack_amount = (player.inventory.jetpack_amount + 100).min(100);
                    collected = true;
                    message = "JETPACK";
                }
                Holoduke => {
                    player.inventory.holoduke_amount = (player.inventory.holoduke_amount + 100).min(100);
                    collected = true;
                    message = "HOLODUKE";
                }
                WeaponPistol => {
                    player.weapons[WeaponType::Pistol as usize].ammo = (player.weapons[WeaponType::Pistol as usize].ammo + 12).min(200);
                    collected = true;
                    message = "PISTOL";
                }
                WeaponShotgun => {
                    player.weapons[WeaponType::Shotgun as usize].is_unlocked = true;
                    player.weapons[WeaponType::Shotgun as usize].ammo = (player.weapons[WeaponType::Shotgun as usize].ammo + 10).min(50);
                    player.current_weapon = WeaponType::Shotgun;
                    collected = true;
                    message = "YOU GOT THE SHOTGUN!";
                }
                WeaponChaingun => {
                    player.weapons[WeaponType::Chaingun as usize].is_unlocked = true;
                    player.weapons[WeaponType::Chaingun as usize].ammo = (player.weapons[WeaponType::Chaingun as usize].ammo + 50).min(400);
                    player.current_weapon = WeaponType::Chaingun;
                    collected = true;
                    message = "YOU GOT THE CHAINGUN CANNON!";
                }
                WeaponRpg => {
                    player.weapons[WeaponType::Rpg as usize].is_unlocked = true;
                    player.weapons[WeaponType::Rpg as usize].ammo = (player.weapons[WeaponType::Rpg as usize].ammo + 5).min(50);
                    player.current_weapon = WeaponType::Rpg;
                    collected = true;
                    message = "YOU GOT THE RPG!";
                }
                WeaponPipebomb => {
                    player.weapons[WeaponType::Pipebomb as usize].is_unlocked = true;
                    player.weapons[WeaponType::Pipebomb as usize].ammo = (player.weapons[WeaponType::Pipebomb as usize].ammo + 5).min(50);
                    player.current_weapon = WeaponType::Pipebomb;
                    collected = true;
                    message = "YOU GOT PIPEBOMBS!";
                }
                WeaponShrinker => {
                    player.weapons[WeaponType::Shrinker as usize].is_unlocked = true;
                    player.weapons[WeaponType::Shrinker as usize].ammo = (player.weapons[WeaponType::Shrinker as usize].ammo + 5).min(50);
                    player.current_weapon = WeaponType::Shrinker;
                    collected = true;
                    message = "YOU GOT THE SHRINKER!";
                }
                WeaponDevastator => {
                    player.weapons[WeaponType::Devastator as usize].is_unlocked = true;
                    player.weapons[WeaponType::Devastator as usize].ammo = (player.weapons[WeaponType::Devastator as usize].ammo + 15).min(99);
                    player.current_weapon = WeaponType::Devastator;
                    collected = true;
                    message = "YOU GOT THE DEVASTATOR!";
                }
                WeaponTripbomb => {
                    player.weapons[WeaponType::Tripbomb as usize].is_unlocked = true;
                    player.weapons[WeaponType::Tripbomb as usize].ammo = (player.weapons[WeaponType::Tripbomb as usize].ammo + 5).min(10);
                    player.current_weapon = WeaponType::Tripbomb;
                    collected = true;
                    message = "YOU GOT LASER TRIPBOMBS!";
                }
                WeaponFreezer => {
                    player.weapons[WeaponType::Freezethrower as usize].is_unlocked = true;
                    player.weapons[WeaponType::Freezethrower as usize].ammo = (player.weapons[WeaponType::Freezethrower as usize].ammo + 25).min(99);
                    player.current_weapon = WeaponType::Freezethrower;
                    collected = true;
                    message = "YOU GOT THE FREEZETHROWER!";
                }
                WeaponExpander => {
                    player.weapons[WeaponType::Expander as usize].is_unlocked = true;
                    player.weapons[WeaponType::Expander as usize].ammo = (player.weapons[WeaponType::Expander as usize].ammo + 20).min(99);
                    player.current_weapon = WeaponType::Expander;
                    collected = true;
                    message = "YOU GOT THE EXPANDER!";
                }
            }

            if collected {
                sound_events.send(PlaySoundEvent { sound_id: 724 }); // ITEM_PICKUP
                if let Some(ref mut sb) = sbar {
                    sb.message_text = message.to_string();
                    sb.message_timer = 2.5;
                }
                commands.entity(pickup_entity).despawn_recursive();
            }
        }
    }
}
