use crate::audio::PlaySoundEvent;
use crate::interactivity::SafeDespawnExt;
use crate::player::types::*;
use bevy::prelude::*;

pub fn handle_inventory_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut PlayerController)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut commands: Commands,
    holoduke_query: Query<Entity, With<HoloDukeDecoy>>,
) {
    for (trans, mut player) in query.iter_mut() {
        // 1. Steroids (Key 'U')
        if keys.just_pressed(KeyCode::KeyU) && player.inventory.steroids_amount > 0 {
            player.inventory.steroids_active = true;
            // Instant unshrink if shrunk!
            player.shrink_timer = 0.0;
            sound_events.send(PlaySoundEvent { sound_id: 723 }); // DUKE_TAKEPILLS
        }

        // 2. Medkit (Key 'M')
        if keys.just_pressed(KeyCode::KeyM)
            && player.inventory.medkit_amount > 0
            && player.health < player.max_health
        {
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
                    commands.safe_despawn_recursive(entity);
                }
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
    for mut player in query.iter_mut() {

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
            player.inventory.nightvision_amount =
                player.inventory.nightvision_amount.saturating_sub(1);
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
                    commands.safe_despawn_recursive(entity);
                }
            }
        }

        // Scuba consumption while underwater
        let is_underwater = matches!(
            player.movement_mode,
            PlayerMovementMode::Swimming | PlayerMovementMode::Diving
        );
        if is_underwater && player.inventory.scuba_amount > 0 {
            player.inventory.scuba_amount = player.inventory.scuba_amount.saturating_sub(1);
        }
    }

    // Scuba & Underwater Air Supply / Drowning
    let is_underwater = matches!(
        player.movement_mode,
        PlayerMovementMode::Swimming | PlayerMovementMode::Diving
    );
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
                    if !player.god_mode {
                        player.health = (player.health - 10).max(0);
                        if player.health == 0 {
                            player.death_timer = 3.0;
                            sound_events.send(PlaySoundEvent { sound_id: 41 }); // DUKE_DEAD
                        } else {
                            sound_events.send(PlaySoundEvent { sound_id: 37 }); // DUKE_PAIN
                        }
                    }
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
}

