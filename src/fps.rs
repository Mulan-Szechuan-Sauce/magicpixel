use sfml::graphics::{RenderWindow, Color, Transformable, Text, Font};
use sfml::system::{Vector2f};

use core::ops::Deref;

const FPS_DISPLAY_TIME_DELTA: f32 = 0.1;
const FPS_HISTORY_SIZE: usize = 10;

pub struct FpsCounter {
    last_display_time: f32,
    last_tick_time: f32,
    fps_history: [f32; FPS_HISTORY_SIZE],
    fps_history_next: usize,
    display_fps: f32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            last_display_time: 0.0,
            last_tick_time: 0.0,
            fps_history: [0.0; FPS_HISTORY_SIZE],
            fps_history_next: 0,
            display_fps: 0.0,
        }
    }

    pub fn tick(&mut self, t: f32) {
        let tick_fps = 1.0 / (t - self.last_tick_time as f32);
        self.last_tick_time = t;

        self.fps_history[self.fps_history_next] = tick_fps;
        self.fps_history_next = (self.fps_history_next + 1) % self.fps_history.len();

        if t > self.last_display_time + FPS_DISPLAY_TIME_DELTA {
            self.last_display_time = t;
            self.display_fps = average(&self.fps_history);
        }
    }

    pub fn get_display_fps(&self) -> f32 {
        self.display_fps
    }
}

fn average(arr: &[f32]) -> f32 {
    let mut s = 0.0;
    for i in arr {
        s += i / arr.len() as f32;
    }
    s
}
