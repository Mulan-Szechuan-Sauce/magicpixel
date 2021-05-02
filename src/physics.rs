use crate::Grid;

pub trait Physics {
    fn update(&mut self, grid: &mut Grid);
}
