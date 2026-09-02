#![allow(dead_code)]

use std::io::Cursor;
use hound::{WavSpec, WavWriter, SampleFormat};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VocSound {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub pcm_samples_16bit: Vec<i16>,
}

impl VocSound {
    /// Parse a Creative Voice (.VOC) file from byte data
    pub fn parse(name: &str, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 26 {
            return Err("VOC file too short (less than 26 bytes header)".to_string());
        }

        // Check header signature
        if !bytes.starts_with(b"Creative Voice File\x1A") {
            return Err("Invalid VOC magic signature".to_string());
        }

        let header_size = u16::from_le_bytes([bytes[20], bytes[21]]) as usize;
        if header_size < 26 || header_size > bytes.len() {
            return Err(format!("Invalid VOC header size: {}", header_size));
        }

        let mut offset = header_size;
        let mut sample_rate = 11025;
        let mut extended_rate_set = false;
        let mut channels = 1;
        let mut bits_per_sample = 16;
        let mut samples_16: Vec<i16> = Vec::new();

        while offset < bytes.len() {
            let block_type = bytes[offset];
            offset += 1;

            if block_type == 0 {
                // Terminator block
                break;
            }

            if offset + 3 > bytes.len() {
                break;
            }

            let block_len = (bytes[offset] as usize)
                | ((bytes[offset + 1] as usize) << 8)
                | ((bytes[offset + 2] as usize) << 16);
            offset += 3;

            if offset + block_len > bytes.len() {
                // Block extends past end of file; clamp to available bytes
                let available = bytes.len().saturating_sub(offset);
                if available == 0 {
                    break;
                }
            }

            let block_data = &bytes[offset..std::cmp::min(offset + block_len, bytes.len())];
            offset += block_len;

            match block_type {
                1 => {
                    // Standard Sound Data:
                    // 1 byte: time constant
                    // 1 byte: pack type (0 = 8-bit unsigned PCM)
                    // (block_len - 2) bytes: 8-bit PCM data (128 = silence)
                    if block_data.len() >= 2 {
                        let tc = block_data[0] as u32;
                        let pack_type = block_data[1];
                        if !extended_rate_set && tc < 256 {
                            let divisor = 256 - tc;
                            if divisor > 0 {
                                sample_rate = 1_000_000 / divisor;
                            }
                        }
                        extended_rate_set = false; // Reset for subsequent blocks
                        
                        let raw_samples = &block_data[2..];
                        match pack_type {
                            0 => {
                                // 8-bit unsigned PCM
                                for &b in raw_samples {
                                    let sample = (b as i16 - 128) << 8;
                                    samples_16.push(sample);
                                }
                            }
                            _ => {
                                // Fallback for unsupported compression: convert as 8-bit PCM
                                for &b in raw_samples {
                                    let sample = (b as i16 - 128) << 8;
                                    samples_16.push(sample);
                                }
                            }
                        }
                    }
                }
                2 => {
                    // Sound Continuation
                    for &b in block_data {
                        let sample = (b as i16 - 128) << 8;
                        samples_16.push(sample);
                    }
                }
                3 => {
                    // Silence
                    if block_data.len() >= 3 {
                        let count = u16::from_le_bytes([block_data[0], block_data[1]]) as usize;
                        let tc = block_data[2] as u32;
                        if !extended_rate_set && tc < 256 {
                            let divisor = 256 - tc;
                            if divisor > 0 {
                                sample_rate = 1_000_000 / divisor;
                            }
                        }
                        samples_16.resize(samples_16.len() + count, 0);
                    }
                }
                8 => {
                    // Extended Sound Data parameter (16-bit time constant)
                    if block_data.len() >= 4 {
                        let tc = u16::from_le_bytes([block_data[0], block_data[1]]) as u32;
                        let mode = block_data[3];
                        if tc < 65536 {
                            let div = 65536 - tc;
                            if div > 0 {
                                if mode == 1 {
                                    channels = 2;
                                    sample_rate = (256_000_000 / div) / 2;
                                } else {
                                    channels = 1;
                                    sample_rate = 256_000_000 / div;
                                }
                                extended_rate_set = true;
                            }
                        }
                    }
                }
                9 => {
                    // Block 9: Extended format (sample rate, bits, channels, format tag)
                    if block_data.len() >= 12 {
                        let rate = u32::from_le_bytes([
                            block_data[0],
                            block_data[1],
                            block_data[2],
                            block_data[3],
                        ]);
                        if rate > 0 && rate < 192000 {
                            sample_rate = rate;
                            extended_rate_set = true;
                        }
                        let bits = block_data[4];
                        let ch = block_data[5] as u16;
                        if ch > 0 { channels = ch; }
                        let format_tag = u16::from_le_bytes([block_data[6], block_data[7]]);
                        
                        let audio_bytes = &block_data[12..];
                        if bits == 16 || format_tag == 4 {
                            bits_per_sample = 16;
                            for chunk in audio_bytes.chunks_exact(2) {
                                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                                samples_16.push(sample);
                            }
                        } else {
                            bits_per_sample = 16; // We convert to 16-bit
                            for &b in audio_bytes {
                                let sample = (b as i16 - 128) << 8;
                                samples_16.push(sample);
                            }
                        }
                    }
                }
                _ => {
                    // Ignore markers, text, loop blocks
                }
            }
        }

        // Clamp sample rate to sensible audio range (4000 Hz to 96000 Hz)
        if sample_rate < 4000 || sample_rate > 96000 {
            sample_rate = 11025;
        }

        Ok(Self {
            name: name.to_string(),
            sample_rate,
            channels,
            bits_per_sample,
            pcm_samples_16bit: samples_16,
        })
    }

