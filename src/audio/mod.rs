#![allow(dead_code)]

pub mod midi;
pub mod rts;
pub mod synth_stream;
pub mod voc;

pub use midi::*;
pub use rts::*;
pub use synth_stream::*;
pub use voc::*;

use crate::grp::Grp;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct DukeAudioPlugin;

impl Plugin for DukeAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MidiAudioStream>()
            .init_resource::<DukeAudioAssets>()
            .init_resource::<DynamicMusicState>()
            .init_resource::<DukeVoiceQueue>()
            .add_event::<PlaySoundEvent>()
            .add_event::<PlaySpatialSoundEvent>()
            .add_event::<PlayNamedSoundEvent>()
            .add_event::<PlayDukeVoiceEvent>()
            .add_event::<PlayMusicTrackEvent>()
            .add_systems(Startup, setup_duke_audio)
            .add_systems(
                Update,
                (
                    handle_play_sound_events,
                    handle_play_spatial_sound_events,
                    handle_play_named_sound_events,
                    handle_play_duke_voice_events,
                    handle_play_music_track_events,
                    update_dynamic_audio_state,
                    sync_music_volume_system,
                ),
            );
    }
}

pub fn update_dynamic_audio_state(
    time: Res<Time>,
    mut music_state: ResMut<DynamicMusicState>,
    mut voice_queue: ResMut<DukeVoiceQueue>,
) {
    let dt = time.delta_seconds();
    music_state.tick(dt);
    voice_queue.tick(dt);
}

#[derive(Event, Debug, Clone)]
pub struct PlaySoundEvent {
    pub sound_id: i32,
}

#[derive(Event, Debug, Clone)]
pub struct PlaySpatialSoundEvent {
    pub sound_id: i32,
    pub volume: f32,
    pub position: Vec3,
}

#[derive(Event, Debug, Clone)]
pub struct PlayNamedSoundEvent {
    pub name: String,
    pub volume: f32,
    pub position: Option<Vec3>,
}

#[derive(Event, Debug, Clone)]
pub struct PlayDukeVoiceEvent {
    pub name: Option<String>,
}

#[derive(Event, Debug, Clone)]
pub struct PlayMusicTrackEvent {
    pub track: LevelMidiTrack,
}

#[derive(Component)]
pub struct MusicTrackEmitter;

#[derive(Resource, Debug, Clone)]
pub struct DynamicMusicState {
    pub current_state: MusicState,
    pub current_track: String,
    pub base_volume: f32,
    pub ducking_timer: f32,
    pub is_underwater: bool,
    pub current_volume: f32,
}

impl Default for DynamicMusicState {
    fn default() -> Self {
        Self {
            current_state: MusicState::AmbientExploration,
            current_track: "STALKER.MID".to_string(),
            base_volume: 0.8,
            ducking_timer: 0.0,
            is_underwater: false,
            current_volume: 0.8,
        }
    }
}

impl DynamicMusicState {
    pub fn get_target_volume(&self) -> f32 {
        let mut vol = self.base_volume;
        if self.ducking_timer > 0.0 {
            vol *= 0.6; // Duck by 40% during voice lines
        }
        if self.is_underwater {
            vol *= 0.7; // Muffle underwater
        }
        vol
    }

    pub fn tick(&mut self, dt: f32) {
        if self.ducking_timer > 0.0 {
            self.ducking_timer = (self.ducking_timer - dt).max(0.0);
        }
    }
}

