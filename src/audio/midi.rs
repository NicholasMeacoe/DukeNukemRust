#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Cursor;
use hound::{WavSpec, WavWriter, SampleFormat};
use midly::{Smf, TrackEventKind, MidiMessage, MetaMessage, Timing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelMidiTrack {
    TitleGrabbag,
    E1L1Stalker,
    E1L2Dethtoll,
    E1L3Streets,
    E1L4Watrwrld,
    E1L5Snake1,
    E1L6TheCall,
    E1L7Ahgeez,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicState {
    #[default]
    AmbientExploration,
    CombatTension,
    SecretFoundJingle,
    BossFight,
}

impl LevelMidiTrack {
    pub fn filename(&self) -> &'static str {
        match self {
            LevelMidiTrack::TitleGrabbag => "GRABBAG.MID",
            LevelMidiTrack::E1L1Stalker => "STALKER.MID",
            LevelMidiTrack::E1L2Dethtoll => "DETHTOLL.MID",
            LevelMidiTrack::E1L3Streets => "STREETS.MID",
            LevelMidiTrack::E1L4Watrwrld => "WATRWLD1.MID",
            LevelMidiTrack::E1L5Snake1 => "SNAKE1.MID",
            LevelMidiTrack::E1L6TheCall => "THECALL.MID",
            LevelMidiTrack::E1L7Ahgeez => "AHGEEZ.MID",
        }
    }

    pub fn for_level(episode: usize, level: usize) -> Self {
        match (episode, level) {
            (1, 1) => LevelMidiTrack::E1L1Stalker,
            (1, 2) => LevelMidiTrack::E1L2Dethtoll,
            (1, 3) => LevelMidiTrack::E1L3Streets,
            (1, 4) => LevelMidiTrack::E1L4Watrwrld,
            (1, 5) => LevelMidiTrack::E1L5Snake1,
            (1, 6) => LevelMidiTrack::E1L6TheCall,
            (1, 7) => LevelMidiTrack::E1L7Ahgeez,
            _ => LevelMidiTrack::TitleGrabbag,
        }
    }

    pub fn filename_for_campaign_map(episode: usize, level: usize) -> &'static str {
        for map in crate::campaign::ALL_CAMPAIGN_MAPS {
            if map.episode == episode && map.level == level {
                return map.music_filename;
            }
        }
        "GRABBAG.MID"
    }
}

/// A timed MIDI note event
#[derive(Clone, Debug)]
struct TimedNote {
    start_sample: usize,
    end_sample: usize,
    channel: u8,
    key: u8,
    velocity: u8,
    instrument: u8,
    is_drum: bool,
}

/// Pure Rust retro MIDI software synthesizer
pub struct MidiSynth {
    pub sample_rate: u32,
}

impl Default for MidiSynth {
    fn default() -> Self {
        Self { sample_rate: 22050 }
    }
}

