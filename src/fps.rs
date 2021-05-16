use sfml::graphics::{Color, Transformable, Text, Font};
use sfml::system::{Vector2f};
use sfml::SfBox;

use std::ops::Deref;

const FPS_DISPLAY_TIME_DELTA: f32 = 0.1;
const FPS_HISTORY_SIZE: usize = 10;

pub struct FpsCounter<'a> {
    last_display_time: f32,
    last_tick_time: f32,
    fps_history: [f32; FPS_HISTORY_SIZE],
    fps_history_next: usize,
    pub text: Text<'a>,
}

impl<'a> FpsCounter<'a> {
    pub fn new(font: &'a Font) -> FpsCounter<'a> {
        let mut text = Text::default();

        text.set_font(font);
        text.set_position(Vector2f::new(0.0, 0.0));
        text.set_character_size(24);
        text.set_fill_color(Color::WHITE);

        FpsCounter {
            last_display_time: 0.0,
            last_tick_time: 0.0,
            fps_history: [0.0; FPS_HISTORY_SIZE],
            fps_history_next: 0,
            text: text,
        }
    }

    pub fn tick(&mut self, t: f32) {
        let tick_fps = 1.0 / (t - self.last_tick_time as f32);
        self.last_tick_time = t;

        self.fps_history[self.fps_history_next] = tick_fps;
        self.fps_history_next = (self.fps_history_next + 1) % self.fps_history.len();

        if t > self.last_display_time + FPS_DISPLAY_TIME_DELTA {
            self.last_display_time = t;
            let display_fps = average(&self.fps_history);
            self.text.set_string(&format!("{:.0}", display_fps));
        }
    }

    pub fn get_display_text(&self) -> &Text {
        &self.text
    }
}

fn average(arr: &[f32]) -> f32 {
    let mut s = 0.0;
    for i in arr {
        s += i / arr.len() as f32;
    }
    s
}
