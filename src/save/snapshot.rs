#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::save::format::*;

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
    let bytes = snapshot.to_bytes();
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    Ok(())
}

pub fn read_save_from_disk(path: &Path) -> Result<SaveGameSnapshot, &'static str> {
    let mut file = File::open(path).map_err(|_| "Failed to open save file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|_| "Failed to read save file")?;
    SaveGameSnapshot::from_bytes(&buffer)
}
