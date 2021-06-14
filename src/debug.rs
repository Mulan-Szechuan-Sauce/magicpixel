extern crate sdl2;

use crate::MAX_FILL;
use crate::RenderContext;
use crate::ParticleGrid;
use sdl2::ttf::Font;
use sdl2::render::{Canvas};
use sdl2::video::{Window};
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;

use crate::fps::FpsCounter;

static DEBUG_WIDTH: u32 = 300;
static DEBUG_HEIGHT: u32 = 500;

pub struct DebugWindow<'a> {
    canvas: Canvas<Window>,
    font: Font<'a, 'a>,
    counter: FpsCounter,
}

impl <'a> DebugWindow<'a> {
    pub fn new(
        x: i32,
        y: i32,
        video_subsystem: &sdl2::VideoSubsystem,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext
    ) -> DebugWindow<'a> {
        let window = video_subsystem.window("MagicPixel Debug", DEBUG_WIDTH, DEBUG_HEIGHT)
            .position(x - DEBUG_WIDTH as i32, y)
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        // Load a font
        let mut font: Font<'a, 'a> = ttf_context.load_font("assets/FiraCode-Light.ttf", 24).unwrap();
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        DebugWindow {
            canvas: canvas,
            font: font,
            counter: FpsCounter::new(),
        }
    }

    // NOTE: Inefficient text rendering, but it's "ok enough"
    pub fn draw_text(&mut self, text: String, x: i32, y: i32, color: Color) {
        let surface = self.font
            .render(&text)
            .blended(color)
            .map_err(|e| e.to_string()).unwrap();

        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();

        let rect = sdl2::rect::Rect::new(x, y, width, height);
        let _ = self.canvas.copy(&texture, None, Some(rect));
    }

    // The year was 1995.
    pub fn render(&mut self, grid: &ParticleGrid, context: &RenderContext, curr_time: f32) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        let grid_y = (context.mouse_y as f32 / context.scale).floor() as i32;
        let grid_x = (context.mouse_x as f32 / context.scale).floor() as i32;

        let particle = grid.get(grid_x, grid_y);

        let fps_text = self.counter.tick(curr_time);
        self.draw_text(format!("FPS: {}", fps_text), 10, 10, Color::WHITE);
        self.draw_text(format!("{:?}", particle.p_type), 10, 35, Color::WHITE);
        self.draw_text(format!("{: >3}/{}", particle.fill_ratio, MAX_FILL), 110, 35, Color::WHITE);
        self.draw_text(format!("{:?}", particle.bearing), 10, 60, Color::WHITE);
        self.draw_text(format!("{:?}", context.draw_type), 10, 85, Color::WHITE);

        self.canvas.present();
    }
}
