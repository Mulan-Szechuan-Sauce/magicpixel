use sfml::graphics::{Color, Shape, RectangleShape, RenderWindow, RenderTarget, Transformable};
use sfml::system::{Vector2f};

use crate::{RenderContext};
use crate::grid::{ParticleGrid};

pub fn draw_overlay(window: &mut RenderWindow, context: &RenderContext, grid: &ParticleGrid) {
    draw_grid_overlay(window, context, grid);
    draw_mouse_highlight(window, context, grid);
}

fn draw_mouse_highlight(window: &mut RenderWindow, context: &RenderContext, grid: &ParticleGrid) {
    if !grid.in_bounds(context.get_mouse_grid_x(), context.get_mouse_grid_y()) {
        return;
    }

    let x = context.get_mouse_grid_x() as f32;
    let y = context.get_mouse_grid_y() as f32;
    let scale = context.scale;

    let mut line = RectangleShape::new();
    line.set_fill_color(Color::WHITE);

    // For tuning line size of the box
    let thickness = 1.0;
    let offset = 0.0;

    line.set_size(Vector2f::new(thickness, scale + thickness));

    // Left
    line.set_position(Vector2f::new(x * scale - offset, y * scale - offset));
    window.draw(&line);

    // Right
    line.set_position(Vector2f::new((x + 1.0) * scale - offset, y * scale - offset));
    window.draw(&line);

    line.set_size(Vector2f::new(scale + thickness, thickness));

    // Top
    line.set_position(Vector2f::new(x * scale - offset, y * scale - offset));
    window.draw(&line);

    // Bottom
    line.set_position(Vector2f::new(x * scale - offset, (y + 1.0) * scale - offset));
    window.draw(&line);
}

fn draw_grid_overlay(window: &mut RenderWindow, context: &RenderContext, grid: &ParticleGrid) {
    let mut line = RectangleShape::new();
    line.set_fill_color(Color::rgb(70, 70, 70));

    line.set_size(Vector2f::new(1.0, context.win_height as f32));

    for x in 1..grid.width {
        line.set_position(Vector2f::new(x as f32 * context.scale, 0.0));
        window.draw(&line);
    }

    line.set_size(Vector2f::new(context.win_width as f32, 1.0));

    for y in 1..grid.height {
        line.set_position(Vector2f::new(0.0, y as f32 * context.scale));
        window.draw(&line);
    }
}
