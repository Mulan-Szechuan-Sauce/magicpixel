use std::cmp::{min};
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid::{Bearing, Grid, ParticleGrid, Particle, ParticleType, MAX_FILL};

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

        if self.try_flow_down(x, y) {
            return true;
        }

        let current_bearing = self.prev_grid.get(x, y).bearing.clone();

        let first_bearing =
            if current_bearing != Bearing::None {
                current_bearing
            } else if self.rng.gen() {
                Bearing::Left
            } else {
                Bearing::Right
            };

        if self.try_flow_horizontal(x, y, first_bearing.clone()) {
            return true;
        }

        if self.try_flow_horizontal(x, y, first_bearing.flip()) {
            return true;
        }

        false
    }

    fn try_flow_down(&mut self, x1: i32, y1: i32) -> bool {
        let x2 = x1;
        let y2 = y1 + 1;

        if !self.should_fill(x1, y1, x2, y2) {
            return false;
        }

        let p_type = self.prev_grid.get(x1, y1).p_type.clone();
        let src_fill_ratio = self.prev_grid.fill_ratio_at(x1, y1);
        let tgt_fill_ratio = self.prev_grid.fill_ratio_at(x2, y2);

        if src_fill_ratio == MAX_FILL && tgt_fill_ratio == MAX_FILL {
            return false;
        }

        let net_fill_ratio = src_fill_ratio + tgt_fill_ratio;

        let new_tgt_fill_ratio = min(net_fill_ratio, MAX_FILL);
        let new_src_fill_ratio = net_fill_ratio - new_tgt_fill_ratio;

        if src_fill_ratio != new_src_fill_ratio || tgt_fill_ratio != new_tgt_fill_ratio {
            let current_bearing = self.prev_grid.get(x1, y1).bearing.clone();

            let new_bearing =
                if current_bearing != Bearing::None {
                    current_bearing
                } else if self.rng.gen() {
                    Bearing::Left
                } else {
                    Bearing::Right
                };

            if new_src_fill_ratio == 0 {
                self.next_grid.set(x1, y1, Default::default());
            } else {
                self.next_grid.set(x1, y1, Particle {
                    p_type: p_type.clone(),
                    fill_ratio: new_src_fill_ratio,
                    bearing: new_bearing.clone(),
                });
            }

            self.next_grid.set(x2, y2, Particle {
                p_type: p_type.clone(),
                fill_ratio: new_tgt_fill_ratio,
                bearing: Bearing::None,
            });

            if new_src_fill_ratio > 0 {
                self.prev_grid.set(x1, y1, Particle {
                    p_type: p_type.clone(),
                    fill_ratio: new_src_fill_ratio,
                    bearing: new_bearing.clone(),
                });

                let _ = self.try_flow_horizontal(x1, y1, new_bearing.clone())
                    || self.try_flow_horizontal(x1, y1, new_bearing.flip());
            }

            self.has_changed_grid.set(x1, y1, true);
            self.has_changed_grid.set(x2, y2, true);

            return true;
        }

        return false;
    }

    fn try_flow_horizontal(&mut self, x1: i32, y1: i32, src_bearing: Bearing) -> bool {
        let x2 = if src_bearing == Bearing::Left { x1 - 1 } else { x1 + 1 };
        let y2 = y1;

        if !self.should_fill(x1, y1, x2, y2) {
            return false;
        }

        let p_type = self.prev_grid.get(x1, y1).p_type.clone();
        let src_fill_ratio = self.prev_grid.fill_ratio_at(x1, y1);
        let tgt_fill_ratio = self.prev_grid.fill_ratio_at(x2, y2);

        let net_fill_ratio = src_fill_ratio + tgt_fill_ratio;

        let new_bearing =
            if tgt_fill_ratio > src_fill_ratio {
                self.prev_grid.get(x2, y2).bearing.clone()
            } else if tgt_fill_ratio < src_fill_ratio {
                src_bearing
            } else if self.rng.gen() {
                Bearing::Right
            } else {
                Bearing::Left
            };

        let mut new_src_fill_ratio: u8 = net_fill_ratio / 2;
        let mut new_tgt_fill_ratio: u8 = new_src_fill_ratio;

        if tgt_fill_ratio == 0 {
            new_tgt_fill_ratio += net_fill_ratio % 2;
        } else if self.rng.gen() {
            new_tgt_fill_ratio += net_fill_ratio % 2;
        } else {
            new_src_fill_ratio += net_fill_ratio % 2;
        }

        if src_fill_ratio != new_src_fill_ratio || tgt_fill_ratio != new_tgt_fill_ratio {
            if new_src_fill_ratio == 0 {
                self.next_grid.set(x1, y1, Default::default());
            } else {
                self.next_grid.set(x1, y1, Particle {
                    p_type: p_type.clone(),
                    fill_ratio: new_src_fill_ratio,
                    bearing: new_bearing.clone(),
                });
            }
            self.has_changed_grid.set(x1, y1, true);

            if new_tgt_fill_ratio == 0 {
                self.next_grid.set(x2, y2, Default::default());
            } else {
                self.next_grid.set(x2, y2, Particle {
                    p_type: p_type,
                    fill_ratio: new_tgt_fill_ratio,
                    bearing: new_bearing,
                });
            }
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

            for yp in y..min(self.prev_grid.height, y + 2) {
                for x in 0..self.prev_grid.width {
                    if *self.has_changed_grid.get(x, yp) {
                        self.prev_grid.set(x, yp, self.next_grid.get(x, yp).clone());
                        self.has_changed_grid.set(x, yp, false);

                        self.next_grid.set(x, yp, Default::default());
                    }
                }
            }
        }
    }
}
