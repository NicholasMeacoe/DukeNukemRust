use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub struct GrpEntry {
    pub name: String,
    pub size: u32,
    pub offset: u64,
}

pub struct Grp {
    pub entries: Vec<GrpEntry>,
    file_path: String,
}

impl Grp {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut file = File::open(&path).map_err(|e| e.to_string())?;
        let mut header = [0u8; 12];
        file.read_exact(&mut header).map_err(|e| e.to_string())?;
        
        if &header != b"KenSilverman" {
            return Err("Invalid GRP header".to_string());
        }
        
        let mut num_files_buf = [0u8; 4];
        file.read_exact(&mut num_files_buf).map_err(|e| e.to_string())?;
        let num_files = u32::from_le_bytes(num_files_buf);
        
        let mut entries = Vec::with_capacity(num_files as usize);
        let mut current_offset = 16 + (num_files as u64 * 16);
        
        for _ in 0..num_files {
            let mut name_buf = [0u8; 12];
            file.read_exact(&mut name_buf).map_err(|e| e.to_string())?;
            let name = String::from_utf8_lossy(&name_buf)
                .trim_matches('\0')
                .trim()
                .to_string();
            
            let mut size_buf = [0u8; 4];
            file.read_exact(&mut size_buf).map_err(|e| e.to_string())?;
            let size = u32::from_le_bytes(size_buf);
            
            entries.push(GrpEntry {
                name,
                size,
                offset: current_offset,
            });
            
            current_offset += size as u64;
        }
        
        Ok(Grp {
            entries,
            file_path: path.as_ref().to_string_lossy().to_string(),
        })
    }

    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, String> {
        let entry = self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("File not found in GRP: {}", name))?;
        
        let mut file = File::open(&self.file_path).map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(entry.offset)).map_err(|e| e.to_string())?;
        
        let mut data = vec![0u8; entry.size as usize];
        file.read_exact(&mut data).map_err(|e| e.to_string())?;
        
        Ok(data)
    }
}
