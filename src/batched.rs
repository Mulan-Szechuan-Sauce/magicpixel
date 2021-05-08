use sfml::system::Vector2i;

use std::mem::swap;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::physics::Physics;
use crate::grid::{Grid, ParticleType};

pub struct BatchedPhysics {
    rng: ThreadRng,
    readGrid: Box<Grid>,
    writeGrid: Box<Grid>,
}

impl BatchedPhysics {
    pub fn new(grid: Grid) -> BatchedPhysics {
        let grid1 = grid.clone();
        let grid2 = grid;

        BatchedPhysics {
            rng: rand::thread_rng(),
            readGrid: Box::new(grid1),
            writeGrid: Box::new(grid2),
        }
    }

    pub fn get_grid(&mut self) -> &mut Box<Grid> {
        &mut self.readGrid
    }

    fn translate(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.readGrid.in_bounds(x2, y2) {
            let p1 = self.readGrid.get(x1, y1);
            self.writeGrid.set(x2, y2, p1.clone());

            if x1 != x2 && y1 != y2 {
                self.readGrid.clear(x1, y1);
            }
        }
    }

    fn move_sand(&mut self, x: i32, y: i32) {
        if self.writeGrid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1)
        } else if self.readGrid.is_empty(x - 1, y + 1) && self.readGrid.is_empty(x + 1, y + 1) {
            let rand_b: bool = self.rng.gen();
            let dir = if rand_b { -1 } else { 1 };
            self.translate(x, y, x + dir, y + 1)
        } else if self.readGrid.is_empty(x - 1, y + 1) {
            self.translate(x, y, x - 1, y + 1)
        } else if self.readGrid.is_empty(x + 1, y + 1) {
            self.translate(x, y, x + 1, y + 1)
        } else {
            self.translate(x, y, x, y)
        }
    }

    fn move_water(&mut self, grid: &mut Grid, x: i32, y: i32) -> Option<TranslateOp> {
        if grid.is_empty(x, y + 1) {
            Some(TranslateOp::new(x, y, x, y + 1))
        } else if grid.is_empty(x - 1, y + 1) {
            grid.set_velocity(x, y, Vector2i::new(-1, 0));
            Some(TranslateOp::new(x, y, x - 1, y + 1))
        } else if grid.is_empty(x + 1, y + 1) {
            grid.set_velocity(x, y, Vector2i::new(1, 0));
            Some(TranslateOp::new(x, y, x + 1, y + 1))
        } else if grid.is_empty(x + 1, y) && grid.get(x, y).velocity.x > 0 {
            Some(TranslateOp::new(x, y, x + 1, y))
        } else if grid.is_empty(x - 1, y) && grid.get(x, y).velocity.x < 0 {
            Some(TranslateOp::new(x, y, x - 1, y))
        } else {
            None
        }
    }
}

impl Physics for BatchedPhysics {
    fn update(&mut self) {
        self.writeGrid.clear_all();

        for y in (0..self.readGrid.height).rev() {
            let mut batched_translations: Vec<TranslateOp> = Vec::new();

            for x in 0..self.readGrid.width {
                let p_type = &self.readGrid.get(x, y).p_type;

                match p_type {
                    ParticleType::Sand => self.move_sand(x, y),
                    // ParticleType::Water => match self.move_water(grid, x, y) {
                    //     Some(val) => {
                    //         batched_translations.push(val)
                    //     },
                    //     None => {}
                    // },
                    _ => {}
                }

            }
        }

        swap(&mut self.writeGrid, &mut self.readGrid);
    }
}

struct TranslateOp {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl TranslateOp {
    fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> TranslateOp {
        TranslateOp {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }
}
