use sfml::graphics::{RenderWindow, RenderTarget, Color, RectangleShape, Shape, RenderStates, Transformable, Text, Font};
use sfml::window::{VideoMode, Event, Style, Key};
use sfml::window::mouse::{Button};
use sfml::system::{Vector2i, Vector2f, Clock};
use std::convert::{TryInto};
use core::ops::Deref;

mod grid;
use grid::*;

mod fps;
use fps::*;

pub struct RenderContext {
    pub scale: f32,
    pub water_rect: RectangleShape<'static>,
    pub sand_rect: RectangleShape<'static>,
    pub wood_rect: RectangleShape<'static>
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

#[derive(Debug)]
struct TranslateOp {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

trait TranslateOpExt {
    fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> TranslateOp;
}

impl TranslateOpExt for TranslateOp {
    fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> TranslateOp {
        TranslateOp {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
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

fn isaac_newton(grid: &mut Grid) {
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

fn create_simple_grid() -> Grid {
    let mut grid = Grid::new(200, 100);

    for y in 10..(grid.height - 10) {
        grid.set(grid.width / 2, y, & Particle {
            p_type: ParticleType::Sand,
            velocity: Vector2i::new(0, 0),
            pressure: Vector2i::new(0, 0)
        });
    }

    for y in 0..(grid.height - 10) {
        for x in 0..10 {
            grid.set(x, y, & Particle {
                p_type: ParticleType::Water,
                velocity: Vector2i::new(0, 0),
                pressure: Vector2i::new(0, 0)
            });
        }
    }

    grid
}

fn render_particle(window: &RenderWindow, scale: f32, rect: &mut RectangleShape, x: i32, y: i32) {
    rect.set_position(Vector2f::new(x as f32 * scale, y as f32 * scale));
    window.draw_rectangle_shape(&rect, &RenderStates::default());
}

fn render_grid(window: &RenderWindow, context: &mut RenderContext, grid: &mut Grid) {
    for x in 0..grid.width {
        for y in 0..grid.height {
            let p = grid.get(x, y);

            match p.p_type {
                ParticleType::Empty => {}
                ParticleType::Wood => {
                    render_particle(window, context.scale, &mut context.wood_rect, x, y);
                },
                ParticleType::Water => {
                    render_particle(window, context.scale, &mut context.water_rect, x, y);
                },
                ParticleType::Sand => {
                    render_particle(window, context.scale, &mut context.sand_rect, x, y);
                }
            }
        }
    }
}

fn insert_particle(
    grid: &mut Grid,
    context: &RenderContext,
    mouse_x: i32,
    mouse_y: i32,
    p_type: &ParticleType
) {
    let x = (mouse_x as f32 / context.scale) as i32;
    let y = (mouse_y as f32 / context.scale) as i32;

    grid.set(x, y, & Particle {
        p_type: p_type.clone(),
        velocity: Vector2i::new(0, 0),
        pressure: Vector2i::new(0, 0)
    });
}
 
fn new_particle_shape(color: Color, scale: f32) -> RectangleShape<'static> {
    let mut rect = RectangleShape::with_size(Vector2f::new(scale, scale));
    rect.set_fill_color(color);
    rect
}

fn average(arr: &[f32]) -> f32 {
    let mut s = 0.0;
    for i in arr {
        s += i / arr.len() as f32;
    }
    s
}

fn main() {
    let mut grid = create_simple_grid();

    let desktop = VideoMode::desktop_mode();

    let scale = 8.0;

    let mut context = RenderContext {
        scale: scale,
        wood_rect: new_particle_shape(Color::rgb(139, 69, 19), scale),
        water_rect: new_particle_shape(Color::BLUE, scale),
        sand_rect: new_particle_shape(Color::rgb(194, 178, 128), scale)
    };

    let win_width = (grid.width as f32 * context.scale).ceil() as u32;
    let win_height = (grid.height as f32 * context.scale).ceil() as u32;

    let mut window = RenderWindow::new(
        VideoMode::new(win_width, win_height, desktop.bits_per_pixel),
        "Ok zoomer",
        Style::CLOSE,
        &Default::default()
    );

    //window.set_framerate_limit(60);

    window.set_position(Vector2i::new(
        ((desktop.width - win_width) / 2).try_into().unwrap(),
        ((desktop.height - win_height) / 2).try_into().unwrap()
    ));

    let clock = Clock::start();
    let mut prev_tick = 0;
    let tick_time = 0.05;
    let mut is_paused = false;
    let mut is_depressed = false;
    let mut draw_p_type = ParticleType::Water;
    let mut mouse_x = 0;
    let mut mouse_y = 0;

    let font = Font::from_file("assets/Jura-Medium.ttf").unwrap();

    let mut fps_text = Text::default();
    fps_text.set_font(font.deref());
    fps_text.set_position(Vector2f::new(0.0, 0.0));
    fps_text.set_character_size(24);
    fps_text.set_fill_color(Color::WHITE);

    let mut fps_counter = FpsCounter::new();

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
                Event::MouseWheelScrolled { delta, .. } => {
                    // TODO: Make this less waste once we have more particles
                    if draw_p_type == ParticleType::Water {
                        draw_p_type = ParticleType::Sand;
                    } else {
                        draw_p_type = ParticleType::Water;
                    }
                },
                Event::MouseButtonPressed { button: Button::LEFT, x, y } => {
                    is_depressed = true;
                    mouse_x = x;
                    mouse_y = y;
                },
                Event::MouseButtonReleased { button: Button::LEFT, x, y } => {
                    is_depressed = false;
                },
                Event::MouseMoved { x, y } => {
                    mouse_x = x;
                    mouse_y = y;
                }
                _ => { /* Do nothing */ }
            }
        }

        // Activate the window for OpenGL rendering
        window.set_active(true);

        window.clear(Color::BLACK);

        // FIXME: Run on a UI thread instead

        if is_depressed {
            insert_particle(&mut grid, &context, mouse_x, mouse_y, &draw_p_type);
        }

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

        render_grid(&window, &mut context, &mut grid);

        // Render the FPS
        fps_counter.tick(curr_time);
        fps_text.set_string(&format!("{:.0}", fps_counter.get_display_fps()));
        window.draw(&fps_text);

        // End the current frame and display its contents on screen
        window.display();
    }
}