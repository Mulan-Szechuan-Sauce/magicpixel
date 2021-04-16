use sfml::graphics::{RenderWindow, RenderTarget, Color, RectangleShape, Shape, RenderStates, Transformable};
use sfml::window::{VideoMode, Event, Style, Key};
use sfml::system::{Vector2i, Vector2f};
use std::convert::{TryInto};

mod grid;
use grid::*;

pub struct RenderContext {
    pub scale: f32
}

fn create_simple_grid() -> Grid {
    let mut grid = Grid::new(200, 100);

    for x in 0..grid.width {
        grid.set(x, grid.height - 1, & Particle {
            p_type: ParticleType::Wood,
            velocity: Vector2i::new(0, 0)
        });
    }

    for y in 10..(grid.height - 10) {
        grid.set(grid.width / 2, y, & Particle {
            p_type: ParticleType::Sand,
            velocity: Vector2i::new(0, 0)
        });
    }

    grid
}

fn render_grid(window: &RenderWindow, context: &RenderContext, grid: &Grid) {
    let scale = context.scale;

    for x in 0..grid.width {
        for y in 0..grid.height {
            let p = grid.get(x, y);

            let mut rect = RectangleShape::with_size(Vector2f::new(scale, scale));
            rect.set_position(Vector2f::new(x as f32 * scale, y as f32 * scale));

            match p.p_type {
                ParticleType::Empty =>
                    rect.set_fill_color(Color::BLACK),
                ParticleType::Wood =>
                    rect.set_fill_color(Color::rgb(139, 69, 19)),
                ParticleType::Sand =>
                    rect.set_fill_color(Color::rgb(194, 178, 128)),
            }

            window.draw_rectangle_shape(&rect, &RenderStates::default());
        }
    }
}

fn main() {
    let grid = create_simple_grid();

    let desktop = VideoMode::desktop_mode();

    let context = RenderContext {
        scale: 12.0
    };

    let win_width = (grid.width as f32 * context.scale).ceil() as u32;
    let win_height = (grid.height as f32 * context.scale).ceil() as u32;

    let mut window = RenderWindow::new(
        VideoMode::new(win_width, win_height, desktop.bits_per_pixel),
        "Ok zoomer",
        Style::CLOSE,
        &Default::default());

    window.set_position(Vector2i::new(
        ((desktop.width - win_width) / 2).try_into().unwrap(),
        ((desktop.height - win_height) / 2).try_into().unwrap()
    ));

    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            match event {
                Event::Closed |
                Event::KeyPressed { code: Key::ESCAPE, .. } =>
                    window.close(),
                _ => { /* Do nothing */ }
            }
        }

        // Activate the window for OpenGL rendering
        window.set_active(true);

        window.clear(Color::BLACK);
        // OpenGL drawing commands go here...

        render_grid(&window, &context, &grid);

        // End the current frame and display its contents on screen
        window.display();
    }
}
