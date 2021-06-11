const FPS_DISPLAY_TIME_DELTA: f32 = 0.1;
const FPS_HISTORY_SIZE: usize = 10;

pub struct FpsCounter {
    last_display_time: f32,
    last_tick_time: f32,
    fps_history: [f32; FPS_HISTORY_SIZE],
    fps_history_next: usize,
    display_text: String,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            last_display_time: 0.0,
            last_tick_time: 0.0,
            fps_history: [0.0; FPS_HISTORY_SIZE],
            fps_history_next: 0,
            display_text: String::new(),
        }
    }

    pub fn tick(&mut self, t: f32) -> String {
        let tick_fps = 1.0 / (t - self.last_tick_time as f32);
        self.last_tick_time = t;

        self.fps_history[self.fps_history_next] = tick_fps;
        self.fps_history_next = (self.fps_history_next + 1) % self.fps_history.len();

        if t > self.last_display_time + FPS_DISPLAY_TIME_DELTA {
            self.last_display_time = t;
            let display_fps = average(&self.fps_history);
            self.display_text = format!("{:.0}", display_fps);
        }

        self.display_text.clone()
    }
}

fn average(arr: &[f32]) -> f32 {
    let mut s = 0.0;
    for i in arr {
        s += i / arr.len() as f32;
    }
    s
}
