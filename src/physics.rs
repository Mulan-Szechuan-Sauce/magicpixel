use std::mem::swap;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid::{Grid, Particle, ParticleType};

pub struct Physics {
    rng: ThreadRng,
    prev_grid: Box<Grid>,
    next_grid: Box<Grid>,
}

impl Physics {
    pub fn new(grid: Grid) -> Physics {
        let grid1 = grid.clone();
        let grid2 = grid;

        Physics {
            rng: rand::thread_rng(),
            prev_grid: Box::new(grid1),
            next_grid: Box::new(grid2),
        }
    }

    pub fn get_grid(&mut self) -> &mut Box<Grid> {
        &mut self.prev_grid
    }

    fn translate(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.prev_grid.in_bounds(x2, y2) {
            let p1 = self.prev_grid.get(x1, y1);
            self.next_grid.set(x2, y2, p1.clone());
        }
    }

    fn move_sand(&mut self, x: i32, y: i32) {
        if self.next_grid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1)
        } else if self.next_grid.is_empty(x - 1, y + 1) && self.next_grid.is_empty(x + 1, y + 1) {
            let rand_b: bool = self.rng.gen();
            let dir = if rand_b { -1 } else { 1 };
            self.translate(x, y, x + dir, y + 1)
        } else if self.next_grid.is_empty(x - 1, y + 1) {
            self.translate(x, y, x - 1, y + 1)
        } else if self.next_grid.is_empty(x + 1, y + 1) {
            self.translate(x, y, x + 1, y + 1)
        } else {
            self.translate(x, y, x, y)
        }
    }
    
    fn move_water(&mut self, x: i32, y: i32) {
        if !self.next_grid.is_empty(x, y) {
            return;
        }

        if self.prev_grid.is_empty(x, y + 1) {
            self.translate(x, y, x, y + 1);
            return;
        }

        if self.try_flow(x, y, x + 1, y) {
            return;
        }

        if self.try_flow(x, y, x - 1, y) {
            return;
        }

        self.translate(x, y, x, y)
    }

    fn try_flow(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if !self.should_fill(x1, y1, x2, y2) {
            return false;
        }

        let src_fill_ratio = self.fill_ratio_at(x1, y1);
        let tgt_fill_ratio = self.fill_ratio_at(x2, y2);

        let net_fill_ratio = src_fill_ratio + tgt_fill_ratio;

        let new_src_fill_ratio = net_fill_ratio / 2 + net_fill_ratio % 2;
        let new_tgt_fill_ratio = net_fill_ratio / 2;

        if src_fill_ratio != new_src_fill_ratio || tgt_fill_ratio != new_tgt_fill_ratio {
            self.next_grid.set(x1, y1, Particle {
                p_type: ParticleType::Water,
                fill_ratio: new_src_fill_ratio,
            });

            if net_fill_ratio / 2 > 0 {
                self.next_grid.set(x2, y2, Particle {
                    p_type: ParticleType::Water,
                    fill_ratio: new_tgt_fill_ratio,
                });
            }
            return true;
        }

        return false;
    }

    fn should_fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if !self.prev_grid.in_bounds(x2, y2) {
            return false;
        }

        let src_tile = self.prev_grid.get(x1, y1);
        let tgt_tile = self.prev_grid.get(x2, y2);

        self.next_grid.get(x2, y2).p_type == ParticleType::Empty &&
            (src_tile.p_type == tgt_tile.p_type
             && src_tile.fill_ratio > tgt_tile.fill_ratio
             || tgt_tile.p_type == ParticleType::Empty)
    }

    fn fill_ratio_at(&mut self, x: i32, y: i32) -> u8 {
        let tile = self.prev_grid.get(x, y);
        if tile.p_type == ParticleType::Empty {
            0
        } else {
            tile.fill_ratio
        }
    }

    pub fn update(&mut self) {
        self.next_grid.clear_all();

        for y in (0..self.prev_grid.height).rev() {
            for x in 0..self.prev_grid.width {
                let p_type = &self.prev_grid.get(x, y).p_type;

                match p_type {
                    ParticleType::Sand => self.move_sand(x, y),
                    ParticleType::Water => self.move_water(x, y),
                    _ => {}
                }

            }
        }

        swap(&mut self.next_grid, &mut self.prev_grid);
    }
}