pub fn sync_music_volume_system(
    time: Res<Time>,
    mut music_state: ResMut<DynamicMusicState>,
    sink_query: Query<&bevy::audio::AudioSink, With<MusicTrackEmitter>>,
) {
    let target = music_state.get_target_volume();
    // Smooth lerp to prevent popping
    music_state.current_volume = music_state.current_volume
        + (target - music_state.current_volume) * (time.delta_seconds() * 5.0).min(1.0);

    for sink in &sink_query {
        sink.set_volume(music_state.current_volume);
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct DukeVoiceQueue {
    pub active_priority: u8,
    pub cooldown_timer: f32,
}

impl DukeVoiceQueue {
    pub fn should_play(&mut self, priority: u8) -> bool {
        // FIXED: Used `>` instead of `>=` to prevent overlapping equal-priority lines
        if self.cooldown_timer <= 0.0 || priority > self.active_priority {
            self.active_priority = priority;
            self.cooldown_timer = 2.0;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
            if self.cooldown_timer == 0.0 {
                self.active_priority = 0;
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct DukeAudioAssets {
    pub sounds_by_name: HashMap<String, Handle<AudioSource>>,
    pub sounds_by_id: HashMap<i32, Handle<AudioSource>>,
    pub sound_id_to_file: HashMap<i32, String>,
    pub music_tracks: HashMap<LevelMidiTrack, Handle<MidiAudioStream>>,
    pub duke_quotes: Vec<Handle<AudioSource>>,
    pub all_sounds: Vec<Handle<AudioSource>>,
}

impl DukeAudioAssets {
    pub fn get_sound_by_id(&self, id: i32) -> Option<Handle<AudioSource>> {
        if let Some(handle) = self.sounds_by_id.get(&id) {
            return Some(handle.clone());
        }
        if let Some(filename) = self.sound_id_to_file.get(&id) {
            return self.get_sound_by_name(filename);
        }
        None
    }

    pub fn get_sound_by_name(&self, name: &str) -> Option<Handle<AudioSource>> {
        let upper = name.to_uppercase();
        if let Some(handle) = self.sounds_by_name.get(&upper) {
            return Some(handle.clone());
        }
        let stripped = upper.trim_end_matches(".VOC").trim_end_matches(".WAV");
        if let Some(handle) = self.sounds_by_name.get(stripped) {
            return Some(handle.clone());
        }
        // Prefix search fallback
        for (key, handle) in &self.sounds_by_name {
            if key.starts_with(stripped) || stripped.starts_with(key) {
                return Some(handle.clone());
            }
        }
        None
    }

    pub fn play_sound(&self, commands: &mut Commands, sound_id: i32) {
        if let Some(handle) = self.get_sound_by_id(sound_id) {
            commands.spawn(AudioBundle {
                source: handle,
                ..default()
            });
        }
    }

    pub fn play_named(&self, commands: &mut Commands, name: &str) {
        if let Some(handle) = self.get_sound_by_name(name) {
            commands.spawn(AudioBundle {
                source: handle,
                ..default()
            });
        }
    }

    pub fn play_duke_quote(&self, commands: &mut Commands) {
        if !self.duke_quotes.is_empty() {
            let idx = rand::random::<usize>() % self.duke_quotes.len();
            if let Some(handle) = self.duke_quotes.get(idx) {
                commands.spawn(AudioBundle {
                    source: handle.clone(),
                    ..default()
                });
            }
        }
    }
}

fn find_grp_path() -> String {
    let candidates = [
        "dukenukem3d/duke3d.grp",
        "duke3d.grp",
        "../dukenukem3d/duke3d.grp",
        "../../dukenukem3d/duke3d.grp",
        "C:/Source/DukeNukemRust/dukenukem3d/duke3d.grp",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    "dukenukem3d/duke3d.grp".to_string()
}

pub fn setup_duke_audio(
    mut commands: Commands,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut midi_sources: ResMut<Assets<MidiAudioStream>>,
    mut audio_assets: ResMut<DukeAudioAssets>,
) {
    let grp_path = find_grp_path();
    println!("DukeAudioPlugin: Initializing audio from {}", grp_path);

    let Ok(grp) = Grp::open(&grp_path) else {
        println!(
            "DukeAudioPlugin: Warning - could not open GRP at {}",
            grp_path
        );
        return;
    };

    // 1. Build standard sound ID -> Filename mappings
    let mut id_map = build_default_sound_id_map();
    let mut name_to_id = HashMap::new();

    // 2. Parse DEFS.CON and USER.CON if available in GRP to dynamically augment sound mappings
    if let Ok(defs_data) = grp.read_file("DEFS.CON") {
        if let Ok(defs_str) = String::from_utf8(defs_data) {
            parse_defs_con_sounds(&defs_str, &mut name_to_id);
        }
    }
    if let Ok(user_data) = grp.read_file("USER.CON") {
        if let Ok(user_str) = String::from_utf8(user_data) {
            parse_user_con_sounds(&user_str, &name_to_id, &mut id_map);
        }
    }

    audio_assets.sound_id_to_file = id_map.clone();

    // 3. Load all .VOC audio files from GRP
    let mut voc_count = 0;
    for entry in &grp.entries {
        let upper_name = entry.name.to_uppercase();
        if upper_name.ends_with(".VOC") {
            if let Ok(voc_data) = grp.read_file(&entry.name) {
                if let Ok(voc_sound) = VocSound::parse(&entry.name, &voc_data) {
                    let wav_bytes = voc_sound.to_wav_bytes();
                    let audio_source = AudioSource {
                        bytes: wav_bytes.into(),
                    };
                    let handle = audio_sources.add(audio_source);

                    let stem = upper_name.trim_end_matches(".VOC").to_string();
                    audio_assets
                        .sounds_by_name
                        .insert(upper_name.clone(), handle.clone());
                    audio_assets
                        .sounds_by_name
                        .insert(stem.clone(), handle.clone());
                    audio_assets.all_sounds.push(handle.clone());
                    voc_count += 1;

                    // Collect iconic Duke quotes for speech triggers
                    if stem.starts_with("MAKEDAY")
                        || stem.starts_with("YIPPIE")
                        || stem.starts_with("BEBACK")
                        || stem.starts_with("COOL01")
                        || stem.starts_with("GROOVY")
                        || stem.starts_with("GETSOM")
                        || stem.starts_with("HAIL")
                        || stem.starts_with("LANIDUK")
                        || stem.starts_with("INDPNC")
                        || stem.starts_with("EAT08")
                        || stem.starts_with("POSTAL")
                        || stem.starts_with("SMACK")
                        || stem.starts_with("WANSOM")
                    {
                        audio_assets.duke_quotes.push(handle.clone());
                    }
                }
            }
        }
    }

    // 4. Map Sound IDs to loaded handles
    let id_mappings = audio_assets.sound_id_to_file.clone();
    for (id, filename) in id_mappings {
        if let Some(handle) = audio_assets.get_sound_by_name(&filename) {
            audio_assets.sounds_by_id.insert(id, handle);
        }
    }

    // 5. Setup streaming MIDI music via SoundFont
    let soundfont_data = std::fs::read("assets/TimGM6mb.sf2").unwrap_or_default();
    let sf2_soundfont = if !soundfont_data.is_empty() {
        let mut reader = std::io::Cursor::new(soundfont_data);
        if let Ok(sf) = rustysynth::SoundFont::new(&mut reader) {
            Some(std::sync::Arc::new(sf))
        } else {
            None
        }
    } else {
        println!("DukeAudioPlugin: Warning - assets/TimGM6mb.sf2 not found! Music will not play.");
        None
    };

    if let Some(soundfont) = sf2_soundfont {
        let tracks_to_load = [
            (LevelMidiTrack::E1L1Stalker, "STALKER.MID"),
            (LevelMidiTrack::TitleGrabbag, "GRABBAG.MID"),
        ];

        for (track_enum, midi_filename) in &tracks_to_load {
            if let Ok(midi_data) = grp.read_file(midi_filename) {
                let stream = MidiAudioStream {
                    midi_data: std::sync::Arc::new(midi_data),
                    sf2_soundfont: soundfont.clone(),
                };
                let music_handle = midi_sources.add(stream);
                audio_assets.music_tracks.insert(*track_enum, music_handle);
                println!(
                    "DukeAudioPlugin: Loaded background music: {}",
                    midi_filename
                );
            }
        }
    }

    // 6. Start playing the iconic E1L1 soundtrack (STALKER.MID) in background loop
    if let Some(e1l1_music) = audio_assets.music_tracks.get(&LevelMidiTrack::E1L1Stalker) {
        commands.spawn((
            bevy::audio::AudioSourceBundle {
                source: e1l1_music.clone(),
                settings: PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::new(0.6)),
            },
            MusicTrackEmitter,
        ));
        println!("DukeAudioPlugin: Background music playing (STALKER.MID - Episode 1 Level 1)");
    }

    println!(
        "DukeAudioPlugin: Successfully loaded {} VOC sound effects ({} sound IDs mapped, {} voice quotes ready, {} music tracks)",
        voc_count,
        audio_assets.sounds_by_id.len(),
        audio_assets.duke_quotes.len(),
        audio_assets.music_tracks.len()
    );
}

pub fn handle_play_sound_events(
    mut events: EventReader<PlaySoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut commands: Commands,
) {
    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_id(ev.sound_id) {
            commands.spawn(AudioBundle {
                source: handle,
                ..default()
            });
        }
    }
}

pub fn handle_play_spatial_sound_events(
    mut events: EventReader<PlaySpatialSoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut commands: Commands,
) {
    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_id(ev.sound_id) {
            let settings = PlaybackSettings::default()
                .with_volume(bevy::audio::Volume::new(ev.volume))
                .with_spatial(true);
            commands.spawn((
                AudioBundle {
                    source: handle,
                    settings,
                },
                TransformBundle::from_transform(Transform::from_translation(ev.position)),
            ));
        }
    }
}

pub fn handle_play_named_sound_events(
    mut events: EventReader<PlayNamedSoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut commands: Commands,
) {
    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_name(&ev.name) {
            let mut settings =
                PlaybackSettings::default().with_volume(bevy::audio::Volume::new(ev.volume));

            if let Some(pos) = ev.position {
                settings = settings.with_spatial(true);
                commands.spawn((
                    AudioBundle {
                        source: handle,
                        settings,
                    },
                    TransformBundle::from_transform(Transform::from_translation(pos)),
                ));
            } else {
                commands.spawn(AudioBundle {
                    source: handle,
                    settings,
                });
            }
        }
    }
}

pub fn handle_play_music_track_events(
    mut events: EventReader<PlayMusicTrackEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut music_state: ResMut<DynamicMusicState>,
    old_music: Query<Entity, With<MusicTrackEmitter>>,
    mut commands: Commands,
) {
    for ev in events.read() {
        if let Some(music_handle) = audio_assets.music_tracks.get(&ev.track) {
            // Despawn old tracks
            for entity in &old_music {
                commands.entity(entity).despawn_recursive();
            }

            music_state.current_track = ev.track.filename().to_string();

            // Spawn new track
            commands.spawn((
                bevy::audio::AudioSourceBundle {
                    source: music_handle.clone(),
                    settings: PlaybackSettings::LOOP
                        .with_volume(bevy::audio::Volume::new(music_state.current_volume)),
                },
                MusicTrackEmitter,
            ));
        }
    }
}

pub fn handle_play_duke_voice_events(
    mut events: EventReader<PlayDukeVoiceEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut voice_queue: Option<ResMut<DukeVoiceQueue>>,
    mut music_state: Option<ResMut<DynamicMusicState>>,
    mut commands: Commands,
) {
    for ev in events.read() {
        let priority = if let Some(ref n) = ev.name {
            if n.contains("REST_IN_PIECES") || n.contains("EAT_SHIT") || n.contains("HAIL") {
                10
            } else if n.contains("LOOKING_GOOD") {
                8
            } else {
                5
            }
        } else {
            5
        };

        let can_play = if let Some(ref mut queue) = voice_queue {
            queue.should_play(priority)
        } else {
            true
        };

        if can_play {
            if let Some(ref mut music) = music_state {
                music.ducking_timer = 2.5; // Duck background music for 2.5s
            }
            if let Some(ref name) = ev.name {
                audio_assets.play_named(&mut commands, name);
            } else {
                audio_assets.play_duke_quote(&mut commands);
            }
        }
    }
}

fn parse_defs_con_sounds(content: &str, name_to_id: &mut HashMap<String, i32>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("define") {
            let sym = parts[1].to_uppercase();
            if let Ok(id) = parts[2].parse::<i32>() {
                name_to_id.insert(sym, id);
            }
        }
    }
}

fn parse_user_con_sounds(
    content: &str,
    name_to_id: &HashMap<String, i32>,
    id_map: &mut HashMap<i32, String>,
) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("definesound") {
            let sym = parts[1].to_uppercase();
            let file = parts[2].to_uppercase();
            if let Some(&id) = name_to_id.get(&sym) {
                id_map.insert(id, file);
            }
        }
    }
}

