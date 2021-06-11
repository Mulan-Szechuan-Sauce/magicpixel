extern crate sdl2;
extern crate gl;

mod fps;

mod physics;
use physics::Physics;

mod grid;
use grid::*;

mod render;
use render::*;

mod debug;
use debug::DebugWindow;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{SystemTime};

pub struct RenderContext {
    pub scale: f32,
    pub win_width: u32,
    pub win_height: u32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl RenderContext {
    pub fn get_mouse_grid_x(&self) -> i32 {
        (self.mouse_x as f32 / self.scale) as i32
    }

    pub fn get_mouse_grid_y(&self) -> i32 {
        (self.mouse_y as f32 / self.scale) as i32
    }
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

fn create_simple_grid() -> ParticleGrid {
    let mut grid = ParticleGrid::new(100, 100);

    for x in 10..20 {
        for y in 0..5 {
            grid.set(x, y, Particle {
                p_type: ParticleType::Water,
                fill_ratio: MAX_FILL,
                ..Default::default()
            });
        }
    }

    grid
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

pub fn main() {
    let grid = create_simple_grid();

    let max_win_width = 2400.0;
    let max_win_height = 1400.0;

    let scale =
        ((max_win_width / grid.width as f32)
          .min(max_win_height / grid.height as f32))
          .floor();

    let win_width = (grid.width as f32 * scale).ceil() as u32;
    let win_height = (grid.height as f32 * scale).ceil() as u32;

    let mut context = RenderContext {
        scale: scale,
        win_width: win_width,
        win_height: win_height,
        grid_width: grid.width,
        grid_height: grid.height,
        mouse_x: 0,
        mouse_y: 0,
    };

    let mut physics = Physics::new(grid);

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("MagicPixel", win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let (debug_x, debug_y) = window.position();
    let mut debug_window = DebugWindow::new(debug_x, debug_y, &video_subsystem, &ttf_context);

    let mut canvas = window.into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();

    // initialization
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    // sdl::render creates a context for you, if you use a Canvas you need to use it.
    let _ = canvas.window().gl_set_context_to_current();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let program_epoch = SystemTime::now();
    let tick_time = 0.05;
    let mut prev_tick = 0;

    let mut renderer = GlslRenderer::new(
        "assets/identity.vert".to_string(),
        "assets/grid.frag".to_string(),
        &context
    );

    let mut is_depressed = false;
    let mut is_paused = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    is_paused = !is_paused;
                },
                Event::MouseMotion { x, y , .. } => {
                    context.mouse_x = x;
                    context.mouse_y = y;
                },
                Event::MouseButtonDown { x, y , .. } => {
                    context.mouse_x = x;
                    context.mouse_y = y;

                    is_depressed = true;
                },
                Event::MouseButtonUp { .. } => {
                    is_depressed = false;
                },
                _ => {}
            }
        }

        if is_depressed {
            insert_particle(physics.get_grid(), &context, &ParticleType::Water);
        }

        canvas.clear();

        let curr_time = SystemTime::now()
            .duration_since(program_epoch)
            .unwrap()
            .as_secs_f32();

        let curr_tick = (curr_time / tick_time) as u32;

        if curr_tick > prev_tick {
            while prev_tick < curr_tick {
                if ! is_paused {
                    physics.update();
                }
                prev_tick += 1;
            }
        }

        renderer.render(&physics.get_grid(), &context);

        canvas.present();
        debug_window.render(curr_time);
    }
}
