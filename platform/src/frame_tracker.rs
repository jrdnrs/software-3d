use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timings {
    pub start: Instant,
    pub frame_count: usize,
    pub prev_frame: Instant,
    pub ideal_frametime: Duration,
    pub delta_seconds: f32,
}

#[derive(Debug)]
pub struct FrameTracker {
    start: Instant,
    frame_count: usize,
    prev_frame: Instant,
    ideal_frametime: Duration,
    delta_seconds: f32,

    reference_start: Instant,
    reference_frame_count: usize,
}

impl FrameTracker {
    pub fn new(fps: f32) -> Self {
        let start = Instant::now();

        let reference_start = start;
        let ideal_frametime = if fps <= 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(1.0 / fps)
        };
        let prev_frame = start;

        Self {
            start,
            frame_count: 0,
            ideal_frametime,
            prev_frame,
            delta_seconds: 0.0,

            reference_start,
            reference_frame_count: 0,
        }
    }

    /// Sleeps until the target frametime is reached
    pub fn sleep_until_target(&mut self) {
        // Based on time since `timer_start`, calculate how many frames should have passed
        // and sleep until the next one. If we're behind, reset the `timer_start``.
        let now = Instant::now();
        let elapsed = now - self.reference_start;
        let target = self.ideal_frametime * self.reference_frame_count as u32;
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        } else {
            self.reference_start = now;
            self.reference_frame_count = 0;
        }
    }

    /// Returns seconds since function was last called
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now - self.prev_frame;
        self.prev_frame = now;
        self.reference_frame_count += 1;
        self.frame_count += 1;
        self.delta_seconds = delta.as_secs_f32();

        self.delta_seconds
    }

    pub fn timings(&self) -> Timings {
        Timings {
            start: self.start,
            frame_count: self.frame_count,
            prev_frame: self.prev_frame,
            ideal_frametime: self.ideal_frametime,
            delta_seconds: self.delta_seconds,
        }
    }
}
