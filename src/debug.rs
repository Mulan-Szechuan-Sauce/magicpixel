extern crate sdl2;

use sdl2::ttf::Font;
use sdl2::render::{Canvas};
use sdl2::video::{Window};
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;

static DEBUG_WIDTH: u32 = 300;
static DEBUG_HEIGHT: u32 = 500;

pub struct DebugWindow<'a> {
    canvas: Canvas<Window>,
    font: Font<'a, 'a>,
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
        let mut font: Font<'a, 'a> = ttf_context.load_font("assets/Jura-Medium.ttf", 24).unwrap();
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        DebugWindow {
            canvas: canvas,
            font: font,
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color) {
        let surface = self.font
            .render("Hello Rust!")
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

    pub fn render(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.draw_text("Hello crust!", 10, 10, Color::WHITE);

        self.canvas.present();
    }
}
