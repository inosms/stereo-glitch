use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct TimeKeeper {
    ticks_per_second: u32,
    last_tick_ms: f64,
    is_in_fixed_tick: bool,
}

impl TimeKeeper {
    pub fn new(ticks_per_second: u32) -> Self {
        Self {
            ticks_per_second,
            last_tick_ms: instant::now(),
            is_in_fixed_tick: false,
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
            self.is_in_fixed_tick = true;
            true
        } else {
            self.is_in_fixed_tick = false;
            false
        }
    }

    // Returns true if the last call to tick() returned true
    // This means that in this system update cycle all fixed updates
    // should be performed
    pub fn is_in_fixed_tick(&self) -> bool {
        self.is_in_fixed_tick
    }
}