pub fn update_player_pickups(
    mut player_query: Query<
        (&Transform, &mut PlayerController),
        Without<crate::interactivity::ItemPickup>,
    >,
    pickup_query: Query<
        (Entity, &Transform, &crate::interactivity::ItemPickup),
        Without<PlayerController>,
    >,
    keycard_query: Query<
        (Entity, &Transform, &crate::interactivity::KeycardPickup),
        Without<PlayerController>,
    >,
    mut sbar_query: Query<&mut crate::hud::StatusbarState>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut voice_events: EventWriter<crate::audio::PlayDukeVoiceEvent>,
    mut commands: Commands,
    mut tint: Option<ResMut<crate::hud::ScreenTintState>>,
    mut rng: ResMut<crate::net::DeterministicRng>,
) {
    let mut sbar = sbar_query.get_single_mut().ok();
    let mut collected_pickups: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    for (p_trans, mut player) in player_query.iter_mut() {

    for (pickup_entity, item_trans, item) in pickup_query.iter() {
        if collected_pickups.contains(&pickup_entity) {
            continue;
        }
        if p_trans.translation.distance_squared(item_trans.translation) < 4.0 {
            // 2.0m radius
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
                    if player.health < 100 && player.inventory.medkit_amount < 100 {
                        player.inventory.medkit_amount =
                            (player.inventory.medkit_amount + 100).min(100);
                        collected = true;
                        message = "PORTABLE MEDKIT";
                    }
                }
                AtomicHealth => {
                    if player.health < 200 {
                        player.health = (player.health + 50).min(200);
                        collected = true;
                        message = "ATOMIC HEALTH (+50 HEALTH)";
                        voice_events.send(crate::audio::PlayDukeVoiceEvent {
                            name: Some("GETSOM01.VOC".into()),
                        });
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
                    let w = &mut player.weapons[WeaponType::Pistol as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 12).min(w.max_ammo);
                        collected = true;
                        message = "PISTOL CLIP (+12)";
                    }
                }
                ShotgunBox => {
                    let w = &mut player.weapons[WeaponType::Shotgun as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 10).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "SHOTGUN SHELLS (+10)";
                    }
                }
                ChaingunBox => {
                    let w = &mut player.weapons[WeaponType::Chaingun as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 50).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "CHAINGUN AMMO BOX (+50)";
                    }
                }
                RpgRocket => {
                    let w = &mut player.weapons[WeaponType::Rpg as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "RPG ROCKETS (+5)";
                    }
                }
                PipebombBox => {
                    let w = &mut player.weapons[WeaponType::Pipebomb as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "PIPEBOMBS (+5)";
                    }
                }
                ShrinkerAmmo => {
                    let w = &mut player.weapons[WeaponType::Shrinker as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "SHRINKER CRYSTALS (+5)";
                    }
                }
                DevastatorBox => {
                    let w = &mut player.weapons[WeaponType::Devastator as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 15).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "DEVASTATOR ROCKETS (+15)";
                    }
                }
                FreezeAmmo => {
                    let w = &mut player.weapons[WeaponType::Freezethrower as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 25).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "FREEZETHROWER AMMO (+25)";
                    }
                }
                ExpanderAmmo => {
                    let w = &mut player.weapons[WeaponType::Expander as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 20).min(w.max_ammo);
                        w.is_unlocked = true;
                        collected = true;
                        message = "EXPANDER AMMO (+20)";
                    }
                }
                Steroids => {
                    if player.inventory.steroids_amount < 400 {
                        player.inventory.steroids_amount =
                            (player.inventory.steroids_amount + 400).min(400);
                        collected = true;
                        message = "STEROIDS";
                    }
                }
                ScubaTank => {
                    if player.inventory.scuba_amount < 100 {
                        player.inventory.scuba_amount =
                            (player.inventory.scuba_amount + 100).min(100);
                        collected = true;
                        message = "SCUBA GEAR";
                    }
                }
                NightvisionGoggles => {
                    if player.inventory.nightvision_amount < 100 {
                        player.inventory.nightvision_amount =
                            (player.inventory.nightvision_amount + 100).min(100);
                        collected = true;
                        message = "NIGHTVISION GOGGLES";
                    }
                }
                ProtectiveBoots => {
                    if player.inventory.boots_amount < 100 {
                        player.inventory.boots_amount =
                            (player.inventory.boots_amount + 100).min(100);
                        collected = true;
                        message = "PROTECTIVE BOOTS";
                    }
                }
                Jetpack => {
                    if player.inventory.jetpack_amount < 100 {
                        player.inventory.jetpack_amount =
                            (player.inventory.jetpack_amount + 100).min(100);
                        collected = true;
                        message = "JETPACK";
                    }
                }
                Holoduke => {
                    if player.inventory.holoduke_amount < 100 {
                        player.inventory.holoduke_amount =
                            (player.inventory.holoduke_amount + 100).min(100);
                        collected = true;
                        message = "HOLODUKE";
                    }
                }
                WeaponPistol => {
                    let w = &mut player.weapons[WeaponType::Pistol as usize];
                    if w.ammo < w.max_ammo {
                        w.ammo = (w.ammo + 12).min(w.max_ammo);
                        collected = true;
                        message = "PISTOL";
                    }
                }
                WeaponShotgun => {
                    let w = &mut player.weapons[WeaponType::Shotgun as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 10).min(w.max_ammo);
                        player.current_weapon = WeaponType::Shotgun;
                        collected = true;
                        message = "YOU GOT THE SHOTGUN!";
                    }
                }
                WeaponChaingun => {
                    let w = &mut player.weapons[WeaponType::Chaingun as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 50).min(w.max_ammo);
                        player.current_weapon = WeaponType::Chaingun;
                        collected = true;
                        message = "YOU GOT THE CHAINGUN CANNON!";
                    }
                }
                WeaponRpg => {
                    let w = &mut player.weapons[WeaponType::Rpg as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        player.current_weapon = WeaponType::Rpg;
                        collected = true;
                        message = "YOU GOT THE RPG!";
                    }
                }
                WeaponPipebomb => {
                    let w = &mut player.weapons[WeaponType::Pipebomb as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        player.current_weapon = WeaponType::Pipebomb;
                        collected = true;
                        message = "YOU GOT PIPEBOMBS!";
                    }
                }
                WeaponShrinker => {
                    let w = &mut player.weapons[WeaponType::Shrinker as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        player.current_weapon = WeaponType::Shrinker;
                        collected = true;
                        message = "YOU GOT THE SHRINKER!";
                    }
                }
                WeaponDevastator => {
                    let w = &mut player.weapons[WeaponType::Devastator as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 15).min(w.max_ammo);
                        player.current_weapon = WeaponType::Devastator;
                        collected = true;
                        message = "YOU GOT THE DEVASTATOR!";
                    }
                }
                WeaponTripbomb => {
                    let w = &mut player.weapons[WeaponType::Tripbomb as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 5).min(w.max_ammo);
                        player.current_weapon = WeaponType::Tripbomb;
                        collected = true;
                        message = "YOU GOT LASER TRIPBOMBS!";
                    }
                }
                WeaponFreezer => {
                    let w = &mut player.weapons[WeaponType::Freezethrower as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 25).min(w.max_ammo);
                        player.current_weapon = WeaponType::Freezethrower;
                        collected = true;
                        message = "YOU GOT THE FREEZETHROWER!";
                    }
                }
                WeaponExpander => {
                    let w = &mut player.weapons[WeaponType::Expander as usize];
                    if !w.is_unlocked || w.ammo < w.max_ammo {
                        w.is_unlocked = true;
                        w.ammo = (w.ammo + 20).min(w.max_ammo);
                        player.current_weapon = WeaponType::Expander;
                        collected = true;
                        message = "WEAPON: EXPANDER";
                    }
                }
            }

            if collected {
                sound_events.send(PlaySoundEvent { sound_id: 39 }); // GET_ITEM
                if let Some(ref mut sb) = sbar {
                    sb.message_text = message.to_string();
                    sb.message_timer = 2.5;
                }
                if let Some(ref mut t) = tint {
                    t.target_color = Color::srgba(0.8, 0.8, 0.2, 0.5);
                }
                if rng.next_f32() < 0.15 {
                    voice_events.send(crate::audio::PlayDukeVoiceEvent {
                        name: Some("LOOK01".into()),
                    });
                }
                collected_pickups.insert(pickup_entity);
                commands.safe_despawn_recursive(pickup_entity);
            }
        }
    }

    // Auto-collect keycards when walking over them
    for (k_entity, k_trans, k_pickup) in keycard_query.iter() {
        if collected_pickups.contains(&k_entity) {
            continue;
        }
        if p_trans.translation.distance_squared(k_trans.translation) < 4.0 {
            let key_name = match k_pickup.key_type {
                1 => {
                    player.has_blue_key = true;
                    "BLUE ACCESS CARD"
                }
                2 => {
                    player.has_red_key = true;
                    "RED ACCESS CARD"
                }
                3 => {
                    player.has_yellow_key = true;
                    "YELLOW ACCESS CARD"
                }
                _ => "ACCESS CARD",
            };
            sound_events.send(PlaySoundEvent { sound_id: 65 }); // KEYCARD_GET
            if let Some(ref mut sb) = sbar {
                sb.message_text = key_name.to_string();
                sb.message_timer = 2.5;
            }
            if let Some(ref mut t) = tint {
                t.target_color = Color::srgba(0.2, 0.4, 0.9, 0.4);
            }
            collected_pickups.insert(k_entity);
            commands.safe_despawn_recursive(k_entity);
        }
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_pickup_test_app() -> App {
        let mut app = App::new();
        app.add_event::<PlaySoundEvent>();
        app.add_event::<crate::audio::PlayDukeVoiceEvent>();
        app.init_resource::<crate::net::DeterministicRng>();
        app.add_systems(Update, update_player_pickups);
        app
    }

    #[test]
    fn test_health_pickups_not_consumed_at_max_capacity() {
        let mut app = create_pickup_test_app();

        // Spawn player with full health (100)
        let mut player = PlayerController::default();
        player.health = 100;
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            player,
        ));

        // Spawn SmallMedkit (Cola)
        let cola = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::SmallMedkit,
                respawn_timer: None,
            },
        )).id();

        // Spawn LargeMedkit (Sixpak)
        let sixpak = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::LargeMedkit,
                respawn_timer: None,
            },
        )).id();

        // Spawn PortableMedkit (FirstAid)
        let firstaid = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::PortableMedkit,
                respawn_timer: None,
            },
        )).id();

        app.update();

        // None of these should be consumed when health >= 100
        assert!(app.world().get_entity(cola).is_some(), "Cola must not be picked up at 100 HP");
        assert!(app.world().get_entity(sixpak).is_some(), "Sixpak must not be picked up at 100 HP");
        assert!(app.world().get_entity(firstaid).is_some(), "FirstAid must not be picked up at 100 HP");
    }

    #[test]
    fn test_atomic_health_not_consumed_at_200_hp() {
        let mut app = create_pickup_test_app();

        let mut player = PlayerController::default();
        player.health = 200;
        let p_entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            player,
        )).id();

        let atomic = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::AtomicHealth,
                respawn_timer: None,
            },
        )).id();

        app.update();

        assert!(app.world().get_entity(atomic).is_some(), "Atomic health must not be picked up at 200 HP");

        // Now lower health to 150 and verify it is consumed
        app.world_mut().get_mut::<PlayerController>(p_entity).unwrap().health = 150;
        app.update();

        assert!(app.world().get_entity(atomic).is_none(), "Atomic health must be picked up when below 200 HP");
        let new_hp = app.world().get_entity(p_entity).unwrap().get::<PlayerController>().unwrap().health;
        assert_eq!(new_hp, 200);
    }

    #[test]
    fn test_armor_vest_not_consumed_at_max_armor() {
        let mut app = create_pickup_test_app();

        let mut player = PlayerController::default();
        player.armor = 100;
        let p_entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            player,
        )).id();

        let armor = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::ArmorVest,
                respawn_timer: None,
            },
        )).id();

        app.update();

        assert!(app.world().get_entity(armor).is_some(), "Armor vest must not be picked up at 100 armor");

        // Damage armor and verify pickup
        app.world_mut().get_mut::<PlayerController>(p_entity).unwrap().armor = 50;
        app.update();

        assert!(app.world().get_entity(armor).is_none(), "Armor vest must be picked up when armor < 100");
        let new_armor = app.world().get_entity(p_entity).unwrap().get::<PlayerController>().unwrap().armor;
        assert_eq!(new_armor, 100);
    }

    #[test]
    fn test_ammo_pickups_not_consumed_at_max_ammo() {
        let mut app = create_pickup_test_app();

        let mut player = PlayerController::default();
        // Max out pistol and shotgun ammo
        let pistol_max = player.weapons[WeaponType::Pistol as usize].max_ammo;
        let shotgun_max = player.weapons[WeaponType::Shotgun as usize].max_ammo;
        player.weapons[WeaponType::Pistol as usize].ammo = pistol_max;
        player.weapons[WeaponType::Shotgun as usize].ammo = shotgun_max;

        let p_entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            player,
        )).id();

        let pistol_clip = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::PistolClip,
                respawn_timer: None,
            },
        )).id();

        let shotgun_box = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::interactivity::ItemPickup {
                kind: crate::interactivity::PickupKind::ShotgunBox,
                respawn_timer: None,
            },
        )).id();

        app.update();

        assert!(app.world().get_entity(pistol_clip).is_some(), "Pistol clip must not be picked up at max ammo");
        assert!(app.world().get_entity(shotgun_box).is_some(), "Shotgun box must not be picked up at max ammo");

        // Spend some ammo and verify they are picked up
        {
            let mut p = app.world_mut().get_mut::<PlayerController>(p_entity).unwrap();
            p.weapons[WeaponType::Pistol as usize].ammo -= 20;
            p.weapons[WeaponType::Shotgun as usize].ammo -= 10;
        }

        app.update();

        assert!(app.world().get_entity(pistol_clip).is_none(), "Pistol clip should be picked up when below max ammo");
        assert!(app.world().get_entity(shotgun_box).is_none(), "Shotgun box should be picked up when below max ammo");
    }
}
