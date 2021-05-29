use std::mem::swap;
use std::cmp::min;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid::{Grid, ParticleGrid, Particle, ParticleType, MAX_FILL};

pub struct Physics {
    rng: ThreadRng,
    prev_grid: Box<ParticleGrid>,
    next_grid: Box<ParticleGrid>,
    has_changed_grid: Grid<bool>,
}

impl Physics {
    pub fn new(grid: ParticleGrid) -> Physics {
        let bool_grid = Grid::new(grid.width, grid.height);

        let grid1 = grid.clone();
        let grid2 = grid;

        Physics {
            rng: rand::thread_rng(),
            prev_grid: Box::new(grid1),
            next_grid: Box::new(grid2),
            has_changed_grid: bool_grid,
        }
    }

    pub fn get_grid(&mut self) -> &mut Box<ParticleGrid> {
        &mut self.prev_grid
    }

    fn translate(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.prev_grid.in_bounds(x2, y2) {
            let p1 = self.prev_grid.get(x1, y1);
            self.next_grid.set(x2, y2, p1.clone());
        }
    }

    fn try_move_sand(&mut self, x: i32, y: i32) -> bool {
        if self.next_grid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1);
            true
        } else if self.next_grid.is_empty(x - 1, y + 1) && self.next_grid.is_empty(x + 1, y + 1) {
            let rand_b: bool = self.rng.gen();
            let dir = if rand_b { -1 } else { 1 };
            self.translate(x, y, x + dir, y + 1);
            true
        } else if self.next_grid.is_empty(x - 1, y + 1) {
            self.translate(x, y, x - 1, y + 1);
            true
        } else if self.next_grid.is_empty(x + 1, y + 1) {
            self.translate(x, y, x + 1, y + 1);
            true
        } else {
            false
        }
    }
    
    fn try_move_water(&mut self, x: i32, y: i32) -> bool {
        if *self.has_changed_grid.get(x, y) {
            return true;
        }

        if self.prev_grid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1);
            self.has_changed_grid.set(x, y + 1, true);
            return true;
        }

        if self.try_flow(x, y, x, y + 1) {
            return true;
        }

        if self.try_flow(x, y, x + 1, y) {
            return true;
        }

        if self.try_flow(x, y, x - 1, y) {
            return true;
        }

        false
    }

    fn try_flow(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if !self.should_fill(x1, y1, x2, y2) {
            return false;
        }

        let p_type = self.prev_grid.get(x1, y1).p_type.clone();
        let src_fill_ratio = self.prev_grid.fill_ratio_at(x1, y1);
        let tgt_fill_ratio = self.prev_grid.fill_ratio_at(x2, y2);

        let net_fill_ratio = src_fill_ratio + tgt_fill_ratio;

        let mut new_src_fill_ratio: u8;
        let mut new_tgt_fill_ratio: u8;

        if y2 > y1 {
            new_tgt_fill_ratio = min(net_fill_ratio, MAX_FILL);
            new_src_fill_ratio = net_fill_ratio - new_tgt_fill_ratio;
        } else {
            new_src_fill_ratio = net_fill_ratio / 2;
            new_tgt_fill_ratio = new_src_fill_ratio;

            if self.rng.gen() {
                new_src_fill_ratio += net_fill_ratio % 2;
            } else {
                new_tgt_fill_ratio += net_fill_ratio % 2;
            }
        }

        if src_fill_ratio != new_src_fill_ratio || tgt_fill_ratio != new_tgt_fill_ratio {
            self.next_grid.set(x1, y1, Particle {
                p_type: p_type.clone(),
                fill_ratio: new_src_fill_ratio,
            });
            self.has_changed_grid.set(x1, y1, true);

            self.next_grid.set(x2, y2, Particle {
                p_type: p_type,
                fill_ratio: new_tgt_fill_ratio,
            });
            self.has_changed_grid.set(x2, y2, true);

            return true;
        }

        return false;
    }

    fn should_fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if !self.prev_grid.in_bounds(x2, y2) {
            return false;
        }

        if *self.has_changed_grid.get(x2, y2) {
            return false;
        }

        let src_tile = self.prev_grid.get(x1, y1);
        let tgt_tile = self.prev_grid.get(x2, y2);

        if tgt_tile.p_type == ParticleType::Empty {
            return true;
        }

        if src_tile.p_type != tgt_tile.p_type {
            return false;
        }

        // Gravity case
        if y2 > y1 {
            true
        } else {
            src_tile.fill_ratio > tgt_tile.fill_ratio
        }
    }

    pub fn update(&mut self) {
        self.next_grid.clear_all();
        self.has_changed_grid.clear_all();

        for y in (0..self.prev_grid.height).rev() {
            for x in 0..self.prev_grid.width {
                let p_type = &self.prev_grid.get(x, y).p_type.clone();

                let updated = match p_type {
                    ParticleType::Sand  => self.try_move_sand(x, y),
                    ParticleType::Water => self.try_move_water(x, y),
                    ParticleType::Empty => false
                };

                if *p_type != ParticleType::Empty {
                    self.has_changed_grid.set(x, y, updated);
                }
            }
        }

        for y in (0..self.prev_grid.height).rev() {
            for x in 0..self.prev_grid.width {
                if !*self.has_changed_grid.get(x, y) {
                    self.translate(x, y, x, y);
                }
            }
        }

        swap(&mut self.next_grid, &mut self.prev_grid);
    }
}
