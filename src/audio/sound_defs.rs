/// Authentic Duke Nukem 3D Sound Definitions (ported from original SOUNDEFS.H).
/// Maps symbolic sound names to their numeric IDs used in GAME.CON and the Build engine.

// Weapons & Impacts
pub const KICK_HIT: i32 = 0;
pub const PISTOL_RICOCHET: i32 = 1;
pub const PISTOL_BODYHIT: i32 = 2;
pub const PISTOL_FIRE: i32 = 3;
pub const EJECT_CLIP: i32 = 4;
pub const INSERT_CLIP: i32 = 5;
pub const CHAINGUN_FIRE: i32 = 6;
pub const RPG_FIRE: i32 = 7;
pub const CAT_FIRE: i32 = 10;
pub const SHRINKER_FIRE: i32 = 11;
pub const TELEPORTER: i32 = 11;
pub const EXPANDER_FIRE: i32 = 11;
pub const PIPEBOMB_EXPLODE: i32 = 14;
pub const LASERTRIP_ARMING: i32 = 16;
pub const LASERTRIP_EXPLODE: i32 = 17;
pub const VENT_BUST: i32 = 18;
pub const GLASS_BREAKING: i32 = 19;
pub const SHORT_CIRCUIT: i32 = 20;
pub const ITEM_SPLASH: i32 = 21;
pub const SHOTGUN_FIRE: i32 = 109;
pub const SOMETHINGFROZE: i32 = 110;
pub const PIPEBOMB_BOUNCE: i32 = 111;
pub const RPG_EXPLODE: i32 = 115;
pub const SELECT_WEAPON: i32 = 118;

// Menu & Interface
pub const MENU_MOVE: i32 = 0;
pub const MENU_SELECT: i32 = 2;

// Player Status & Quotes
pub const DUKE_PAIN: i32 = 37;
pub const DUKE_GRUNT: i32 = 38;
pub const DUKE_DEAD: i32 = 41;
pub const DUKE_LAND: i32 = 42;
pub const DUKE_SCREAM: i32 = 43;
pub const END_OF_LEVEL_WARN: i32 = 83;
pub const SECRET_AREA: i32 = 88;
pub const BONUS_SPEECH1: i32 = 195;
pub const BONUS_SPEECH2: i32 = 196;
pub const DUKE_LOOKINTOMIRROR: i32 = 252;

// Enemy Voices & Attacks
pub const TROOP_ATTACK: i32 = 510;
pub const TROOP_DIE: i32 = 511;
pub const PIG_ATTACK: i32 = 537;
pub const PIG_DIE: i32 = 538;
pub const OCTA_ATTACK: i32 = 570;
pub const OCTA_DIE: i32 = 572;

// Underwater & Atmosphere
pub const DUKE_BREATHING: i32 = 23;
pub const DUKE_EXHALING: i32 = 24;
pub const DUKE_GASP: i32 = 25;
pub const DUKE_ONWATER: i32 = 40;
pub const DUKE_UNDERWATER: i32 = 360;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_defs_constants() {
        assert_eq!(PISTOL_FIRE, 3);
        assert_eq!(CHAINGUN_FIRE, 6);
        assert_eq!(RPG_FIRE, 7);
        assert_eq!(PIPEBOMB_EXPLODE, 14);
        assert_eq!(GLASS_BREAKING, 19);
        assert_eq!(DUKE_PAIN, 37);
        assert_eq!(DUKE_DEAD, 41);
        assert_eq!(DUKE_LAND, 42);
        assert_eq!(MENU_MOVE, 0);
        assert_eq!(MENU_SELECT, 2);
        assert_eq!(BONUS_SPEECH1, 195);
        assert_eq!(BONUS_SPEECH2, 196);
        assert_eq!(DUKE_BREATHING, 23);
        assert_eq!(DUKE_EXHALING, 24);
        assert_eq!(DUKE_GASP, 25);
        assert_eq!(DUKE_ONWATER, 40);
        assert_eq!(DUKE_UNDERWATER, 360);
    }
}
