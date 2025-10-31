use core::time::Duration;
use std::collections::VecDeque;
use web_time::Instant;

pub struct MetricsRecorder {
    fps: f32,
    last_update_time: Instant,
    frames_last_time_span: usize,
    // Rolling 5s performance history for overlay averages
    step_hist_5s: VecDeque<(Instant, f32)>,
    draw_hist_5s: VecDeque<(Instant, f32)>,
    last_step_count: usize,
}

impl MetricsRecorder {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
            step_hist_5s: VecDeque::new(),
            draw_hist_5s: VecDeque::new(),
            last_step_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.fps = 0.0;
        self.last_update_time = Instant::now();
        self.frames_last_time_span = 0;
        self.step_hist_5s.clear();
        self.draw_hist_5s.clear();
        self.last_step_count = 0;
    }

    pub fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }
    pub fn last_step_count(&self) -> usize {
        self.last_step_count
    }
    pub fn set_last_step_count(&mut self, v: usize) {
        self.last_step_count = v;
    }

    pub fn record_sample(&mut self, step_ms: f32, draw_ms: f32) {
        let now = Instant::now();
        self.step_hist_5s.push_back((now, step_ms));
        self.draw_hist_5s.push_back((now, draw_ms));
        // Prune older than 5 seconds
        let window = Duration::from_secs(5);
        while let Some((t, _)) = self.step_hist_5s.front() {
            if now.duration_since(*t) > window {
                self.step_hist_5s.pop_front();
            } else {
                break;
            }
        }
        while let Some((t, _)) = self.draw_hist_5s.front() {
            if now.duration_since(*t) > window {
                self.draw_hist_5s.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn step_avg_5s(&self) -> f32 {
        avg_5s(&self.step_hist_5s)
    }
    pub fn draw_avg_5s(&self) -> f32 {
        avg_5s(&self.draw_hist_5s)
    }
}

impl Default for MetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

fn avg_5s(hist: &VecDeque<(Instant, f32)>) -> f32 {
    if hist.is_empty() {
        return 0.0;
    }
    let sum: f32 = hist.iter().map(|(_, v)| *v).sum();
    sum / (hist.len() as f32)
}
