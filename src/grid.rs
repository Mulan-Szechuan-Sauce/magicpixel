use sfml::system::{Vector2i};
use std::convert::{TryInto};

#[derive(Clone)]
pub struct Particle {
    pub p_type: ParticleType,
    pub velocity: Vector2i
}

#[derive(Clone, PartialEq)]
pub enum ParticleType {
    Sand,
    Wood,
    Empty
}

#[derive(Clone)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<Particle>
}

pub trait GridExt {
    fn new(width: i32, height: i32) -> Grid;
    fn get(&mut self, x: i32, y: i32) -> &mut Particle;
    fn set(&mut self, x: i32, y: i32, p: &Particle);

    fn set_type(&mut self, x: i32, y: i32, p_type: ParticleType);
    fn set_velocity(&mut self, x: i32, y: i32, velocity: Vector2i);
    fn reset_velocity(&mut self, x: i32, y: i32);
    fn is_empty(&mut self, x: i32, y: i32) -> bool;
}

impl GridExt for Grid {
    fn new(width: i32, height: i32) -> Grid {
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

    fn get(&mut self, x: i32, y: i32) -> &mut Particle {
        &mut self.grid[(x + y * self.width) as usize]
    }

    fn set(&mut self, x: i32, y: i32, p: &Particle) {
        self.grid[(x + y * self.width) as usize] = p.clone();
    }

    fn set_type(&mut self, x: i32, y: i32, p_type: ParticleType) {
        self.grid[(x + y * self.width) as usize].p_type = p_type;
    }

    fn set_velocity(&mut self, x: i32, y: i32, velocity: Vector2i) {
        self.get(x, y).velocity = velocity;
    }

    fn reset_velocity(&mut self, x: i32, y: i32) {
        self.set_velocity(x, y, Vector2i::new(0, 0));
    }

    fn is_empty(&mut self, x: i32, y: i32) -> bool {
        x < 0 || x >= self.width || y < 0 || y >= self.height ||
            self.get(x, y).p_type == ParticleType::Empty
    }
}
