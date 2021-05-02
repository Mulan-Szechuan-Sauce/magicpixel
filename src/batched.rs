use sfml::system::Vector2i;

use crate::physics::Physics;
use crate::grid::{Grid, ParticleType};

pub struct BatchedPhysics {
}

impl Physics for BatchedPhysics {
    fn update(&mut self, grid: &mut Grid) {
        for y in (0..grid.height).rev() {
            let mut batched_translations: Vec<TranslateOp> = Vec::new();

            for x in 0..grid.width {
                let particle = grid.get(x, y);

                match particle.p_type {
                    ParticleType::Sand => move_sand(grid, x, y),
                    ParticleType::Water => match move_water(grid, x, y) {
                        Some(val) => {
                            batched_translations.push(val)
                        },
                        None => {}
                    },
                    _ => {}
                }

            }

            for thing in batched_translations {
                if grid.is_empty(thing.x2, thing.y2) {
                    grid.translate(thing.x1, thing.y1, thing.x2, thing.y2);
                }
            }
        }
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

fn move_sand(grid: &mut Grid, x: i32, y: i32) {
    if grid.is_empty(x, y + 1) {
        grid.translate(x, y, x, y + 1)
    } else if grid.is_empty(x - 1, y + 1) {
        grid.translate(x, y, x - 1, y + 1)
    } else if grid.is_empty(x + 1, y + 1) {
        grid.translate(x, y, x + 1, y + 1)
    }
}

fn move_water(grid: &mut Grid, x: i32, y: i32) -> Option<TranslateOp> {
    if grid.is_empty(x, y + 1) {
        Some(TranslateOp::new(x, y, x, y + 1))
    } else if grid.is_empty(x - 1, y + 1) {
        grid.get(x, y).velocity = Vector2i::new(-1, 0);
        Some(TranslateOp::new(x, y, x - 1, y + 1))
    } else if grid.is_empty(x + 1, y + 1) {
        grid.get(x, y).velocity = Vector2i::new(1, 0);
        Some(TranslateOp::new(x, y, x + 1, y + 1))
    } else if grid.is_empty(x + 1, y) && grid.get(x, y).velocity.x > 0 {
        Some(TranslateOp::new(x, y, x + 1, y))
    } else if grid.is_empty(x - 1, y) && grid.get(x, y).velocity.x < 0 {
        Some(TranslateOp::new(x, y, x - 1, y))
    } else {
        None
    }
}
