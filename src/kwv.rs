#![allow(dead_code)]
use std::io::{Read, Cursor};
use hound::{WavSpec, WavWriter, SampleFormat};

#[derive(Debug)]
pub struct KvvWave {
    pub name: String,
    pub data: Vec<u8>,
    pub sample_rate: u32,
}

impl KvvWave {
    pub fn to_wav_bytes(&self) -> Vec<u8> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let mut out = Vec::new();
        {
            let mut writer = WavWriter::new(Cursor::new(&mut out), spec).expect("Failed to create WavWriter");
            for &b in &self.data {
                // Convert 8-bit unsigned (0-255, 128 mid) to 16-bit signed (-32768-32767, 0 mid)
                let sample = (b as i16 - 128) << 8;
                writer.write_sample(sample).expect("Failed to write sample");
            }
            writer.finalize().expect("Failed to finalize WAV");
        }
        
        out
    }
}

pub struct Kwv {
    pub waves: Vec<KvvWave>,
}

fn read_u32(reader: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map(|_| u32::from_le_bytes(buf)).map_err(|e| e.to_string())
}

impl Kwv {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut reader = Cursor::new(data);
        
        let version = read_u32(&mut reader)?;
        if version != 0 {
            return Err(format!("Unsupported KWV version: {}", version));
        }

        let num_waves = read_u32(&mut reader)?;
        let mut headers = Vec::with_capacity(num_waves as usize);

        for _ in 0..num_waves {
            let mut name_buf = [0u8; 16];
            reader.read_exact(&mut name_buf).map_err(|e| e.to_string())?;
            let name = String::from_utf8_lossy(&name_buf)
                .trim_matches('\0')
                .trim()
                .to_string();
            
            let length = read_u32(&mut reader)?;
            let _rep_start = read_u32(&mut reader)?;
            let _rep_length = read_u32(&mut reader)?;
            let _fine_tune = read_u32(&mut reader)?;
            
            headers.push((name, length));
        }

        let mut waves = Vec::with_capacity(num_waves as usize);
        for (name, length) in headers {
            let mut wave_data = vec![0u8; length as usize];
            reader.read_exact(&mut wave_data).map_err(|e| e.to_string())?;
            
            waves.push(KvvWave {
                name,
                data: wave_data,
                sample_rate: 11025,
            });
        }

        Ok(Kwv { waves })
    }
}
