use serde::{Serialize, Deserialize};
use std::convert::{TryInto};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Particle {
    pub p_type: ParticleType,
    pub fill_ratio: u8,
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            p_type: ParticleType::Empty,
            fill_ratio: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid<T> where T: Clone {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<T>
}

macro_rules! grid_index {
    ($self:ident, $x:expr, $y:expr) => {
        ($x + $y * $self.width) as usize
    };
}

macro_rules! grid_at {
    ($self:ident, $x:expr, $y:expr) => {
        $self.grid[grid_index!($self, $x, $y)]
    };
}

impl<T> Grid<T> where T: Clone + Default {
    pub fn get(&self, x: i32, y: i32) -> &T {
        &grid_at!(self, x, y)
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut T {
        &mut grid_at!(self, x, y)
    }

    pub fn new(width: i32, height: i32) -> Grid<T> {
        Grid::<T> {
            width: width,
            height: height,
            grid: vec![Default::default(); (width * height).try_into().unwrap()]
        }
    }

    pub fn set(&mut self, x: i32, y: i32, p: T) {
        grid_at!(self, x, y) = p;
    }

    pub fn swap(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.grid.swap(grid_index!(self, x1, y1), grid_index!(self, x2, y2));
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, x: i32, y: i32) {
        self.set(x, y, Default::default())
    }

    #[allow(dead_code)]
    pub fn clear_all(&mut self) {
        for i in 0..self.grid.len() {
            self.grid[i] = Default::default()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ParticleType {
    Wood,
    Sand,
    Water,
    Empty
}

pub type ParticleGrid = Grid<Particle>;

impl Grid<Particle> {
    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            self.get(x, y).p_type == ParticleType::Empty
        }
    }
}
