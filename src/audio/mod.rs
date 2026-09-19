#![allow(dead_code)]

pub mod midi;
pub mod rts;
pub mod sound_defs;
pub mod synth_stream;
pub mod voc;

pub use midi::*;
pub use rts::*;
pub use sound_defs::*;
pub use synth_stream::*;
pub use voc::*;

use crate::grp::Grp;
use crate::player::PlayerMovementMode;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct DukeAudioPlugin;

impl Plugin for DukeAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MidiAudioStream>()
            .init_resource::<DukeAudioAssets>()
            .init_resource::<DynamicMusicState>()
            .init_resource::<DukeVoiceQueue>()
            .init_resource::<DukeRtsResource>()
            .init_resource::<AudioVoiceLimiter>()
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
                    update_underwater_audio_state,
                    update_dynamic_audio_state,
                    sync_music_volume_system,
                    update_ambient_sound_emitters,
                ),
            );
    }
}

pub fn update_underwater_audio_state(
    player_query: Query<&crate::player::PlayerController>,
    mut music_state: ResMut<DynamicMusicState>,
) {
    if let Ok(player) = player_query.get_single() {
        music_state.is_underwater = player.movement_mode == PlayerMovementMode::Diving;
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
    pub underwater_mult: f32,
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
            underwater_mult: 1.0,
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
            vol *= 0.4; // Muffle underwater (attenuate by 60%)
        }
        vol
    }

    pub fn tick(&mut self, dt: f32) {
        if self.ducking_timer > 0.0 {
            self.ducking_timer = (self.ducking_timer - dt).max(0.0);
        }
        let target_mult = if self.is_underwater { 0.4 } else { 1.0 };
        self.underwater_mult += (target_mult - self.underwater_mult) * (dt * 5.0).min(1.0);
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

/// Represents an ambient environment sound emitter in the map (e.g. MUSICANDSFX sprites).
#[derive(Component, Debug, Clone)]
pub struct AmbientSoundEmitter {
    pub sound_id: i32,
    pub range: f32,
    pub repeat_delay: f32,
    pub timer: f32,
}

pub fn update_ambient_sound_emitters(
    time: Res<Time>,
    mut emitters: Query<(&Transform, &mut AmbientSoundEmitter)>,
    player_query: Query<&Transform, With<crate::player::PlayerController>>,
    music_state: Option<Res<DynamicMusicState>>,
    mut sound_events: EventWriter<PlaySpatialSoundEvent>,
) {
    let dt = time.delta_seconds();
    let Ok(player_trans) = player_query.get_single() else {
        return;
    };
    let p_pos = player_trans.translation;
    let underwater_mult = if let Some(ref state) = music_state {
        state.underwater_mult
    } else {
        1.0
    };

    for (trans, mut emitter) in emitters.iter_mut() {
        emitter.timer -= dt;
        if emitter.timer <= 0.0 {
            emitter.timer = emitter.repeat_delay;
            let dist = trans.translation.distance(p_pos);
            if dist <= emitter.range {
                let base_vol = (1.0 - (dist / emitter.range).clamp(0.0, 1.0)).max(0.1);
                let volume = base_vol * underwater_mult;
                sound_events.send(PlaySpatialSoundEvent {
                    sound_id: emitter.sound_id,
                    volume,
                    position: trans.translation,
                });
            }
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct DukeRtsResource(pub DukeRts);

pub const MAX_ACTIVE_VOICES: usize = 32;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceChannel {
    pub sequence: u64,
}

#[derive(Resource, Debug, Clone)]
pub struct AudioVoiceLimiter {
    pub max_voices: usize,
    pub next_id: u64,
}

impl Default for AudioVoiceLimiter {
    fn default() -> Self {
        Self {
            max_voices: MAX_ACTIVE_VOICES,
            next_id: 0,
        }
    }
}

pub fn cull_oldest_voice_if_needed(
    current_voices: &mut Vec<(Entity, u64)>,
    max_voices: usize,
    commands: &mut Commands,
) {
    while current_voices.len() >= max_voices {
        if let Some((idx, &(oldest_entity, _))) = current_voices
            .iter()
            .enumerate()
            .min_by_key(|(_, &(_, seq))| seq)
        {
            commands.entity(oldest_entity).despawn_recursive();
            current_voices.remove(idx);
        } else {
            break;
        }
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
    pub wav_music_tracks: HashMap<LevelMidiTrack, Handle<AudioSource>>,
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
        let mapped = match name.to_uppercase().as_str() {
            "LOOKING_GOOD" => "LOOK01",
            "FOUND_SECRET" => "COOL01",
            "BONUS_SPEECH1" => "DMDG",
            "BONUS_SPEECH2" => "GROOVY",
            _ => name,
        };
        let upper = mapped.to_uppercase();
        if let Some(handle) = self.sounds_by_name.get(&upper) {
            return Some(handle.clone());
        }
        let stripped = upper.trim_end_matches(".VOC").trim_end_matches(".WAV");
        if let Some(handle) = self.sounds_by_name.get(stripped) {
            return Some(handle.clone());
        }
        // Prefix search fallback (e.g. LOOK01 matching LOOKIN01 or vice versa, GROOVY matching GROOVY02)
        for (key, handle) in &self.sounds_by_name {
            if key.starts_with(stripped) || stripped.starts_with(key) {
                return Some(handle.clone());
            }
        }
        // Voice fallbacks for bonus speeches and look quotes if specific stem was not present
        if stripped == "DMDG" {
            if let Some(handle) = self.sounds_by_name.get("LETSRK03") {
                return Some(handle.clone());
            }
            if let Some(handle) = self.sounds_by_name.get("DAMN03") {
                return Some(handle.clone());
            }
        }
        if stripped == "LOOK01" {
            if let Some(handle) = self.sounds_by_name.get("LOOKIN01") {
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

    // 3b. Read and parse DUKE.RTS lump file if present in GRP
    if let Ok(rts_bytes) = grp.read_file("DUKE.RTS") {
        if let Ok(duke_rts) = crate::audio::rts::DukeRts::parse(&rts_bytes) {
            println!(
                "DukeAudioPlugin: Loaded DUKE.RTS with {} speech lumps",
                duke_rts.lumps.len()
            );
            for (lump_name, lump_data) in &duke_rts.lumps {
                if let Ok(voc_sound) = VocSound::parse(lump_name, lump_data) {
                    let wav_bytes = voc_sound.to_wav_bytes();
                    let audio_source = AudioSource {
                        bytes: wav_bytes.into(),
                    };
                    let handle = audio_sources.add(audio_source);
                    audio_assets
                        .sounds_by_name
                        .insert(format!("RTS_{}", lump_name), handle.clone());
                    audio_assets
                        .sounds_by_name
                        .insert(lump_name.clone(), handle);
                }
            }
            commands.insert_resource(DukeRtsResource(duke_rts));
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
    let sf2_path = "assets/TimGM6mb.sf2";
    let sf2_soundfont = if std::path::Path::new(sf2_path).exists() {
        if let Ok(soundfont_data) = std::fs::read(sf2_path) {
            let mut reader = std::io::Cursor::new(soundfont_data);
            if let Ok(sf) = rustysynth::SoundFont::new(&mut reader) {
                Some(std::sync::Arc::new(sf))
            } else {
                println!(
                    "DukeAudioPlugin: Warning - failed to parse SoundFont at {}! Falling back to MidiSynth.",
                    sf2_path
                );
                None
            }
        } else {
            None
        }
    } else {
        println!("DukeAudioPlugin: Warning - assets/TimGM6mb.sf2 not found! Falling back to built-in MidiSynth synthesizer.");
        None
    };

    let tracks_to_load = [
        (LevelMidiTrack::E1L1Stalker, "STALKER.MID"),
        (LevelMidiTrack::TitleGrabbag, "GRABBAG.MID"),
    ];

    if let Some(soundfont) = sf2_soundfont {
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

    // Fallback: If SoundFont is missing or loading fails, do NOT leave background music silent!
    // Fall back to the built-in pure-Rust software synthesizer:
    if audio_assets.music_tracks.is_empty() {
        println!("DukeAudioPlugin: Synthesizing background music with built-in pure-Rust software synthesizer (MidiSynth)...");
        let synth = crate::audio::midi::MidiSynth::new(22050);
        for (track_enum, midi_filename) in &tracks_to_load {
            if let Ok(midi_data) = grp.read_file(midi_filename) {
                if let Ok(wav_bytes) = synth.midi_to_wav(&midi_data) {
                    let audio_source = AudioSource {
                        bytes: wav_bytes.into(),
                    };
                    let handle = audio_sources.add(audio_source);
                    audio_assets.wav_music_tracks.insert(*track_enum, handle);
                    println!(
                        "DukeAudioPlugin: Synthesized fallback background music (MidiSynth): {}",
                        midi_filename
                    );
                }
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
    } else if let Some(e1l1_wav) = audio_assets.wav_music_tracks.get(&LevelMidiTrack::E1L1Stalker) {
        commands.spawn((
            AudioBundle {
                source: e1l1_wav.clone(),
                settings: PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::new(0.6)),
            },
            MusicTrackEmitter,
        ));
        println!("DukeAudioPlugin: Background music playing via MidiSynth fallback (STALKER.MID - Episode 1 Level 1)");
    }

    println!(
        "DukeAudioPlugin: Successfully loaded {} VOC sound effects ({} sound IDs mapped, {} voice quotes ready, {} music tracks)",
        voc_count,
        audio_assets.sounds_by_id.len(),
        audio_assets.duke_quotes.len(),
        audio_assets.music_tracks.len() + audio_assets.wav_music_tracks.len()
    );
}

pub fn handle_play_sound_events(
    mut events: EventReader<PlaySoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut voice_limiter: Option<ResMut<AudioVoiceLimiter>>,
    voice_query: Query<(Entity, &VoiceChannel)>,
    mut commands: Commands,
) {
    let max_voices = voice_limiter
        .as_ref()
        .map(|l| l.max_voices)
        .unwrap_or(MAX_ACTIVE_VOICES);

    let mut current_voices: Vec<(Entity, u64)> = voice_query
        .iter()
        .map(|(e, vc)| (e, vc.sequence))
        .collect();

    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_id(ev.sound_id) {
            cull_oldest_voice_if_needed(&mut current_voices, max_voices, &mut commands);

            let seq = if let Some(ref mut limiter) = voice_limiter {
                let id = limiter.next_id;
                limiter.next_id += 1;
                id
            } else {
                0
            };

            let entity = commands
                .spawn((
                    AudioBundle {
                        source: handle,
                        ..default()
                    },
                    VoiceChannel { sequence: seq },
                ))
                .id();
            current_voices.push((entity, seq));
        }
    }
}

pub fn handle_play_spatial_sound_events(
    mut events: EventReader<PlaySpatialSoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut voice_limiter: Option<ResMut<AudioVoiceLimiter>>,
    voice_query: Query<(Entity, &VoiceChannel)>,
    mut commands: Commands,
) {
    let max_voices = voice_limiter
        .as_ref()
        .map(|l| l.max_voices)
        .unwrap_or(MAX_ACTIVE_VOICES);

    let mut current_voices: Vec<(Entity, u64)> = voice_query
        .iter()
        .map(|(e, vc)| (e, vc.sequence))
        .collect();

    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_id(ev.sound_id) {
            cull_oldest_voice_if_needed(&mut current_voices, max_voices, &mut commands);

            let seq = if let Some(ref mut limiter) = voice_limiter {
                let id = limiter.next_id;
                limiter.next_id += 1;
                id
            } else {
                0
            };

            let settings = PlaybackSettings::default()
                .with_volume(bevy::audio::Volume::new(ev.volume))
                .with_spatial(true);
            let entity = commands
                .spawn((
                    AudioBundle {
                        source: handle,
                        settings,
                    },
                    TransformBundle::from_transform(Transform::from_translation(ev.position)),
                    VoiceChannel { sequence: seq },
                ))
                .id();
            current_voices.push((entity, seq));
        }
    }
}

pub fn handle_play_named_sound_events(
    mut events: EventReader<PlayNamedSoundEvent>,
    audio_assets: Res<DukeAudioAssets>,
    mut voice_limiter: Option<ResMut<AudioVoiceLimiter>>,
    voice_query: Query<(Entity, &VoiceChannel)>,
    mut commands: Commands,
) {
    let max_voices = voice_limiter
        .as_ref()
        .map(|l| l.max_voices)
        .unwrap_or(MAX_ACTIVE_VOICES);

    let mut current_voices: Vec<(Entity, u64)> = voice_query
        .iter()
        .map(|(e, vc)| (e, vc.sequence))
        .collect();

    for ev in events.read() {
        if let Some(handle) = audio_assets.get_sound_by_name(&ev.name) {
            cull_oldest_voice_if_needed(&mut current_voices, max_voices, &mut commands);

            let seq = if let Some(ref mut limiter) = voice_limiter {
                let id = limiter.next_id;
                limiter.next_id += 1;
                id
            } else {
                0
            };

            let mut settings =
                PlaybackSettings::default().with_volume(bevy::audio::Volume::new(ev.volume));

            let entity = if let Some(pos) = ev.position {
                settings = settings.with_spatial(true);
                commands
                    .spawn((
                        AudioBundle {
                            source: handle,
                            settings,
                        },
                        TransformBundle::from_transform(Transform::from_translation(pos)),
                        VoiceChannel { sequence: seq },
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        AudioBundle {
                            source: handle,
                            settings,
                        },
                        VoiceChannel { sequence: seq },
                    ))
                    .id()
            };
            current_voices.push((entity, seq));
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
        } else if let Some(wav_handle) = audio_assets.wav_music_tracks.get(&ev.track) {
            // Despawn old tracks
            for entity in &old_music {
                commands.entity(entity).despawn_recursive();
            }

            music_state.current_track = ev.track.filename().to_string();

            // Spawn new track
            commands.spawn((
                AudioBundle {
                    source: wav_handle.clone(),
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
    rts_resource: Option<Res<DukeRtsResource>>,
    mut audio_sources: Option<ResMut<Assets<AudioSource>>>,
    mut voice_queue: Option<ResMut<DukeVoiceQueue>>,
    mut music_state: Option<ResMut<DynamicMusicState>>,
    mut voice_limiter: Option<ResMut<AudioVoiceLimiter>>,
    voice_query: Query<(Entity, &VoiceChannel)>,
    mut commands: Commands,
) {
    let max_voices = voice_limiter
        .as_ref()
        .map(|l| l.max_voices)
        .unwrap_or(MAX_ACTIVE_VOICES);

    let mut current_voices: Vec<(Entity, u64)> = voice_query
        .iter()
        .map(|(e, vc)| (e, vc.sequence))
        .collect();

    for ev in events.read() {
        let sound_name = match ev.name.as_deref() {
            Some("LOOKING_GOOD") => Some("LOOK01"),
            Some("FOUND_SECRET") => Some("COOL01"),
            other => other,
        };

        let priority = if let Some(n) = sound_name {
            if n.contains("REST_IN_PIECES") || n.contains("EAT_SHIT") || n.contains("HAIL") {
                10
            } else if n.contains("LOOK") {
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

        if !can_play {
            continue;
        }

        let mut handle_to_play: Option<Handle<AudioSource>> = None;

        if let Some(name) = sound_name {
            let is_rts_random = name.eq_ignore_ascii_case("RTS")
                || name.eq_ignore_ascii_case("TAUNT")
                || name.eq_ignore_ascii_case("RTS_TAUNT");

            if is_rts_random {
                if let Some(ref rts) = rts_resource {
                    if let Some((lump_name, lump_data)) = rts.0.sample_random_taunt() {
                        if let Some(handle) = audio_assets.get_sound_by_name(lump_name) {
                            handle_to_play = Some(handle);
                        } else if let Some(ref mut sources) = audio_sources {
                            let src = DukeRts::lump_to_audio_source(lump_data);
                            handle_to_play = Some(sources.add(src));
                        }
                    }
                }
            } else if let Some(ref rts) = rts_resource {
                let rts_key = name.strip_prefix("RTS_").unwrap_or(name);
                if let Some(lump_data) = rts.0.get_sound(rts_key) {
                    if let Some(handle) = audio_assets
                        .get_sound_by_name(name)
                        .or_else(|| audio_assets.get_sound_by_name(rts_key))
                    {
                        handle_to_play = Some(handle);
                    } else if let Some(ref mut sources) = audio_sources {
                        let src = DukeRts::lump_to_audio_source(lump_data);
                        handle_to_play = Some(sources.add(src));
                    }
                } else if let Some(handle) = audio_assets.get_sound_by_name(name) {
                    handle_to_play = Some(handle);
                }
            } else if let Some(handle) = audio_assets.get_sound_by_name(name) {
                handle_to_play = Some(handle);
            }
        } else {
            // Random taunt (ev.name is None)
            if let Some(ref rts) = rts_resource {
                if let Some((lump_name, lump_data)) = rts.0.sample_random_taunt() {
                    if let Some(handle) = audio_assets.get_sound_by_name(lump_name) {
                        handle_to_play = Some(handle);
                    } else if let Some(ref mut sources) = audio_sources {
                        let src = DukeRts::lump_to_audio_source(lump_data);
                        handle_to_play = Some(sources.add(src));
                    }
                }
            }
            if handle_to_play.is_none() && !audio_assets.duke_quotes.is_empty() {
                let idx = rand::random::<usize>() % audio_assets.duke_quotes.len();
                handle_to_play = audio_assets.duke_quotes.get(idx).cloned();
            }
        }

        if let Some(handle) = handle_to_play {
            if let Some(ref mut music) = music_state {
                music.ducking_timer = 2.5; // Duck background music for 2.5s only when sound exists
            }

            cull_oldest_voice_if_needed(&mut current_voices, max_voices, &mut commands);

            let seq = if let Some(ref mut limiter) = voice_limiter {
                let id = limiter.next_id;
                limiter.next_id += 1;
                id
            } else {
                0
            };

            let entity = commands
                .spawn((
                    AudioBundle {
                        source: handle,
                        ..default()
                    },
                    VoiceChannel { sequence: seq },
                ))
                .id();
            current_voices.push((entity, seq));
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
    map.insert(88, "SECRET.VOC".into()); // SECRET_AREA
    map.insert(109, "SHOTGUN7.VOC".into()); // SHOTGUN_FIRE
    map.insert(110, "FREEZE.VOC".into()); // SOMETHINGFROZE
    map.insert(118, "WPNSEL21.VOC".into()); // SELECT_WEAPON
    map.insert(195, "DMDG.VOC".into()); // BONUS_SPEECH1 ("Damn, I'm good!")
    map.insert(196, "GROOVY.VOC".into()); // BONUS_SPEECH2 ("Groovy!")
    map.insert(252, "LOOKIN01.VOC".into()); // DUKE_LOOKINTOMIRROR
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
        assert_eq!(map.get(&0), Some(&"KICKHIT.VOC".to_string()));
        assert_eq!(map.get(&2), Some(&"BULITHIT.VOC".to_string()));
        assert_eq!(map.get(&3), Some(&"PISTOL.VOC".to_string()));
        assert_eq!(map.get(&88), Some(&"SECRET.VOC".to_string()));
        assert_eq!(map.get(&109), Some(&"SHOTGUN7.VOC".to_string()));
        assert_eq!(map.get(&76), Some(&"SWITCH1.VOC".to_string()));
        assert_eq!(map.get(&69), Some(&"SQUISH1A.VOC".to_string()));
        assert_eq!(map.get(&195), Some(&"DMDG.VOC".to_string()));
        assert_eq!(map.get(&196), Some(&"GROOVY.VOC".to_string()));
        assert_eq!(map.get(&252), Some(&"LOOKIN01.VOC".to_string()));
    }

    #[test]
    fn test_voice_aliases_and_ducking() {
        let mut assets = DukeAudioAssets::default();
        // Insert mock source
        let mut dummy_sources = Assets::<AudioSource>::default();
        let handle = dummy_sources.add(AudioSource {
            bytes: vec![0u8; 100].into(),
        });
        assets.sounds_by_name.insert("COOL01".to_string(), handle.clone());
        assets.sounds_by_name.insert("LOOKIN01".to_string(), handle.clone());

        // Test alias lookups
        assert!(assets.get_sound_by_name("FOUND_SECRET").is_some());
        assert!(assets.get_sound_by_name("LOOKING_GOOD").is_some());
        assert!(assets.get_sound_by_name("NON_EXISTENT_VOICE").is_none());
    }

    #[test]
    fn test_midisynth_fallback_wav_generation() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            if let Ok(midi_data) = grp.read_file("STALKER.MID") {
                let synth = crate::audio::midi::MidiSynth::new(22050);
                let wav_res = synth.midi_to_wav(&midi_data);
                assert!(wav_res.is_ok());
                let wav_bytes = wav_res.unwrap();
                assert!(wav_bytes.starts_with(b"RIFF"));
                let audio_source = AudioSource {
                    bytes: wav_bytes.into(),
                };
                let mut dummy_sources = Assets::<AudioSource>::default();
                let handle = dummy_sources.add(audio_source);
                assert!(handle.id() != bevy::asset::AssetId::invalid());
            }
        }
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
        assert!((music.get_target_volume() - 0.192).abs() < 0.001); // 0.8 * 0.6 * 0.4 = 0.192

        let mut voice = DukeVoiceQueue::default();
        assert!(voice.should_play(5)); // Low priority plays when idle
        assert!(!voice.should_play(2)); // Lower priority rejected while on cooldown
        assert!(voice.should_play(10)); // High priority overrides active line
    }

    #[test]
    fn test_underwater_audio_sync_and_volume_attenuation() {
        let mut app = App::new();
        app.init_resource::<DynamicMusicState>();

        // Spawn player in Diving mode
        let mut player = crate::player::PlayerController::default();
        player.movement_mode = PlayerMovementMode::Diving;
        let player_entity = app.world_mut().spawn(player).id();

        // Run underwater audio state system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_underwater_audio_state);
        schedule.run(app.world_mut());

        let music_state = app.world().resource::<DynamicMusicState>();
        assert!(music_state.is_underwater, "Music state should be marked underwater when player is diving");

        // Underwater attenuation: volume multiplier 0.4 (attenuated by 60%)
        let underwater_vol = music_state.get_target_volume();
        let base_vol = music_state.base_volume;
        assert!((underwater_vol - (base_vol * 0.4)).abs() < 0.001, "Target volume should be attenuated by 60% (mult 0.4)");

        // Test transition out of water and smooth lerping back towards 1.0
        app.world_mut().entity_mut(player_entity).get_mut::<crate::player::PlayerController>().unwrap().movement_mode = PlayerMovementMode::Standing;

        schedule.run(app.world_mut());

        let mut music_state = app.world_mut().resource_mut::<DynamicMusicState>();
        assert!(!music_state.is_underwater, "Music state should no longer be underwater when player is standing");
        assert!((music_state.get_target_volume() - base_vol).abs() < 0.001);

        // Simulate ticking/lerping volume back to 1.0
        music_state.underwater_mult = 0.4;
        music_state.tick(0.1);
        assert!(music_state.underwater_mult > 0.4, "Underwater multiplier should smoothly lerp back towards 1.0");
        for _ in 0..20 {
            music_state.tick(0.1);
        }
        assert!((music_state.underwater_mult - 1.0).abs() < 0.01, "Underwater multiplier should reach 1.0");
    }

    #[test]
    fn test_rts_resource_loading_and_taunt_dispatch() {
        let mut app = App::new();
        app.insert_resource(Assets::<AudioSource>::default())
            .init_resource::<DukeAudioAssets>()
            .init_resource::<DynamicMusicState>()
            .init_resource::<AudioVoiceLimiter>()
            .add_event::<PlayDukeVoiceEvent>();

        // Build mock DUKE.RTS lump data (WAD format)
        let mut wad_bytes = Vec::new();
        wad_bytes.extend_from_slice(b"IWAD");
        wad_bytes.extend_from_slice(&2i32.to_le_bytes()); // 2 lumps
        wad_bytes.extend_from_slice(&12i32.to_le_bytes()); // infotable offset

        // Lump directory: Lump 1 = "TAUNT1" at offset 44, size 4
        wad_bytes.extend_from_slice(&44i32.to_le_bytes());
        wad_bytes.extend_from_slice(&4i32.to_le_bytes());
        let mut name1 = [0u8; 8];
        name1[0..6].copy_from_slice(b"TAUNT1");
        wad_bytes.extend_from_slice(&name1);

        // Lump 2 = "TAUNT2" at offset 48, size 4
        wad_bytes.extend_from_slice(&48i32.to_le_bytes());
        wad_bytes.extend_from_slice(&4i32.to_le_bytes());
        let mut name2 = [0u8; 8];
        name2[0..6].copy_from_slice(b"TAUNT2");
        wad_bytes.extend_from_slice(&name2);

        // Lump contents
        wad_bytes.extend_from_slice(b"WAV1");
        wad_bytes.extend_from_slice(b"WAV2");

        let rts = DukeRts::parse(&wad_bytes).expect("Failed to parse mock RTS");
        assert_eq!(rts.lumps.len(), 2);
        assert_eq!(rts.get_sound("TAUNT1"), Some(b"WAV1".as_slice()));
        assert_eq!(rts.get_sound("TAUNT2"), Some(b"WAV2".as_slice()));

        // Store in Bevy resource DukeRtsResource
        app.insert_resource(DukeRtsResource(rts));

        // Send random taunt event (name: None)
        app.world_mut().send_event(PlayDukeVoiceEvent { name: None });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_play_duke_voice_events);
        schedule.run(app.world_mut());

        // Verify voice entity spawned
        let count = app.world_mut().query::<&VoiceChannel>().iter(app.world()).count();
        assert_eq!(count, 1, "RTS taunt event should spawn an audio voice entity");

        // Verify music ducking was activated
        let music_state = app.world().resource::<DynamicMusicState>();
        assert!(music_state.ducking_timer > 0.0, "Voice taunt should activate music ducking timer");
    }

    #[test]
    fn test_audio_voice_limiter_channel_culling() {
        let mut app = App::new();
        let mut dummy_sources = Assets::<AudioSource>::default();
        let dummy_handle = dummy_sources.add(AudioSource {
            bytes: vec![0u8; 100].into(),
        });

        let mut assets = DukeAudioAssets::default();
        assets.sounds_by_id.insert(1, dummy_handle.clone());
        app.insert_resource(assets);
        app.insert_resource(dummy_sources);
        app.add_event::<PlaySoundEvent>();

        // Configure limiter with max 4 voices
        app.insert_resource(AudioVoiceLimiter {
            max_voices: 4,
            next_id: 0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_play_sound_events);

        // Spawn 8 sounds sequentially
        for _ in 0..8 {
            app.world_mut().send_event(PlaySoundEvent { sound_id: 1 });
            schedule.run(app.world_mut());
        }

        // Verify total active voice entities do NOT exceed 4
        let remaining_channels: Vec<u64> = app.world_mut().query::<&VoiceChannel>().iter(app.world()).map(|v| v.sequence).collect();
        assert_eq!(remaining_channels.len(), 4, "Active voices must be capped at 4");

        // Oldest voices (sequence 0, 1, 2, 3) should have been culled, leaving sequence 4, 5, 6, 7
        for seq in remaining_channels {
            assert!(seq >= 4, "Oldest voices should be stolen/culled by voice limiter");
        }
    }
}
