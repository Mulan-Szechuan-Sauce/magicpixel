use std::convert::{TryInto};

use fraction::Fraction;

#[derive(Clone, Debug)]
pub struct Particle {
    pub p_type: ParticleType,
    pub fill_ratio: Fraction,
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            p_type: ParticleType::Empty,
            fill_ratio: Fraction::new(1u8, 1u8),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ParticleType {
    Sand,
    Water,
    Wood,
    Empty
}

#[derive(Clone, Debug)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<Particle>
}

impl Grid {
    pub fn new(width: i32, height: i32) -> Grid {
        Grid {
            width: width,
            height: height,
            grid: vec![Default::default(); (width * height).try_into().unwrap()]
        }
    }

    pub fn get(&self, x: i32, y: i32) -> &Particle {
        &self.grid[(x + y * self.width) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, p: Particle) {
        self.grid[(x + y * self.width) as usize] = p;
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            self.get(x, y).p_type == ParticleType::Empty
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn clear(&mut self, x: i32, y: i32) {
        self.set(x, y, Default::default())
    }

    pub fn clear_all(&mut self) {
        for i in 0..self.grid.len() {
            self.grid[i] = Default::default()
        }
    }
}
