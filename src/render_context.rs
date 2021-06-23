use crate::ParticleGrid;
use crate::ParticleType;

pub struct RenderContext {
    pub scale: f32,
    pub win_width: u32,
    pub win_height: u32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub draw_type: ParticleType,
    pub max_fill: u8,
}

impl RenderContext {
    pub fn new(grid: &ParticleGrid, max_fill: u8) -> RenderContext {
        // FIXME: This is waste
        // GetDesktopDisplayMode
        let max_win_width = 2400.0;
        let max_win_height = 1400.0;

        let scale =
            ((max_win_width / grid.width as f32)
             .min(max_win_height / grid.height as f32))
            .floor();

        let win_width = (grid.width as f32 * scale).ceil() as u32;
        let win_height = (grid.height as f32 * scale).ceil() as u32;

        RenderContext {
            scale: scale,
            win_width: win_width,
            win_height: win_height,
            grid_width: grid.width,
            grid_height: grid.height,
            mouse_x: 0,
            mouse_y: 0,
            draw_type: ParticleType::Water,
            max_fill: max_fill,
        }
    }
}
