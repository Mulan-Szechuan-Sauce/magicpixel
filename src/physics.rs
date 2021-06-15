use std::cmp::{min};
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid::{Grid, ParticleGrid, Particle, ParticleType, MAX_FILL};

macro_rules! random_eval {
    ($rng:expr, $x:expr, $y:expr) => {
        if $rng.gen() {
            $x
            $y
        } else {
            $y
            $x
        }
    };
}

macro_rules! random_condition {
    ($rng:expr, $x_cond:expr, $x_body:expr, $y_cond:expr, $y_body:expr) => {
        if $rng.gen() {
            if $x_cond {
                $x_body
            } else if $y_cond {
                $y_body
            }
        } else {
            if $y_cond {
                $y_body
            } else if $x_cond {
                $x_body
            }
        }
    }
}

pub struct Physics {
    rng: ThreadRng,
    active_grid: Box<ParticleGrid>,
    change_grid: Box<ParticleGrid>,
    has_changed_grid: Grid<bool>,
}

impl Physics {
    pub fn new(grid: ParticleGrid) -> Physics {
        let bool_grid = Grid::new(grid.width, grid.height);

        let grid1 = grid.clone();
        let grid2 = grid;

        Physics {
            rng: rand::thread_rng(),
            active_grid: Box::new(grid1),
            change_grid: Box::new(grid2),
            has_changed_grid: bool_grid,
        }
    }

    pub fn get_grid(&mut self) -> &mut Box<ParticleGrid> {
        &mut self.active_grid
    }

    fn try_displace_sand(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if !self.active_grid.in_bounds(x2, y2) {
            return false;
        }

        let p_type = self.active_grid.get(x2, y2).p_type;

        if p_type == ParticleType::Water || p_type == ParticleType::Empty {
            // TODO: try_flow_horizontal if water instead of swapping above
            self.change_grid.set(x1, y1, self.active_grid.get(x2, y2).clone());
            self.active_grid.swap(x1, y1, x2, y2);
            true
        } else {
            false
        }
    }

    fn try_move_sand(&mut self, x: i32, y: i32) {
        if self.try_displace_sand(x, y, x, y + 1) {
            self.has_changed_grid.set(x, y, true);
            return;
        }

        let first_dir = if self.rng.gen() { 1 } else { -1 };

        if self.try_displace_sand(x, y, x + first_dir, y + 1) ||
            self.try_displace_sand(x, y, x - first_dir, y + 1) {
            self.has_changed_grid.set(x, y, true);
        }
    }

    // Slurp into the target (BFS from the target)
    fn slurp_into(&mut self, src_left: i32, src_right: i32, src_y: i32, tgt_x: i32, tgt_y: i32) {
        let mut bfs_left = tgt_x - 1;
        let mut bfs_right = tgt_x + 1;

        while bfs_left >= src_left || bfs_right <= src_right {
            let mut slurp_x = -1;

            random_eval!(
                self.rng,
                if slurp_x < 0 && bfs_left >= src_left {
                    if self.active_grid.get(bfs_left, src_y).p_type == ParticleType::Water &&
                        self.active_grid.get(bfs_left, src_y).fill_ratio > 1 {
                        slurp_x = bfs_left;
                    }

                    bfs_left -= 1;
                },
                if slurp_x < 0 && bfs_right <= src_right {
                    if self.active_grid.get(bfs_right, src_y).p_type == ParticleType::Water &&
                        self.active_grid.get(bfs_right, src_y).fill_ratio > 1 {
                        slurp_x = bfs_right;
                    }
                    bfs_right += 1;
                }
            );

            if slurp_x >= 0 {
                self.active_grid.get_mut(slurp_x, src_y).fill_ratio -= 1;
                let mut dirty_mut = self.change_grid.get_mut(tgt_x, tgt_y);

                if dirty_mut.p_type == ParticleType::Water {
                    dirty_mut.fill_ratio += 1;
                } else {
                    dirty_mut.fill_ratio = 1;
                    dirty_mut.p_type = ParticleType::Water;
                }

                self.has_changed_grid.set(tgt_x, tgt_y, true);
                return;
            }
        }
    }

    // Inclusive right-most point of continuous water
    fn find_water_block_end(&self, x: i32, y: i32) -> i32 {
        let mut right_x = x;

        while right_x + 1 < self.active_grid.width &&
              self.active_grid.get(right_x + 1, y).p_type == ParticleType::Water {
            right_x += 1;
        }

        right_x
    }

