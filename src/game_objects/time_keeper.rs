use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct TimeKeeper {
    ticks_per_second: u32,
    last_fixed_tick_ms: f64,
    last_variable_tick_ms: f64,
    variable_delta_ms: f64,
    is_in_fixed_tick: bool,
}

impl TimeKeeper {
    pub fn new(ticks_per_second: u32) -> Self {
        Self {
            ticks_per_second,
            last_fixed_tick_ms: instant::now(),
            last_variable_tick_ms: instant::now(),
            variable_delta_ms: 0.0,
            is_in_fixed_tick: false,
        }
    }

    // Returns true if a tick has passed since the last call to tick()
    // and updates the last_fixed_tick_ms
    pub fn tick(&mut self) -> bool {
        let now = instant::now();
        self.variable_delta_ms = now - self.last_variable_tick_ms;
        self.last_variable_tick_ms = now;
        let fixed_delta = now - self.last_fixed_tick_ms;
        let ms_per_tick = 1000.0 / self.ticks_per_second as f64;
        if fixed_delta >= ms_per_tick {
            self.last_fixed_tick_ms = now;
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

    // Returns the current time in seconds
    pub fn now(&self) -> f64 {
        instant::now() / 1000.0
    }

    // Returns the time since the last call to tick() in seconds
    pub fn delta_seconds(&self) -> f32 {
        self.variable_delta_ms as f32 / 1000.0
    }
}
