use sfml::graphics::{
    RenderWindow, RenderTarget, Color, RectangleShape,
    Transformable, Text, Font, Texture, Sprite
};
use sfml::window::{VideoMode, Event, Style, Key};
use sfml::window::mouse::{Button};
use sfml::system::{Vector2i, Vector2f, Clock};
use std::convert::{TryInto};
use sfml::SfBox;

mod ui_grid;
use ui_grid::draw_overlay;

mod physics;
use physics::Physics;

mod grid;
use grid::*;

mod fps;
use fps::{FpsCounter};

mod render;
use render::*;

pub struct RenderContext {
    pub scale: f32,
    pub win_width: u32,
    pub win_height: u32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub rect: RectangleShape<'static>,
    pub font: SfBox<Font>,
    pub display_texture: SfBox<Texture>,
    pub display_pixels: Box<[u8]>,
}

impl RenderContext {
    pub fn get_mouse_grid_x(&self) -> i32 {
        (self.mouse_x as f32 / self.scale) as i32
    }

    pub fn get_mouse_grid_y(&self) -> i32 {
        (self.mouse_y as f32 / self.scale) as i32
    }
}

fn create_simple_grid() -> ParticleGrid {
    #[allow(unused_mut)]
    let mut grid = ParticleGrid::new(100, 100);
    //let mut grid = ParticleGrid::new(2, 4);

    // for x in 0..100 {
    //     grid.set(x, 49, Particle {
    //         p_type: ParticleType::Water,
    //         fill_ratio: 1,
    //         ..Default::default()
    //     });
    // }

    grid
}

fn render_grid(window: &mut RenderWindow, context: &mut RenderContext, grid: &ParticleGrid) {
    for x in 0..grid.width {
        for y in 0..grid.height {
            let p = grid.get(x, y);
            let scale = context.scale as usize;
            let fill_amount = (255u64 * p.fill_ratio as u64 / MAX_FILL as u64) as u8;

            for s in 0..(scale*scale) {
                // This may be slow
                let x_initial = x as usize * scale + s % scale;
                let y_initial = y as usize * scale + s / scale;
                let i = 4 * (x_initial + y_initial * scale * grid.width as usize);

                unsafe {
                    let pixel_ptr: *mut u8 =
                        context.display_pixels.as_mut_ptr().add(i);

                    match p.p_type {
                        ParticleType::Water => {
                            *pixel_ptr.offset(0) = 0;
                            *pixel_ptr.offset(1) = 0;
                            *pixel_ptr.offset(2) = 255;
                            *pixel_ptr.offset(3) = fill_amount;
                        },
                        _ => {
                            *pixel_ptr.offset(0) = 0;
                            *pixel_ptr.offset(1) = 0;
                            *pixel_ptr.offset(2) = 0;
                            *pixel_ptr.offset(3) = 0;
                        }
                    };
                }
            }
        }
    }

    unsafe {
        context.display_texture.update_from_pixels(
            &context.display_pixels,
            grid.width as u32 * context.scale as u32,
            grid.height as u32 * context.scale as u32,
            0,
            0 
        );
    }

    let s = Sprite::with_texture(&context.display_texture);
    window.draw(&s);
}

fn insert_particle(
    grid: &mut ParticleGrid,
    context: &RenderContext,
    p_type: &ParticleType
) {
    let x = (context.mouse_x as f32 / context.scale) as i32;
    let y = (context.mouse_y as f32 / context.scale) as i32;

    if grid.in_bounds(x, y) {
        grid.set(x, y, Particle {
            p_type: p_type.clone(),
            ..Default::default()
        });
    }
}

fn sum_water(grid: &ParticleGrid) -> u64 {
    let mut net_fill_ratio: u64 = 0;

    for particle in &grid.grid {
        if particle.p_type == ParticleType::Water {
            net_fill_ratio += particle.fill_ratio as u64
        }
    }

    return net_fill_ratio;
}

fn main() {
    let grid = create_simple_grid();

    let desktop = VideoMode::desktop_mode();

    let max_win_width = 2400.0;
    let max_win_height = 1400.0;

    let scale =
        ((max_win_width / grid.width as f32)
          .min(max_win_height / grid.height as f32))
          .floor();

    let win_width = (grid.width as f32 * scale).ceil() as u32;
    let win_height = (grid.height as f32 * scale).ceil() as u32;

    let pixel_count = win_width as usize * win_height as usize;
    let rgba_pixel_count = 4 * pixel_count;

    let mut context = RenderContext {
        scale: scale,
        win_width: win_width,
        win_height: win_height,
        grid_width: grid.width,
        grid_height: grid.height,
        mouse_x: 0,
        mouse_y: 0,
        rect: RectangleShape::with_size(Vector2f::new(scale, scale)),
        // FIXME:
        font: Font::from_file("/home/elijah/code/magicpixel/assets/Jura-Medium.ttf").unwrap(),
        display_texture: Texture::new(win_width, win_height).unwrap(),
        display_pixels: vec![0; rgba_pixel_count].into_boxed_slice(),
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

    // FIXME: don't clone this, use Rc
    let fps_font = context.font.clone();
    let debug_font = context.font.clone();
    let mut fps_counter = FpsCounter::new(&fps_font);

    let mut physics = Physics::new(grid);

    let mut renderer = GlslRenderer::new("assets/grid.frag".to_string(), &context);

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
                Event::KeyPressed { code: Key::R, .. } =>
                    physics = Physics::new(create_simple_grid()),
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
                    context.mouse_x = x;
                    context.mouse_y = y;
                },
                Event::MouseButtonReleased { button: Button::LEFT, .. } => {
                    is_depressed = false;
                },
                Event::MouseMoved { x, y } => {
                    context.mouse_x = x;
                    context.mouse_y = y;
                }
                _ => { /* Do nothing */ }
            }
        }

        // Activate the window for OpenGL rendering
        window.set_active(true);

        window.clear(Color::BLACK);

        // FIXME: Run on a UI thread instead

        if is_depressed {
            insert_particle(physics.get_grid(), &context, &draw_p_type);
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

        renderer.render(&mut window, physics.get_grid());

        // Render the FPS
        fps_counter.tick(curr_time);
        window.draw(fps_counter.get_display_text());

        let mut debug_text = Text::default();

        debug_text.set_font(&debug_font);
        debug_text.set_position(Vector2f::new(0.0, 24.0));
        debug_text.set_character_size(24);
        debug_text.set_fill_color(Color::WHITE);

        let x = context.get_mouse_grid_x();
        let y = context.get_mouse_grid_y();
        let grid = physics.get_grid();

        if grid.in_bounds(x, y) {
            let particle = grid.get(x, y).clone();
            let fr = particle.fill_ratio;
            let p_type = particle.p_type;
            let bearing = particle.bearing;

            let water_count = sum_water(grid);

            debug_text.set_string(&format!("{}, {:?}, {:?}, {}", fr, bearing, p_type, water_count));
        }

        window.draw(&debug_text);

        draw_overlay(&mut window, &context, &grid);

        // End the current frame and display its contents on screen
        window.display();
    }
}
