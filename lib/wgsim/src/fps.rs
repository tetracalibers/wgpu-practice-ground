use std::{
  collections::VecDeque,
  time::{Duration, Instant},
};

#[derive(Debug)]
pub struct FpsCounter {
  last_second_frames: VecDeque<Instant>,
  last_print_time: Instant,
}

impl Default for FpsCounter {
  fn default() -> Self {
    Self::new()
  }
}

impl FpsCounter {
  pub fn new() -> Self {
    Self {
      last_second_frames: VecDeque::with_capacity(128),
      last_print_time: Instant::now(),
    }
  }

  pub fn print_fps(&mut self, interval: u64) {
    let now = Instant::now();
    let a_second_ago = now - Duration::from_secs(1);

    while self.last_second_frames.front().map_or(false, |t| *t < a_second_ago) {
      self.last_second_frames.pop_front();
    }

    self.last_second_frames.push_back(now);

    if now - self.last_print_time >= Duration::from_secs(interval) {
      let fps = self.last_second_frames.len();
      println!("FPS: {}", fps);
      self.last_print_time = now;
    }
  }
}
