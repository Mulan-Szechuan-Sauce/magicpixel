use std::mem::swap;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid::{Grid, ParticleType};

pub struct Physics {
    rng: ThreadRng,
    read_grid: Box<Grid>,
    write_grid: Box<Grid>,
}

impl Physics {
    pub fn new(grid: Grid) -> Physics {
        let grid1 = grid.clone();
        let grid2 = grid;

        Physics {
            rng: rand::thread_rng(),
            read_grid: Box::new(grid1),
            write_grid: Box::new(grid2),
        }
    }

    pub fn get_grid(&mut self) -> &mut Box<Grid> {
        &mut self.read_grid
    }

    fn translate(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.read_grid.in_bounds(x2, y2) {
            let p1 = self.read_grid.get(x1, y1);
            self.write_grid.set(x2, y2, p1.clone());

            if x1 != x2 && y1 != y2 {
                self.read_grid.clear(x1, y1);
            }
        }
    }

    fn move_sand(&mut self, x: i32, y: i32) {
        if self.write_grid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1)
        } else if self.read_grid.is_empty(x - 1, y + 1) && self.read_grid.is_empty(x + 1, y + 1) {
            let rand_b: bool = self.rng.gen();
            let dir = if rand_b { -1 } else { 1 };
            self.translate(x, y, x + dir, y + 1)
        } else if self.read_grid.is_empty(x - 1, y + 1) {
            self.translate(x, y, x - 1, y + 1)
        } else if self.read_grid.is_empty(x + 1, y + 1) {
            self.translate(x, y, x + 1, y + 1)
        } else {
            self.translate(x, y, x, y)
        }
    }

    fn move_water(&mut self, x: i32, y: i32) {
    }

    pub fn update(&mut self) {
        self.write_grid.clear_all();

        for y in (0..self.read_grid.height).rev() {
            for x in 0..self.read_grid.width {
                let p_type = &self.read_grid.get(x, y).p_type;

                match p_type {
                    ParticleType::Sand => self.move_sand(x, y),
                    ParticleType::Water => self.move_water(x, y),
                    _ => {}
                }

            }
        }

        swap(&mut self.write_grid, &mut self.read_grid);
    }
}
