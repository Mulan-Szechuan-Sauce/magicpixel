use sfml::graphics::{RenderWindow, RenderTarget, Color, RectangleShape, Shape, RenderStates, Transformable};
use sfml::window::{VideoMode, Event, Style, Key};
use sfml::system::{Vector2i, Vector2f, Clock};
use std::convert::{TryInto};

mod grid;
use grid::*;

pub struct RenderContext {
    pub scale: f32
}

fn move_sand(grid: &mut Grid, x: i32, y: i32) {
    if grid.is_empty(x, y + 1) {
        let p = grid.get(x, y).clone();
        grid.set(x, y + 1, &p);

        grid.set_type(x, y, ParticleType::Empty);
        grid.reset_velocity(x, y);
    } else if grid.is_empty(x - 1, y + 1) {
        let p = grid.get(x, y).clone();
        grid.set(x - 1, y + 1, &p);

        grid.set_type(x, y, ParticleType::Empty);
        grid.reset_velocity(x, y);
    } else if grid.is_empty(x + 1, y + 1) {
        let p = grid.get(x, y).clone();
        grid.set(x + 1, y + 1, &p);

        grid.set_type(x, y, ParticleType::Empty);
        grid.reset_velocity(x, y);
    }
}

fn isaac_newton(grid: &mut Grid) {
    for y in (0..grid.height).rev() {
        for x in 0..grid.width {
            let particle = grid.get(x, y);

            match particle.p_type {
                ParticleType::Sand => move_sand(grid, x, y),
                _ => {}
            }
        }
    }
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

fn render_grid(window: &RenderWindow, context: &RenderContext, grid: &mut Grid) {
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
    let mut grid = create_simple_grid();

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

    let clock = Clock::start();
    let mut prev_tick = 0;
    let tick_time = 0.1;
    let mut is_paused = false;

    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            match event {
                Event::Closed |
                Event::KeyPressed { code: Key::ESCAPE, .. } =>
                    window.close(),
                Event::KeyPressed { code: Key::P, .. } =>
                    is_paused = !is_paused,
                _ => { /* Do nothing */ }
            }
        }

        // Activate the window for OpenGL rendering
        window.set_active(true);

        window.clear(Color::BLACK);

        // FIXME: Run on a UI thread instead

        let curr_time = clock.elapsed_time().as_seconds();
        let curr_tick = (curr_time / tick_time) as u32;

        if curr_tick > prev_tick {
            while prev_tick < curr_tick {
                if ! is_paused {
                    isaac_newton(&mut grid);
                }
                prev_tick += 1;
            }
        }

        render_grid(&window, &context, &mut grid);

        // End the current frame and display its contents on screen
        window.display();
    }
}
