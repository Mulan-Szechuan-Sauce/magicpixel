use sfml::system::{Vector2i};
use std::convert::{TryInto};

#[derive(Clone)]
pub struct Particle {
    pub p_type: ParticleType,
    pub velocity: Vector2i
}

#[derive(Clone)]
pub enum ParticleType {
    Sand,
    Wood,
    Empty
}

pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Particle>
}

pub trait GridExt {
    fn new(width: u32, height: u32) -> Grid;
    fn get(&self, x: u32, y: u32) -> &Particle;
    fn set(&mut self, x: u32, y: u32, p: &Particle);
}

impl GridExt for Grid {
    fn new(width: u32, height: u32) -> Grid {
        let empty = Particle {
            p_type: ParticleType::Empty,
            velocity: Vector2i::new(0, 0)
        };

        Grid {
            width: width,
            height: height,
            grid: vec![empty; (width * height).try_into().unwrap()]
        }
    }

    fn get(&self, x: u32, y: u32) -> &Particle {
        &self.grid[(x + y * self.width) as usize]
    }

    fn set(&mut self, x: u32, y: u32, p: &Particle) {
        self.grid[(x + y * self.width) as usize] = p.clone();
    }
}
