extern crate clap;
extern crate sdl2;

mod fps;
mod physics;
mod grid;
mod render;
mod debug;
mod render_context;
mod save_state;

use std::cmp::max;
use std::cmp::min;
use physics::Physics;
use grid::*;
use render::*;
use debug::DebugWindow;
use render_context::RenderContext;
use save_state::SaveState;

use clap::{AppSettings, Clap};

use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{ MouseButton };

use std::time::{SystemTime};

static TICK_TIME: f32 = 0.05;

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

struct EventLoopContext {
    program_epoch: SystemTime,
    prev_tick: u32,
    depression: Option<MouseButton>,
    is_paused: bool,
    draw_type_index: usize,
    draw_types: Vec<ParticleType>,
    save_filepath: String,
}

impl EventLoopContext {
    fn new(save_filepath: String) -> EventLoopContext {
        let draw_types = vec!(
            ParticleType::Water,
            ParticleType::Sand,
            ParticleType::Wood,
        );

        EventLoopContext {
            program_epoch: SystemTime::now(),
            prev_tick: 0,
            depression: None, // :)
            is_paused: false,
            draw_type_index: 0,
            draw_types: draw_types,
            save_filepath: save_filepath,
        }
    }
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

pub fn main() -> Result<(), String> {
    let opts: Opts = Opts::parse();

    let (grid, max_fill) = match (opts.file_path.clone(), opts.width, opts.height) {
        (Some(fp), _, _) => {
            let state = SaveState::load(fp);
            (state.grid, state.max_fill)
        },
        (_, Some(width), Some(height)) => {
            (ParticleGrid::new(width, height), opts.max_fill)
        },
        _ => {
            return Err("Must specify a file path, or a height&width".to_string());
        }
    };

    let save_filepath = match opts.file_path {
        Some(fp) => fp,
        None     => "save.mp".to_string()
    };

    let elc = EventLoopContext::new(save_filepath);

    run(elc, grid, max_fill);
    Ok(())
}

fn run(mut elc: EventLoopContext, grid: ParticleGrid, max_fill: u8) {
    let mut context = RenderContext::new(&grid, max_fill);
    let mut physics = Physics::new(grid, max_fill);

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("MagicPixel", context.win_width, context.win_height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let main_window_id = window.id();

    let (debug_x, debug_y) = window.position();
    let mut debug_window = DebugWindow::new(debug_x, debug_y, &video_subsystem, &ttf_context);

    let mut renderer = GlslRenderer::new(
        "assets/identity.vert".to_string(),
        "assets/grid.frag".to_string(),
        &context,
        window,
        &video_subsystem
    );

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let events: Vec<Event> = event_pump.poll_iter().collect();

        for event in events {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    elc.is_paused = !elc.is_paused;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    let state = SaveState {
                        grid: physics.get_grid().as_ref().clone(),
                        max_fill: max_fill,
                    };
                    state.save(elc.save_filepath.clone());
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    physics.update();
                },
                Event::KeyDown { keycode: Some(Keycode::Comma), .. } => {
                    elc.draw_type_index = (elc.draw_type_index + 1) % elc.draw_types.len();
                    context.draw_type = elc.draw_types.get(elc.draw_type_index).unwrap().clone();
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
                        elc.depression = Some(mouse_btn);
                    }
                },
                Event::MouseButtonUp { window_id, .. } => {
                    if window_id == main_window_id {
                        elc.depression = None;
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
                    elc.depression = None;
                },
                Event::Window { win_event: WindowEvent::Enter, .. } => {
                    if event_pump.mouse_state().left() {
                        elc.depression = Some(MouseButton::Left);
                    } else if event_pump.mouse_state().right() {
                        elc.depression = Some(MouseButton::Right);
                    }
                },
                _ => {}
            }
        }

        handle_depression(&context, &elc, &mut physics); // Therapy

        let curr_time = get_current_time(&elc);
        tick_physics(curr_time, &mut elc, &mut physics);
        debug_window.render(&physics.get_grid(), &context, curr_time);

        renderer.render(&physics.get_grid(), &context);
    }
}

fn handle_depression(context: &RenderContext, elc: &EventLoopContext, physics: &mut Physics) {
    match elc.depression {
        Some(MouseButton::Left) => {
            let draw_type = elc.draw_types.get(elc.draw_type_index).unwrap();
            insert_particle(physics.get_grid(), &context, &draw_type)
        },
        Some(MouseButton::Right) =>
            edit_particle(physics.get_grid(), &context, |_| {
                Default::default()
            }),
        _ => {},
    }
}

fn get_current_time(elc: &EventLoopContext) -> f32 {
    SystemTime::now()
        .duration_since(elc.program_epoch)
        .unwrap()
        .as_secs_f32()
}

fn tick_physics(curr_time: f32, elc: &mut EventLoopContext, physics: &mut Physics) {
    let curr_tick = (curr_time / TICK_TIME) as u32;

    if curr_tick > elc.prev_tick {
        while elc.prev_tick < curr_tick {
            if ! elc.is_paused {
                physics.update();
            }
            elc.prev_tick += 1;
        }
    }
}
