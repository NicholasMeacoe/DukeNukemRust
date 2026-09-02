#![allow(dead_code)]

pub mod episodes;
pub mod progression;

pub use episodes::*;
pub use progression::*;

use bevy::prelude::*;

pub struct CampaignPlugin;

impl Plugin for CampaignPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CampaignProgression>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_44_map_database_integrity() {
        assert_eq!(ALL_CAMPAIGN_MAPS.len(), 41); // 8 in E1, 11 in E2, 11 in E3, 11 in E4 = 41 total campaign levels

        for map in ALL_CAMPAIGN_MAPS {
            assert!(map.episode >= 1 && map.episode <= 4);
            assert!(map.level >= 1 && map.level <= 11);
            assert!(!map.title.is_empty());
            assert!(map.map_filename.starts_with('E'));
            assert!(map.music_filename.ends_with(".MID"));
            assert!(map.par_time_seconds > 0.0);
        }
    }

    #[test]
    fn test_episode_1_progression_and_secret_exit() {
        let mut prog = CampaignProgression::default();
        assert_eq!(prog.current_episode, 1);
        assert_eq!(prog.current_level, 1);
        assert_eq!(prog.current_map_filename(), "E1L1.MAP");

        // L1 -> L2
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::NextLevel {
                episode: 1,
                level: 2
            }
        );
        // L2 -> L3
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::NextLevel {
                episode: 1,
                level: 3
            }
        );

        // L3 -> Secret Exit to E1L8
        assert_eq!(
            prog.advance_level(true),
            LevelAdvanceResult::SecretLevel {
                episode: 1,
                level: 8
            }
        );
        assert_eq!(prog.current_level, 8);
        assert_eq!(prog.current_map_filename(), "E1L8.MAP");

        // Completing E1L8 returns to E1L4
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::ReturnFromSecret {
                episode: 1,
                level: 4
            }
        );
        assert_eq!(prog.current_level, 4);
        assert_eq!(prog.current_map_filename(), "E1L4.MAP");

        // L4 -> L5
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::NextLevel {
                episode: 1,
                level: 5
            }
        );

        // L5 -> L6 (Boss)
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::NextLevel {
                episode: 1,
                level: 6
            }
        );
        assert_eq!(prog.current_map_filename(), "E1L6.MAP");

        // Defeating L6 Boss completes Episode 1
        assert_eq!(
            prog.advance_level(false),
            LevelAdvanceResult::EpisodeCompleted { episode: 1 }
        );
        assert!(prog.episode_completed);
    }

    #[test]
    fn test_campaign_registry_dynamic_ingestion() {
        let script = crate::scripting::compiler::CompiledScript {
            bytecode: Vec::new(),
            actor_script_ptrs: Vec::new(),
            actor_types: Vec::new(),
            symbols: std::collections::HashMap::new(),
            actions: std::collections::HashMap::new(),
            moves: std::collections::HashMap::new(),
            ais: std::collections::HashMap::new(),
            volumes: vec![
                crate::scripting::types::DynamicVolumeDef {
                    volume_id: 0,
                    title: "L.A. MELTDOWN".into(),
                },
                crate::scripting::types::DynamicVolumeDef {
                    volume_id: 1,
                    title: "LUNAR APOCALYPSE".into(),
                },
            ],
            skills: vec![
                crate::scripting::types::DynamicSkillDef {
                    skill_id: 0,
                    title: "PIECE OF CAKE".into(),
                },
                crate::scripting::types::DynamicSkillDef {
                    skill_id: 1,
                    title: "LET'S ROCK".into(),
                },
            ],
            levels: vec![crate::scripting::types::DynamicLevelDef {
                volume: 0,
                level: 0,
                filename: "E1L1.map".into(),
                par_time_str: "01:45".into(),
                three_dr_time_str: "00:53".into(),
                title: "HOLLYWOOD HOLOCAUST".into(),
            }],
            quotes: {
                let mut m = std::collections::HashMap::new();
                m.insert(113, "CLIPPING: OFF".into());
                m
            },
            sounds: Vec::new(),
        };

        let registry = CampaignRegistry::from_compiled_script(&script);
        assert_eq!(registry.get_volume_title(0), Some("L.A. MELTDOWN"));
        assert_eq!(registry.get_skill_title(1), Some("LET'S ROCK"));
        assert_eq!(registry.get_level_map_filename(0, 0), Some("E1L1.map"));
        assert_eq!(registry.get_quote(113), Some("CLIPPING: OFF"));
    }
}
