#![allow(dead_code)]

use crate::save::format::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn get_save_dir() -> PathBuf {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

pub fn get_save_path_for_slot(slot: usize) -> PathBuf {
    get_save_dir().join(format!("game_{}.sav", slot))
}

pub fn get_quicksave_path() -> PathBuf {
    get_save_dir().join("quicksave.sav")
}

pub fn write_save_to_disk(path: &Path, snapshot: &SaveGameSnapshot) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = bincode::serialize(snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    Ok(())
}

pub fn read_save_from_disk(path: &Path) -> Result<SaveGameSnapshot, &'static str> {
    let mut file = File::open(path).map_err(|_| "Failed to open save file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|_| "Failed to read save file")?;
    bincode::deserialize(&buffer).map_err(|_| "Failed to deserialize save file")
}
