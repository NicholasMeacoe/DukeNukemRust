#![allow(dead_code)]

use bevy::prelude::*;
use crate::player::types::PlayerController;
use crate::game_flow::state::GamePhase;

use crate::interactivity::types::ToiletProp;

impl ToiletProp {
    pub fn interact(&mut self, player: &mut PlayerController) -> usize {
        if self.cooldown_timer <= 0.0 {
            // Urinate -> restore +10 HP up to 100
            player.health = (player.health + 10).min(100);
            self.cooldown_timer = 220.0;
            34 // DUKE_URINATE sound id
        } else {
            // Already urinated -> flush sound
            35 // FLUSH_TOILET sound id
        }
    }
}

#[derive(Component, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FountainProp;

impl FountainProp {
    pub fn drink(&mut self, player: &mut PlayerController) -> Option<usize> {
        if player.health < 100 {
            player.health = (player.health + 1).min(100);
            Some(36) // DUKE_DRINKING sound id
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityCameraMonitor {
    pub camera_tag: i16,
    pub is_viewing: bool,
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DancerProp {
    pub tip_timer: f32,
    pub total_tips: usize,
}

impl Default for DancerProp {
    fn default() -> Self {
        Self { tip_timer: 0.0, total_tips: 0 }
    }
}

impl DancerProp {
    pub fn tip(&mut self) -> usize {
        self.tip_timer = 1.0;
        self.total_tips += 1;
        if self.total_tips % 2 == 0 {
            37 // DUKE_TIP1 ("Shake it baby!")
        } else {
            38 // DUKE_TIP2 ("You wanna dance?")
        }
    }
}

#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoneyItem {
    pub velocity: Vec3,
    pub flutter_timer: f32,
    pub is_settled: bool,
}

impl Default for MoneyItem {
    fn default() -> Self {
        Self {
            velocity: Vec3::new(0.0, 1.5, 0.0),
            flutter_timer: 0.0,
            is_settled: false,
        }
    }
}

pub fn update_toilet_cooldowns(
    time: Res<Time>,
    mut toilets: Query<&mut ToiletProp>,
) {
    let dt = time.delta_seconds();
    for mut toilet in toilets.iter_mut() {
        if toilet.cooldown_timer > 0.0 {
            toilet.cooldown_timer -= dt;
            if toilet.cooldown_timer < 0.0 {
                toilet.cooldown_timer = 0.0;
            }
        }
    }
}

pub fn update_money_physics(
    time: Res<Time>,
    mut money_query: Query<(&mut Transform, &mut MoneyItem)>,
) {
    let dt = time.delta_seconds();
    for (mut trans, mut money) in money_query.iter_mut() {
        if money.is_settled {
            continue;
        }

        money.flutter_timer += dt * 5.0;
        let horizontal_flutter = Vec3::new(
            money.flutter_timer.sin() * 0.3,
            0.0,
            money.flutter_timer.cos() * 0.3,
        );

        money.velocity.y -= 4.0 * dt; // Gentle falling gravity
        trans.translation += (money.velocity + horizontal_flutter) * dt;

        // Ground collision (stops falling if hits floor)
        if trans.translation.y <= 0.1 {
            trans.translation.y = 0.1;
            money.is_settled = true;
            money.velocity = Vec3::ZERO;
        }
    }
}

pub struct ExtendedPropsPlugin;

impl Plugin for ExtendedPropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_toilet_cooldowns, update_money_physics)
                .run_if(in_state(GamePhase::Playing)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toilet_urination_and_flush_lifecycle() {
        let mut toilet = ToiletProp::default();
        let mut player = PlayerController::default();
        player.health = 80;

        // 1. First interaction: Urinate -> +10 HP, 220s cooldown, DUKE_URINATE sound id 34
        let sfx1 = toilet.interact(&mut player);
        assert_eq!(sfx1, 34);
        assert_eq!(player.health, 90);
        assert_eq!(toilet.cooldown_timer, 220.0);

        // 2. Second interaction during cooldown: Flush -> no heal, FLUSH_TOILET sound id 35
        let sfx2 = toilet.interact(&mut player);
        assert_eq!(sfx2, 35);
        assert_eq!(player.health, 90);

        // 3. Max health cap at 100
        toilet.cooldown_timer = 0.0;
        player.health = 95;
        let sfx3 = toilet.interact(&mut player);
        assert_eq!(sfx3, 34);
        assert_eq!(player.health, 100);
    }

    #[test]
    fn test_water_fountain_drinking() {
        let mut fountain = FountainProp::default();
        let mut player = PlayerController::default();
        player.health = 98;

        let sfx1 = fountain.drink(&mut player);
        assert_eq!(sfx1, Some(36));
        assert_eq!(player.health, 99);

        let sfx2 = fountain.drink(&mut player);
        assert_eq!(sfx2, Some(36));
        assert_eq!(player.health, 100);

        let sfx3 = fountain.drink(&mut player);
        assert_eq!(sfx3, None); // Full health -> no drink
        assert_eq!(player.health, 100);
    }

    #[test]
    fn test_dancer_tipping_quotes() {
        let mut dancer = DancerProp::default();
        let quote1 = dancer.tip();
        assert_eq!(quote1, 38); // First tip -> "You wanna dance?"
        assert_eq!(dancer.total_tips, 1);

        let quote2 = dancer.tip();
        assert_eq!(quote2, 37); // Second tip -> "Shake it baby!"
        assert_eq!(dancer.total_tips, 2);
    }

    #[test]
    fn test_money_flutter_physics() {
        let money = MoneyItem::default();
        assert!(!money.is_settled);
        assert_eq!(money.velocity.y, 1.5);
    }
}