    fn find_unfilled_in_range(&mut self, left_x: i32, right_x: i32, y: i32) -> Vec<i32> {
        if y < 0 || y >= self.active_grid.height {
            return Vec::new();
        }

        // Preallocate so we don't wast time on vec expansion
        let mut unfilled = Vec::with_capacity((right_x - left_x) as usize);

        for x in left_x..=right_x {
            let particle = self.active_grid.get(x, y);

            if particle.p_type == ParticleType::Empty ||
                particle.p_type == ParticleType::Water && particle.fill_ratio < MAX_FILL {
                unfilled.push(x);
            }
        }

        unfilled
    }

    fn flow_down(&mut self, x: i32, y: i32) {
        let target = self.active_grid.get(x, y + 1).clone();

        if target.p_type == ParticleType::Empty {
            self.active_grid.swap(x, y, x, y + 1);
        } else {
            let source = self.active_grid.get(x, y);

            let net_fr = source.fill_ratio + target.fill_ratio;
            let new_target_fr = min(MAX_FILL, net_fr);
            let new_source_fr = net_fr - new_target_fr;

            if new_source_fr == 0 {
                self.active_grid.clear(x, y);
            } else {
                self.active_grid.get_mut(x, y).fill_ratio = new_source_fr;
            }

            self.active_grid.get_mut(x, y + 1).fill_ratio = new_target_fr;
        }
    }

    fn inner_fill(&mut self, range_left: i32, range_right: i32, x: i32, y: i32) {
        let mut bfs_left = x - 1;
        let mut bfs_right = x + 1;

        let mut max_fr = 0;
        let mut max_x  = -1;

        let base_fr = self.active_grid.get(x, y).fill_ratio;

        while bfs_left >= range_left || bfs_right <= range_right {
            random_eval!(
                self.rng,
                if bfs_left >= range_left {
                    let fr = self.active_grid.get(bfs_left, y).fill_ratio;

                    if fr <= base_fr {
                        bfs_left = range_left - 1;
                    } else {
                        if fr > max_fr {
                            max_fr = fr;
                            max_x = bfs_left;
                        }

                        bfs_left -= 1;
                    }
                },
                if bfs_right <= range_right {
                    let fr = self.active_grid.get(bfs_right, y).fill_ratio;

                    if fr <= base_fr {
                        bfs_right = range_right + 1;
                    } else {
                        if fr > max_fr {
                            max_fr = fr;
                            max_x = bfs_right;
                        }

                        bfs_right += 1;
                    }
                }
            );
        }

        // Yay bfs
        if max_fr > base_fr + 1 {
            self.active_grid.get_mut(max_x, y).fill_ratio -= 1;
            self.active_grid.get_mut(x, y).fill_ratio += 1;
        }
    }

    fn try_move_water(&mut self, x: i32, y: i32) -> i32 {
        let right_x = self.find_water_block_end(x, y);

        let underlings = self.find_unfilled_in_range(x, right_x, y + 1);

        if underlings.len() > 0 {
            for xi in underlings {
                self.flow_down(xi, y);
            }
        } else {
            // Horizontal flow
            random_condition!(
                self.rng,
                self.active_grid.is_empty(x - 1, y),
                self.slurp_into(x, right_x, y, x - 1, y),
                self.active_grid.is_empty(right_x + 1, y),
                self.slurp_into(x, right_x, y, right_x + 1, y)
            );

            for xi in x..=right_x {
                self.inner_fill(x, right_x, xi, y);
            }
        }

        right_x - x
    }

    pub fn update(&mut self) {
        for y in (0..self.active_grid.height).rev() {
            let mut x = 0;

            while x < self.active_grid.width {
                if *self.has_changed_grid.get(x, y) {
                    x += 1;
                    continue;
                }

                let p_type = &self.active_grid.get(x, y).p_type.clone();

                let mut skippy_boi = 1;

                match p_type {
                    ParticleType::Sand  => self.try_move_sand(x, y),
                    ParticleType::Water => {
                        skippy_boi += self.try_move_water(x, y);
                    },
                    ParticleType::Wood => {},
                    ParticleType::Empty => {}
                };

                x += skippy_boi;
            }

            for yp in y..min(self.active_grid.height, y + 2) {
                for x in 0..self.active_grid.width {
                    if *self.has_changed_grid.get(x, yp) {
                        self.active_grid.set(x, yp, self.change_grid.get(x, yp).clone());
                        self.has_changed_grid.set(x, yp, false);

                        self.change_grid.set(x, yp, Default::default());
                    }
                }
            }
        }
    }
}
