use sfml::system::{Vector2i};
use std::convert::{TryInto};

#[derive(Clone)]
pub struct Particle {
    pub p_type: ParticleType,
    pub velocity: Vector2i,
    pub pressure: Vector2i
}

#[derive(Clone, PartialEq)]
pub enum ParticleType {
    Sand,
    Water,
    Wood,
    Empty
}

#[derive(Clone)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<Particle>
}

impl Grid {
    pub fn new(width: i32, height: i32) -> Grid {
        let empty = Particle {
            p_type: ParticleType::Empty,
            velocity: Vector2i::new(0, 0),
            pressure: Vector2i::new(0, 0),
        };

        Grid {
            width: width,
            height: height,
            grid: vec![empty; (width * height).try_into().unwrap()]
        }
    }

    pub fn get(&mut self, x: i32, y: i32) -> &mut Particle {
        &mut self.grid[(x + y * self.width) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, p: &Particle) {
        self.grid[(x + y * self.width) as usize] = p.clone();
    }

    pub fn set_type(&mut self, x: i32, y: i32, p_type: ParticleType) {
        self.grid[(x + y * self.width) as usize].p_type = p_type;
    }

    pub fn set_velocity(&mut self, x: i32, y: i32, velocity: Vector2i) {
        self.get(x, y).velocity = velocity;
    }

    pub fn reset_velocity(&mut self, x: i32, y: i32) {
        self.set_velocity(x, y, Vector2i::new(0, 0));
    }

    pub fn is_empty(&mut self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            self.get(x, y).p_type == ParticleType::Empty
        }
    }

    pub fn translate(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if x2 >= 0 && x2 < self.width && y2 >= 0 && y2 < self.height {
            let p1 = self.get(x1, y1);
            self.grid[(x2 + y2 * self.width) as usize] = p1.clone();
        }

        self.set_type(x1, y1, ParticleType::Empty);
        self.reset_velocity(x1, y1);
    }
}
