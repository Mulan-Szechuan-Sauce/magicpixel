use crate::ParticleGrid;

use serde::{Serialize, Deserialize};
use std::io::prelude::*;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub max_fill: u8,
    pub grid: ParticleGrid,
}

impl SaveState {
    pub fn load(path: String) -> SaveState {
        let mut f = File::open(path).expect("File not found");

        // Ignore version for now
        let mut version: [u8; 1] = [0; 1];
        let _ = f.read(&mut version).expect("File is empty");

        let mut buff_bois: Vec<u8> = Vec::new();
        let _ = f.read_to_end(&mut buff_bois).unwrap();

        bincode::deserialize(&buff_bois).expect("Corrupted file")
    }

    pub fn save(&self, path: String) {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();

        let mut buffer = File::create(path).expect("Could not save to path");
        // Version byte
        let _ = buffer.write(&[1u8]);
        let _ = buffer.write_all(encoded.as_ref());
    }
}