/// Fallback dictionary of Duke Nukem 3D sound IDs to original VOC filenames
pub fn build_default_sound_id_map() -> HashMap<i32, String> {
    let mut map = HashMap::new();
    map.insert(0, "KICKHIT.VOC".into()); // KICK_HIT
    map.insert(1, "RICOCHET.VOC".into()); // PISTOL_RICOCHET
    map.insert(2, "BULITHIT.VOC".into()); // PISTOL_BODYHIT
    map.insert(3, "PISTOL.VOC".into()); // PISTOL_FIRE
    map.insert(4, "CLIPOUT.VOC".into()); // EJECT_CLIP
    map.insert(5, "CLIPIN.VOC".into()); // INSERT_CLIP
    map.insert(6, "CHAINGUN.VOC".into()); // CHAINGUN_FIRE
    map.insert(7, "RPGFIRE.VOC".into()); // RPG_SHOOT
    map.insert(8, "POOLBALL.VOC".into()); // POOLBALLHIT
    map.insert(9, "BOMBEXPL.VOC".into()); // RPG_EXPLODE
    map.insert(10, "CATFIRE.VOC".into()); // CAT_FIRE (Devastator)
    map.insert(11, "SHRINKER.VOC".into()); // SHRINKER_FIRE
    map.insert(12, "SHRINK.VOC".into()); // ACTOR_SHRINKING
    map.insert(13, "PBOMBBNC.VOC".into()); // PIPEBOMB_BOUNCE
    map.insert(14, "BOMBEXPL.VOC".into()); // PIPEBOMB_EXPLODE
    map.insert(15, "LSRBMBPT.VOC".into()); // LASERTRIP_ONWALL
    map.insert(16, "LSRBMBWN.VOC".into()); // LASERTRIP_ARMING
    map.insert(17, "BOMBEXPL.VOC".into()); // LASERTRIP_EXPLODE
    map.insert(18, "VENTBUST.VOC".into()); // VENT_BUST
    map.insert(19, "GLASS.VOC".into()); // GLASS_BREAKING
    map.insert(20, "GLASHEVY.VOC".into()); // GLASS_HEAVYBREAK
    map.insert(21, "SHORTED.VOC".into()); // SHORT_CIRCUIT
    map.insert(22, "SPLASH.VOC".into()); // ITEM_SPLASH
    map.insert(23, "HLMINHAL.VOC".into()); // DUKE_BREATHING
    map.insert(24, "HLMEXHAL.VOC".into()); // DUKE_EXHALING
    map.insert(25, "GASP.VOC".into()); // DUKE_GASP
    map.insert(28, "PISSING.VOC".into()); // DUKE_URINATE
    map.insert(36, "DRINK18.VOC".into()); // DUKE_DRINKING
    map.insert(37, "DAMN03.VOC".into()); // DUKE_KILLED1 (Pain)
    map.insert(38, "EXERT.VOC".into()); // DUKE_GRUNT (Jump)
    map.insert(39, "HARTBEAT.VOC".into()); // DUKE_HARTBEAT
    map.insert(40, "WETFEET.VOC".into()); // DUKE_ONWATER
    map.insert(41, "DMDEATH.VOC".into()); // DUKE_DEAD
    map.insert(42, "LAND02.VOC".into()); // DUKE_LAND
    map.insert(43, "DUCTWLK.VOC".into()); // DUKE_WALKINDUCTS
    map.insert(44, "COOL01.VOC".into()); // DUKE_GLAD
    map.insert(45, "YES.VOC".into()); // DUKE_YES
    map.insert(49, "JETPAKON.VOC".into()); // DUKE_JETPACK_ON
    map.insert(50, "JETPAKI.VOC".into()); // DUKE_JETPACK_IDLE
    map.insert(51, "JETPAKOF.VOC".into()); // DUKE_JETPACK_OFF
    map.insert(52, "ROAM06.VOC".into()); // LIZTROOP_GROWL / PRED_ROAM
    map.insert(62, "PREDPN.VOC".into()); // LIZARD_PAIN
    map.insert(63, "PREDDY.VOC".into()); // LIZARD_DEATH
    map.insert(64, "LIZSPIT.VOC".into()); // LIZARD_SPIT
    map.insert(69, "SQUISH1A.VOC".into()); // SQUISHED
    map.insert(70, "TELEPORT.VOC".into()); // TELEPORTER
    map.insert(71, "GBELEV01.VOC".into()); // ELEVATOR_ON
    map.insert(73, "GBELEV02.VOC".into()); // ELEVATOR_OFF
    map.insert(74, "CDOOR1B.VOC".into()); // DOOR_OPERATE1
    map.insert(75, "SUBWAY.VOC".into()); // SUBWAY
    map.insert(76, "SWITCH1.VOC".into()); // SWITCH_ON
    map.insert(77, "FAN.VOC".into()); // FAN
    map.insert(78, "GROOVY02.VOC".into()); // DUKE_GETWEAPON3
    map.insert(79, "FLUSH.VOC".into()); // FLUSH_TOILET
    map.insert(81, "TRUMBLE.VOC".into()); // EARTHQUAKE
    map.insert(82, "ALARM1A.VOC".into()); // INTRUDER_ALERT
    map.insert(83, "ENDSEQ.VOC".into()); // END_OF_LEVEL_WARN
    map.insert(109, "SHOTGUN7.VOC".into()); // SHOTGUN_FIRE
    map.insert(110, "FREEZE.VOC".into()); // SOMETHINGFROZE
    map.insert(118, "WPNSEL21.VOC".into()); // SELECT_WEAPON
    map.insert(649, "GOGGLE12.VOC".into()); // NITEVISION_ONOFF
    map.insert(670, "COOL01.VOC".into()); // DUKE_GETWEAPON1
    map.insert(671, "GETSOM1A.VOC".into()); // DUKE_GETWEAPON2
    map.insert(672, "GROOVY02.VOC".into()); // DUKE_GETWEAPON3
    map.insert(674, "HAIL01.VOC".into()); // DUKE_GETWEAPON6
    map.insert(722, "AHH04.VOC".into()); // DUKE_USEMEDKIT
    map.insert(723, "GULP01.VOC".into()); // DUKE_TAKEPILLS

    // Enemy sounds
    map.insert(507, "ROAM06.VOC".into()); // PRED_ROAM
    map.insert(509, "PREDRG.VOC".into()); // PRED_RECOG
    map.insert(510, "GBLASR01.VOC".into()); // PRED_ATTACK
    map.insert(511, "PREDPN.VOC".into()); // PRED_PAIN
    map.insert(512, "PREDDY.VOC".into()); // PRED_DYING

    map.insert(533, "ROAM29.VOC".into()); // PIG_ROAM
    map.insert(536, "PIGRG.VOC".into()); // PIG_RECOG
    map.insert(537, "SHOTGUN7.VOC".into()); // PIG_ATTACK
    map.insert(538, "PIGPN.VOC".into()); // PIG_PAIN
    map.insert(539, "PIGDY.VOC".into()); // PIG_DYING

    map.insert(568, "OCTARM.VOC".into()); // OCTA_ROAM
    map.insert(569, "OCTARG.VOC".into()); // OCTA_RECOG
    map.insert(570, "OCTAAT1.VOC".into()); // OCTA_ATTACK1
    map.insert(572, "OCTAPN.VOC".into()); // OCTA_PAIN
    map.insert(573, "OCTADY.VOC".into()); // OCTA_DYING
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_track_mapping() {
        assert_eq!(LevelMidiTrack::for_level(1, 1), LevelMidiTrack::E1L1Stalker);
        assert_eq!(
            LevelMidiTrack::for_level(1, 2),
            LevelMidiTrack::E1L2Dethtoll
        );
        assert_eq!(LevelMidiTrack::for_level(1, 3), LevelMidiTrack::E1L3Streets);
        assert_eq!(
            LevelMidiTrack::for_level(1, 4),
            LevelMidiTrack::E1L4Watrwrld
        );
        assert_eq!(LevelMidiTrack::for_level(1, 5), LevelMidiTrack::E1L5Snake1);
        assert_eq!(LevelMidiTrack::for_level(1, 6), LevelMidiTrack::E1L6TheCall);
        assert_eq!(LevelMidiTrack::E1L1Stalker.filename(), "STALKER.MID");
    }

    #[test]
    fn test_rts_wad_parsing() {
        let mut wad_bytes = Vec::new();
        wad_bytes.extend_from_slice(b"IWAD");
        wad_bytes.extend_from_slice(&1i32.to_le_bytes());
        wad_bytes.extend_from_slice(&16i32.to_le_bytes());
        wad_bytes.extend_from_slice(b"TEST");

        let mut name = [0u8; 8];
        name[0..4].copy_from_slice(b"DUKE");
        wad_bytes.extend_from_slice(&12i32.to_le_bytes());
        wad_bytes.extend_from_slice(&4i32.to_le_bytes());
        wad_bytes.extend_from_slice(&name);

        let rts = DukeRts::parse(&wad_bytes).expect("Should parse mock RTS");
        let sound = rts.get_sound("DUKE").expect("Should find lump DUKE");
        assert_eq!(sound, b"TEST");
    }

    #[test]
    fn test_default_sound_mappings() {
        let map = build_default_sound_id_map();
        assert_eq!(map.get(&3), Some(&"PISTOL.VOC".to_string()));
        assert_eq!(map.get(&109), Some(&"SHOTGUN7.VOC".to_string()));
        assert_eq!(map.get(&76), Some(&"SWITCH1.VOC".to_string()));
        assert_eq!(map.get(&69), Some(&"SQUISH1A.VOC".to_string()));
    }

    #[test]
    fn test_3d_audio_spatial_attenuation() {
        let cam_pos = Vec3::new(0.0, 0.0, 0.0);
        let near_sound_pos = Vec3::new(0.0, 0.0, 2.0); // 2m away
        let far_sound_pos = Vec3::new(0.0, 0.0, 32.0); // 32m away

        let near_dist = cam_pos.distance(near_sound_pos);
        let near_att = (1.0 / (1.0 + (near_dist / 8.0).powi(2))).clamp(0.05, 1.0);

        let far_dist = cam_pos.distance(far_sound_pos);
        let far_att = (1.0 / (1.0 + (far_dist / 8.0).powi(2))).clamp(0.05, 1.0);

        assert!(near_att > 0.9);
        assert!(far_att < 0.1);
        assert!(near_att > far_att);
    }

    #[test]
    fn test_dynamic_music_and_voice_queue_priority() {
        let mut music = DynamicMusicState::default();
        assert_eq!(music.get_target_volume(), 0.8);

        music.ducking_timer = 2.0;
        assert!((music.get_target_volume() - 0.48).abs() < 0.001); // 0.8 * 0.6 = 0.48

        music.is_underwater = true;
        assert!((music.get_target_volume() - 0.336).abs() < 0.001); // 0.48 * 0.7 = 0.336

        let mut voice = DukeVoiceQueue::default();
        assert!(voice.should_play(5)); // Low priority plays when idle
        assert!(!voice.should_play(2)); // Lower priority rejected while on cooldown
        assert!(voice.should_play(10)); // High priority overrides active line
    }
}
