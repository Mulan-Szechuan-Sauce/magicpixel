extern crate sdl2;
extern crate gl;

mod physics;
use physics::Physics;

mod grid;
use grid::*;

mod render;
use render::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

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

    for x in 69..80 {
        for y in 0..1 {
            grid.set(x, y, Particle {
                p_type: ParticleType::Water,
                fill_ratio: 1,
                ..Default::default()
            });
        }
    }

    grid
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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();
 
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    // initialization
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    // sdl::render creates a context for you, if you use a Canvas you need to use it.
    let _ = canvas.window().gl_set_context_to_current();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let program_epoch = SystemTime::now();
    let tick_time = 0.05;
    let mut prev_tick = 0;
    let is_paused = false;

    let mut renderer = GlslRenderer::new("assets/grid.frag".to_string(), &context);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        //canvas.clear();

        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

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

        canvas.present();
    }
}
