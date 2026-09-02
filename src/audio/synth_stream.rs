use bevy::prelude::*;
use bevy::audio::Decodable;
use bevy::reflect::TypePath;
use rodio::Source;
use rustysynth::{Synthesizer, SynthesizerSettings, MidiFile, MidiFileSequencer, SoundFont};
use std::sync::Arc;
use std::io::Cursor;

#[derive(TypePath, Asset, Clone)]
pub struct MidiAudioStream {
    pub midi_data: Arc<Vec<u8>>,
    pub sf2_soundfont: Arc<SoundFont>,
}

impl Decodable for MidiAudioStream {
    type Decoder = MidiAudioDecoder;
    type DecoderItem = f32; // rodio expects f32 or i16 samples

    fn decoder(&self) -> Self::Decoder {
        let sample_rate = 44100;
        let mut settings = SynthesizerSettings::new(sample_rate as i32);
        settings.maximum_polyphony = 64;
        
        let sequencer = match Synthesizer::new(&self.sf2_soundfont, &settings) {
            Ok(synthesizer) => {
                let mut seq = MidiFileSequencer::new(synthesizer);
                if let Ok(midi_file) = MidiFile::new(&mut Cursor::new(self.midi_data.as_ref())) {
                    seq.play(&Arc::new(midi_file), true);
                }
                Some(seq)
            },
            Err(_) => None,
        };

        MidiAudioDecoder {
            sequencer,
            left_buffer: vec![0.0; 256],
            right_buffer: vec![0.0; 256],
            buffer_index: 256, // force initial render
            sample_rate,
            channel_toggle: false,
        }
    }
}

pub struct MidiAudioDecoder {
    sequencer: Option<MidiFileSequencer>,
    left_buffer: Vec<f32>,
    right_buffer: Vec<f32>,
    buffer_index: usize,
    sample_rate: u32,
    channel_toggle: bool,
}

impl Iterator for MidiAudioDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_index >= self.left_buffer.len() {
            if let Some(ref mut seq) = self.sequencer {
                seq.render(&mut self.left_buffer, &mut self.right_buffer);
            } else {
                // Yield silence if sequencer failed to initialize
                self.left_buffer.fill(0.0);
                self.right_buffer.fill(0.0);
            }
            self.buffer_index = 0;
            self.channel_toggle = false; // Left first
        }

        let sample = if !self.channel_toggle {
            self.channel_toggle = true;
            self.left_buffer[self.buffer_index]
        } else {
            self.channel_toggle = false;
            let s = self.right_buffer[self.buffer_index];
            self.buffer_index += 1;
            s
        };

        Some(sample) // Stream runs forever
    }
}

impl Source for MidiAudioDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2 // Stereo
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