impl MidiSynth {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Converts raw MIDI bytes into a standard 16-bit PCM WAV byte stream
    pub fn midi_to_wav(&self, midi_bytes: &[u8]) -> Result<Vec<u8>, String> {
        let smf = Smf::parse(midi_bytes).map_err(|e| format!("MIDI parse error: {:?}", e))?;
        let ticks_per_beat = match smf.header.timing {
            Timing::Metrical(t) => t.as_int() as f64,
            Timing::Timecode(fps, sub) => (fps.as_f32() * sub as f32) as f64,
        };

        let mut tempo_micros_per_quarter = 500_000.0; // Default 120 BPM
        let mut notes: Vec<TimedNote> = Vec::new();

        // Extract and flatten events across tracks
        for track in smf.tracks {
            let mut current_sec: f64 = 0.0;
            let mut active_notes: HashMap<(u8, u8), (usize, u8)> = HashMap::new(); // (channel, key) -> (start_sample, velocity)
            let mut current_prog: [u8; 16] = [0; 16];

            for event in track {
                let delta = event.delta.as_int() as u64;
                if delta > 0 {
                    let seconds_per_tick = (tempo_micros_per_quarter / 1_000_000.0) / ticks_per_beat;
                    current_sec += delta as f64 * seconds_per_tick;
                }

                let current_sample = (current_sec * self.sample_rate as f64) as usize;

                match event.kind {
                    TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                        tempo_micros_per_quarter = tempo.as_int() as f64;
                    }
                    TrackEventKind::Midi { channel, message } => {
                        let ch = channel.as_int();
                        let is_drum = ch == 9; // MIDI channel 10 (0-indexed 9) is Percussion
                        match message {
                            MidiMessage::ProgramChange { program } => {
                                current_prog[ch as usize] = program.as_int();
                            }
                            MidiMessage::NoteOn { key, vel } => {
                                let key_num = key.as_int();
                                let vel_num = vel.as_int();
                                if vel_num > 0 {
                                    active_notes.insert((ch, key_num), (current_sample, vel_num));
                                } else if let Some((start_sample, start_vel)) = active_notes.remove(&(ch, key_num)) {
                                    notes.push(TimedNote {
                                        start_sample,
                                        end_sample: current_sample.max(start_sample + (self.sample_rate as f64 * 0.05) as usize),
                                        channel: ch,
                                        key: key_num,
                                        velocity: start_vel,
                                        instrument: current_prog[ch as usize],
                                        is_drum,
                                    });
                                }
                            }
                            MidiMessage::NoteOff { key, .. } => {
                                let key_num = key.as_int();
                                if let Some((start_sample, start_vel)) = active_notes.remove(&(ch, key_num)) {
                                    notes.push(TimedNote {
                                        start_sample,
                                        end_sample: current_sample.max(start_sample + (self.sample_rate as f64 * 0.05) as usize),
                                        channel: ch,
                                        key: key_num,
                                        velocity: start_vel,
                                        instrument: current_prog[ch as usize],
                                        is_drum,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Close any remaining active notes
            let end_sample = (current_sec * self.sample_rate as f64) as usize;
            for ((ch, key_num), (start_sample, vel)) in active_notes {
                notes.push(TimedNote {
                    start_sample,
                    end_sample: end_sample.max(start_sample + (self.sample_rate as f64 * 0.1) as usize),
                    channel: ch,
                    key: key_num,
                    velocity: vel,
                    instrument: current_prog[ch as usize],
                    is_drum: ch == 9,
                });
            }
        }

        if notes.is_empty() {
            return Err("No notes found in MIDI file".to_string());
        }

        let total_samples = notes.iter().map(|n| n.end_sample).max().unwrap_or(0) + (self.sample_rate as usize);
        let mut buffer: Vec<f32> = vec![0.0; total_samples];

        // Synthesize each note into the buffer
        for note in &notes {
            let start = note.start_sample;
            let end = note.end_sample.min(total_samples);
            let len = end.saturating_sub(start);
            if len == 0 { continue; }

            let vel_scale = (note.velocity as f32 / 127.0) * 0.25;

            if note.is_drum {
                // Drum synthesis (Kick, Snare, Hi-hat, Crash, Tom)
                let drum_key = note.key;
                let decay_time = match drum_key {
                    35 | 36 => 0.25, // Bass Drum
                    38 | 40 => 0.20, // Snare
                    42 | 44 => 0.08, // Closed Hi-hat
                    46 | 49 | 57 => 0.60, // Open Hat / Crash
                    _ => 0.15,
                };
                let drum_len = ((decay_time * self.sample_rate as f32) as usize).min(total_samples - start);

                for i in 0..drum_len {
                    let t = i as f32 / self.sample_rate as f32;
                    let env = (1.0 - t / decay_time).max(0.0);
                    let sample = match drum_key {
                        35 | 36 => {
                            // Pitch-drop sine wave for kick drum
                            let freq = 120.0 * (-t * 20.0).exp() + 45.0;
                            (t * freq * std::f32::consts::TAU).sin() * env * env
                        }
                        38 | 40 => {
                            // Snare: noise + tone
                            let noise = (rand::random::<f32>() - 0.5) * 2.0;
                            let tone = (t * 180.0 * std::f32::consts::TAU).sin();
                            (noise * 0.7 + tone * 0.3) * env
                        }
                        42 | 44 | 46 | 49 | 57 => {
                            // Cymbals / Hi-hats: filtered high-frequency noise
                            let noise = (rand::random::<f32>() - 0.5) * 2.0;
                            noise * env
                        }
                        _ => {
                            let freq = 80.0 + (drum_key as f32 * 5.0);
                            (t * freq * std::f32::consts::TAU).sin() * env
                        }
                    };
                    buffer[start + i] += sample * vel_scale * 1.2;
                }
            } else {
                // Melodic synthesis (FM/Multi-oscillator General MIDI emulation)
                let freq = 440.0 * 2.0f32.powf((note.key as f32 - 69.0) / 12.0);
                let duration_sec = len as f32 / self.sample_rate as f32;
                let attack = 0.015f32;
                let release = 0.05f32;

                for i in 0..len {
                    let t = i as f32 / self.sample_rate as f32;
                    // ADSR envelope
                    let env = if t < attack {
                        t / attack
                    } else if t > duration_sec - release {
                        ((duration_sec - t) / release).max(0.0)
                    } else {
                        1.0 - (t / duration_sec) * 0.3
                    };

                    let phase = t * freq * std::f32::consts::TAU;
                    
                    // Rich retro synth timbre based on instrument family
                    let s = match note.instrument {
                        // Bass instruments (32..39): Square/Saw tooth mix
                        32..=39 => {
                            let saw = 2.0 * (phase / std::f32::consts::TAU).fract() - 1.0;
                            let sub = (phase * 0.5).sin();
                            saw * 0.6 + sub * 0.4
                        }
                        // Guitars (24..31): Overdriven / harmonic rich wave
                        24..=31 => {
                            let raw = (phase.sin() + 0.5 * (phase * 2.0).sin() + 0.25 * (phase * 3.0).sin()) * 1.5;
                            raw.clamp(-0.8, 0.8)
                        }
                        // Strings/Pads (40..55): Smooth multi-sine blend
                        40..=55 => {
                            phase.sin() * 0.6 + (phase * 2.01).sin() * 0.3 + (phase * 3.0).sin() * 0.1
                        }
                        // Brass / Reed (56..79): Bright sawtooth
                        56..=79 => {
                            let saw = 2.0 * (phase / std::f32::consts::TAU).fract() - 1.0;
                            saw * 0.7 + (phase * 2.0).sin() * 0.3
                        }
                        // Default / Pianos / Synths: Warm triangle/sine combo
                        _ => {
                            let tri = 2.0 * (2.0 * (phase / std::f32::consts::TAU).fract() - 1.0).abs() - 1.0;
                            tri * 0.5 + phase.sin() * 0.5
                        }
                    };

                    buffer[start + i] += s * env * vel_scale;
                }
            }
        }

        // Apply soft limiter & convert to 16-bit PCM WAV
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut out = Vec::new();
        {
            let mut writer = WavWriter::new(Cursor::new(&mut out), spec)
                .map_err(|e| format!("WavWriter error: {:?}", e))?;
            for &sample in &buffer {
                let clamped = sample.clamp(-1.0, 1.0);
                let pcm_16 = (clamped * 32767.0) as i16;
                writer.write_sample(pcm_16).map_err(|e| format!("Write sample error: {:?}", e))?;
            }
            writer.finalize().map_err(|e| format!("WAV finalize error: {:?}", e))?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_track_mapping() {
        assert_eq!(LevelMidiTrack::for_level(1, 1), LevelMidiTrack::E1L1Stalker);
        assert_eq!(LevelMidiTrack::for_level(1, 2), LevelMidiTrack::E1L2Dethtoll);
        assert_eq!(LevelMidiTrack::for_level(1, 3), LevelMidiTrack::E1L3Streets);
        assert_eq!(LevelMidiTrack::for_level(1, 4), LevelMidiTrack::E1L4Watrwrld);
        assert_eq!(LevelMidiTrack::for_level(1, 5), LevelMidiTrack::E1L5Snake1);
        assert_eq!(LevelMidiTrack::for_level(1, 6), LevelMidiTrack::E1L6TheCall);
        assert_eq!(LevelMidiTrack::E1L1Stalker.filename(), "STALKER.MID");
    }

    #[test]
    fn test_synthesize_real_grp_midi_stalker() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            if let Ok(midi_data) = grp.read_file("STALKER.MID") {
                let synth = MidiSynth::new(22050);
                let wav_res = synth.midi_to_wav(&midi_data);
                assert!(wav_res.is_ok(), "Failed to synthesize STALKER.MID: {:?}", wav_res.err());
                let wav = wav_res.unwrap();
                assert!(wav.len() > 1000);
                assert!(wav.starts_with(b"RIFF"));
                println!("Successfully synthesized STALKER.MID ({} bytes of WAV)", wav.len());
            }
        }
    }
}
