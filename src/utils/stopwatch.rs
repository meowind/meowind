use std::time::Instant;

pub struct Stopwatch {
    pub start: Instant,
}

impl Stopwatch {
    pub fn start_new() -> Stopwatch {
        Stopwatch {
            start: Instant::now(),
        }
    }

    pub fn millis(&mut self) -> usize {
        self.start.elapsed().as_millis() as usize
    }

    pub fn micros(&mut self) -> usize {
        self.start.elapsed().as_micros() as usize
    }
}
