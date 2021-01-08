use std::{
    fmt::{Debug, Display},
    time::{Duration, Instant},
};

pub struct Uptime(Instant);

impl Uptime {
    pub fn new() -> Self {
        Self(Instant::now())
    }

    pub fn duration(&self) -> Duration {
        self.0.elapsed()
    }

    pub fn as_secs_u32(&self) -> u32 {
        self.duration().as_secs() as u32
    }
}

impl Display for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut time = self.duration().as_secs();

        let ts = time % 60;
        time /= 60;
        let tm = time % 60;
        time /= 60;
        let th = time % 24;

        write!(f, "{}:{:02}:{:02}", th, tm, ts)
    }
}

impl Debug for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uptime={}s", self.0.elapsed().as_secs())
    }
}
