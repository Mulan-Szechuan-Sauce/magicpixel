use sfml::graphics::{RenderWindow, RenderTarget, Color, RectangleShape, Shape, RenderStates, Transformable, Text, Font, Texture};
use sfml::window::{VideoMode, Event, Style, Key};
use sfml::window::mouse::{Button};
use sfml::system::{Vector2i, Vector2f, Clock};
use std::convert::{TryInto};
use sfml::SfBox;

mod physics;
use physics::Physics;

mod grid;
use grid::*;

mod fps;
use fps::{FpsCounter};


use std::rc::Rc;

pub struct Child {
    pub word: Rc<String>,
}

pub struct Parent {
    pub word: Rc<String>,
    pub child: Child,
}

impl Parent {
    pub fn new() -> Parent {
        let word: Rc<String> = Rc::new("hello".to_string());

        let c = Child {
            word: Rc::clone(&word)
        };

        Parent {
            word: word,
            child: c,
        }
    }
}

pub struct RenderContext {
    pub scale: f32,
    pub rect: RectangleShape<'static>,
    pub font: SfBox<Font>,
    pub display_texture: SfBox<Texture>,
    pub display_pixels: Box<[u8]>,
}

fn create_simple_grid() -> ParticleGrid {
    let mut grid = ParticleGrid::new(50, 25);

    // for y in 10..(grid.height - 10) {
    //     grid.set(grid.width / 2, y, Particle {
    //         p_type: ParticleType::Sand,
    //         ..Default::default()
    //     });
    // }

    // for y in 0..(grid.height - 10) {
    //     for x in 20..40 {
    //         grid.set(x, y, Particle {
    //             p_type: ParticleType::Water,
    //             ..Default::default()
    //         });
    //     }
    // }

    // grid.set(5, 5, Particle {
    //     p_type: ParticleType::Water,
    //     ..Default::default()
    // });

    grid.set(5, 24, Particle {
        p_type: ParticleType::Water,
        fill_ratio: 3,
        ..Default::default()
    });
    grid.set(6, 24, Particle {
        p_type: ParticleType::Water,
        fill_ratio: 1,
        ..Default::default()
    });

    grid
}

fn render_particle(window: &RenderWindow, context: &mut RenderContext, p: &Particle, x: i32, y: i32) {
    let scale = context.scale;
    let rect = &mut context.rect;

    match p.p_type {
        ParticleType::Water => {
            let baz: u64 = 255u64 * p.fill_ratio as u64 / MAX_FILL as u64;
            rect.set_fill_color(Color::rgba(0, 0, 255, baz as u8));
        },
        ParticleType::Sand => {
            rect.set_fill_color(Color::rgb(194, 178, 128));
        },
        _ => {}
    }

    rect.set_position(Vector2f::new(x as f32 * scale, y as f32 * scale));
    window.draw_rectangle_shape(&rect, &RenderStates::default());
}

fn render_grid(window: &RenderWindow, context: &mut RenderContext, grid: &ParticleGrid) {
    for x in 0..grid.width {
        for y in 0..grid.height {
            let p = grid.get(x, y);

            if p.p_type != ParticleType::Empty {
                render_particle(window, context, &p, x, y);
            }
        }
    }

    //context.display_pixels
}

fn insert_particle(
    grid: &mut ParticleGrid,
    context: &RenderContext,
    mouse_x: i32,
    mouse_y: i32,
    p_type: &ParticleType
) {
    let x = (mouse_x as f32 / context.scale) as i32;
    let y = (mouse_y as f32 / context.scale) as i32;

    if grid.in_bounds(x, y) {
        grid.set(x, y, Particle {
            p_type: p_type.clone(),
            ..Default::default()
        });
    }
}

fn main() {
    let grid = create_simple_grid();

    let desktop = VideoMode::desktop_mode();

    let scale = 40.0;

    let win_width = (grid.width as f32 * scale).ceil() as u32;
    let win_height = (grid.height as f32 * scale).ceil() as u32;

    let mut context = RenderContext {
        scale: scale,
        rect: RectangleShape::with_size(Vector2f::new(scale, scale)),
        // FIXME:
        font: Font::from_file("/home/elijah/code/magicpixel/assets/Jura-Medium.ttf").unwrap(),
        display_texture: Texture::new(win_width, win_height).unwrap(),
        display_pixels: vec![0; (win_width * win_height) as usize].into_boxed_slice(),
    };

    let mut window = RenderWindow::new(
        VideoMode::new(win_width, win_height, desktop.bits_per_pixel),
        "Ok zoomer",
        Style::CLOSE,
        &Default::default()
    );

    window.set_position(Vector2i::new(
        ((desktop.width - win_width) / 2).try_into().unwrap(),
        ((desktop.height - win_height) / 2).try_into().unwrap()
    ));

    let clock = Clock::start();
    let mut prev_tick = 0;
    let tick_time = 0.05;
    let mut is_paused = true;
    let mut is_depressed = false;
    let mut draw_p_type = ParticleType::Water;
    let mut mouse_x = 0;
    let mut mouse_y = 0;

    // FIXME: don't clone this, use Rc
    let fps_font = context.font.clone();
    let debug_font = context.font.clone();
    let mut fps_counter = FpsCounter::new(&fps_font);

    let mut physics = Physics::new(grid);

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
                Event::KeyReleased { code: Key::SPACE, .. } =>
                    physics.update(),
                Event::MouseWheelScrolled { .. } => {
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
                Event::MouseButtonReleased { button: Button::LEFT, .. } => {
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
            insert_particle(physics.get_grid(), &context, mouse_x, mouse_y, &draw_p_type);
        }

        let curr_time = clock.elapsed_time().as_seconds();
        let curr_tick = (curr_time / tick_time) as u32;

        if curr_tick > prev_tick {
            while prev_tick < curr_tick {
                if ! is_paused {
                    physics.update();
                }
                prev_tick += 1;
            }
        }

        render_grid(&window, &mut context, physics.get_grid());

        // Render the FPS
        fps_counter.tick(curr_time);
        window.draw(fps_counter.get_display_text());

        let mut debug_text = Text::default();

        debug_text.set_font(&debug_font);
        debug_text.set_position(Vector2f::new(0.0, 24.0));
        debug_text.set_character_size(24);
        debug_text.set_fill_color(Color::WHITE);

        let x = (mouse_x as f32 / context.scale) as i32;
        let y = (mouse_y as f32 / context.scale) as i32;
        let grid = physics.get_grid();

        if grid.in_bounds(x, y) {
            let france = grid.get(x, y).fill_ratio;
            debug_text.set_string(&format!("{}", france));
        }

        window.draw(&debug_text);

        // End the current frame and display its contents on screen
        window.display();
    }
}
