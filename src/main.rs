extern crate clap;
extern crate sdl2;
extern crate gl;

mod fps;
mod physics;
mod grid;
mod render;
mod debug;

use std::cmp::max;
use std::cmp::min;
use physics::Physics;
use grid::*;
use render::*;
use debug::DebugWindow;

use clap::{AppSettings, Clap};
use serde::{Serialize, Deserialize};

use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{ MouseButton };

use std::io::prelude::*;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short = 'f', long, default_value = "8")]
    max_fill: u8,
    #[clap(short = 's', long)]
    file_path: Option<String>,
    #[clap(short = 'h', long)]
    height: Option<i32>,
    #[clap(short = 'w', long)]
    width: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    max_fill: u8,
    grid: ParticleGrid,
}

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

fn insert_particle(
    grid: &mut ParticleGrid,
    context: &RenderContext,
    p_type: &ParticleType
) {
    edit_particle(grid, context, |_| {
        Particle {
            p_type: p_type.clone(),
            fill_ratio: context.max_fill,
        }
    });
}

fn edit_particle<F>(grid: &mut ParticleGrid, context: &RenderContext, edit_func: F) where
    F: Fn(&Particle) -> Particle
{
    let x = (context.mouse_x as f32 / context.scale) as i32;
    let y = (context.mouse_y as f32 / context.scale) as i32;

    if grid.in_bounds(x, y) {
        grid.set(x, y, edit_func(grid.get(x, y)));
    }
}

fn load_state(path: String) -> SaveState {
    let mut f = File::open(path).expect("File not found");

    // Ignore version for now
    let mut version: [u8; 1] = [0; 1];
    let _ = f.read(&mut version).expect("File is empty");

    let mut buff_bois: Vec<u8> = Vec::new();
    let _ = f.read_to_end(&mut buff_bois).unwrap();

    bincode::deserialize(&buff_bois).expect("Corrupted file")
}

fn save_state(path: String, state: SaveState) {
    let encoded: Vec<u8> = bincode::serialize(&state).unwrap();

    let mut buffer = File::create(path).expect("Could not save to path");
    // Version byte
    let _ = buffer.write(&[1u8]);
    let _ = buffer.write_all(encoded.as_ref());
}

pub fn main() -> Result<(), String> {
    let opts: Opts = Opts::parse();

    let (grid, max_fill) = match (opts.file_path, opts.width, opts.height) {
        (Some(fp), _, _) => {
            let state = load_state(fp);
            (state.grid, state.max_fill)
        },
        (_, Some(width), Some(height)) => {
            (ParticleGrid::new(width, height), opts.max_fill)
        },
        _ => {
            return Err("Must specify a file path, or a height&width".to_string());
        }
    };

    // FIXME: This is waste
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
        draw_type: ParticleType::Water,
        max_fill: max_fill,
    };

    let mut physics = Physics::new(grid, max_fill);

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("MagicPixel", win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let main_window_id = window.id();

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

    let mut depression = None; // :)
    let mut is_paused = false;

    let draw_types = vec!(
        ParticleType::Water,
        ParticleType::Sand,
        ParticleType::Wood,
    );
    let mut draw_type_index: usize = 0;

    'running: loop {
        let events: Vec<Event> = event_pump.poll_iter().collect();

        for event in events {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    is_paused = !is_paused;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    save_state("save.mp".to_string(), SaveState {
                        grid: physics.get_grid().as_ref().clone(),
                        max_fill: max_fill,
                    });
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    physics.update();
                },
                Event::KeyDown { keycode: Some(Keycode::Comma), .. } => {
                    draw_type_index = (draw_type_index + 1) % draw_types.len();
                    context.draw_type = draw_types.get(draw_type_index).unwrap().clone();
                },
                Event::MouseMotion { x, y , window_id, .. } => {
                    if window_id == main_window_id {
                        context.mouse_x = x;
                        context.mouse_y = y;
                    }
                },
                Event::MouseButtonDown { x, y , window_id, mouse_btn, .. } => {
                    context.mouse_x = x;
                    context.mouse_y = y;

                    if window_id == main_window_id {
                        depression = Some(mouse_btn);
                    }
                },
                Event::MouseButtonUp { window_id, .. } => {
                    if window_id == main_window_id {
                        depression = None;
                    }
                },
                Event::MouseWheel { y, .. } => {
                    // wow impressive
                    edit_particle(physics.get_grid(), &context, |p| {
                        if p.p_type == ParticleType::Empty {
                            p.clone()
                        } else {
                            let new_fill_ratio = p.fill_ratio as i32 + y;

                            Particle {
                                fill_ratio: max(1, min(max_fill, new_fill_ratio as u8)),
                                ..p.clone()
                            }
                        }
                    });
                },
                Event::Window { win_event: WindowEvent::Leave, .. } => {
                    depression = None;
                },
                Event::Window { win_event: WindowEvent::Enter, .. } => {
                    if event_pump.mouse_state().left() {
                        depression = Some(MouseButton::Left);
                    } else if event_pump.mouse_state().right() {
                        depression = Some(MouseButton::Right);
                    }
                },
                _ => {}
            }
        }

        match depression {
            Some(MouseButton::Left) => {
                let draw_type = draw_types.get(draw_type_index).unwrap();
                insert_particle(physics.get_grid(), &context, &draw_type)
            },
            Some(MouseButton::Right) =>
                edit_particle(physics.get_grid(), &context, |_| {
                    Default::default()
                }),
            _ => {},
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
        debug_window.render(&physics.get_grid(), &context, curr_time);
    }

    Ok(())
}
