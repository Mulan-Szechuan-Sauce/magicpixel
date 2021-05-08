use sfml::system::{Vector2i};
use std::convert::{TryInto};

#[derive(Clone, Debug)]
pub struct Particle {
    pub p_type: ParticleType,
    pub velocity: Vector2i,
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

    fn mut_get(&mut self, x: i32, y: i32) -> &mut Particle {
        &mut self.grid[(x + y * self.width) as usize]
    }

    pub fn get(&self, x: i32, y: i32) -> &Particle {
        &self.grid[(x + y * self.width) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, p: Particle) {
        self.grid[(x + y * self.width) as usize] = p;
    }

    pub fn set_type(&mut self, x: i32, y: i32, p_type: ParticleType) {
        self.grid[(x + y * self.width) as usize].p_type = p_type;
    }

    pub fn set_velocity(&mut self, x: i32, y: i32, velocity: Vector2i) {
        self.mut_get(x, y).velocity = velocity;
    }

    pub fn reset_velocity(&mut self, x: i32, y: i32) {
        self.set_velocity(x, y, Vector2i::new(0, 0));
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
        self.set(x, y, Particle {
            p_type: ParticleType::Empty,
            velocity: Vector2i::new(0, 0),
        })
    }

    pub fn clear_all(&mut self) {
        for i in 0..self.grid.len() {
            self.grid[i] = Particle {
                p_type: ParticleType::Empty,
                velocity: Vector2i::new(0, 0),
            }
        }
    }
}
