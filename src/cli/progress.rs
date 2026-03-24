use std::cell::Cell;
use std::fmt;
use std::time::Instant;

use yansi::Paint;

pub struct Progress {
    step: Cell<usize>,
    steps: usize,
    start: Instant,
}

impl Progress {
    pub fn new(steps: usize) -> Self {
        Self {
            step: Cell::new(0),
            steps,
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let step = self.step.get();
        self.step.set(step + 1);

        write!(
            f,
            "{}",
            format!("[{:3.0}%]", 100.0 * step as f64 / self.steps as f64).dim()
        )
    }
}