    /// Converts decoded 16-bit PCM samples to standard RIFF WAV byte format for Bevy Audio
    pub fn to_wav_bytes(&self) -> Vec<u8> {
        let spec = WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut out = Vec::new();
        {
            let mut writer = WavWriter::new(Cursor::new(&mut out), spec)
                .expect("Failed to initialize WavWriter for VOC to WAV");
            for &sample in &self.pcm_samples_16bit {
                writer.write_sample(sample).expect("Failed to write PCM sample");
            }
            writer.finalize().expect("Failed to finalize WAV output");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_voc_parsing() {
        let mut voc_bytes = Vec::new();
        // 20-byte magic
        voc_bytes.extend_from_slice(b"Creative Voice File\x1A");
        // header size = 26
        voc_bytes.extend_from_slice(&26u16.to_le_bytes());
        // version 1.10 = 0x010A
        voc_bytes.extend_from_slice(&0x010Au16.to_le_bytes());
        // 2's complement check: (~0x010A + 0x1234) & 0xFFFF
        let check = (!0x010Au16).wrapping_add(0x1234);
        voc_bytes.extend_from_slice(&check.to_le_bytes());

        // Block 1 (Sound data): tc = 165 (approx 11025Hz), pack_type = 0, 4 sample bytes
        voc_bytes.push(1); // type 1
        let block_len: u32 = 2 + 4;
        voc_bytes.push((block_len & 0xFF) as u8);
        voc_bytes.push(((block_len >> 8) & 0xFF) as u8);
        voc_bytes.push(((block_len >> 16) & 0xFF) as u8);
        voc_bytes.push(165); // tc
        voc_bytes.push(0);   // 8-bit unsigned PCM
        voc_bytes.extend_from_slice(&[128, 200, 50, 128]); // 4 samples

        // Block 0 (Terminator)
        voc_bytes.push(0);

        let sound = VocSound::parse("TEST", &voc_bytes).expect("Should parse mock VOC");
        assert_eq!(sound.name, "TEST");
        assert_eq!(sound.pcm_samples_16bit.len(), 4);
        assert_eq!(sound.pcm_samples_16bit[0], 0); // 128 - 128 = 0
        assert_eq!(sound.pcm_samples_16bit[1], (200 - 128) << 8);

        let wav = sound.to_wav_bytes();
        assert!(wav.starts_with(b"RIFF"));
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn test_parse_real_grp_voc_files() {
        if let Ok(grp) = crate::grp::Grp::open("dukenukem3d/duke3d.grp") {
            let mut parsed_count = 0;
            for entry in &grp.entries {
                if entry.name.to_uppercase().ends_with(".VOC") {
                    if let Ok(voc_data) = grp.read_file(&entry.name) {
                        let res = VocSound::parse(&entry.name, &voc_data);
                        assert!(res.is_ok(), "Failed to parse VOC {}: {:?}", entry.name, res.err());
                        let sound = res.unwrap();
                        assert!(sound.sample_rate > 0);
                        let wav = sound.to_wav_bytes();
                        assert!(wav.len() > 44);
                        parsed_count += 1;
                    }
                }
            }
            println!("Successfully parsed {} VOC files from duke3d.grp into WAV!", parsed_count);
            assert!(parsed_count > 50, "Expected at least 50 VOC files in duke3d.grp");
        }
    }
}
