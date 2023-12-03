use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct TimeKeeper {
    ticks_per_second: u32,
    last_tick_ms: f64,
}

impl TimeKeeper {
    pub fn new(ticks_per_second: u32) -> Self {
        Self {
            ticks_per_second,
            last_tick_ms: instant::now(),
        }
    }

    // Returns true if a tick has passed since the last call to tick()
    // and updates the last_tick_ms
    pub fn tick(&mut self) -> bool {
        let now = instant::now();
        let delta = now - self.last_tick_ms;
        let ms_per_tick = 1000.0 / self.ticks_per_second as f64;
        if delta >= ms_per_tick {
            self.last_tick_ms = now;
            true
        } else {
            false
        }
    }

    // Returns true if a tick has passed since the last call to tick()
    // Does not update the last_tick_ms
    pub fn peek(&self) -> bool {
        let now = instant::now();
        let delta = now - self.last_tick_ms;
        let ms_per_tick = 1000.0 / self.ticks_per_second as f64;
        delta > ms_per_tick
    }
}
