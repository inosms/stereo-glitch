use std::collections::HashSet;

use bevy_ecs::system::Resource;

use super::time_keeper::TimeKeeper;

#[derive(Resource)]
pub struct GlitchAreaVisibility {
    // 0 = invisible, 1 = fully visible
    // if the player has more than 0 charge, the glitch area is fully visible
    // this variable is used for slow interpolation between the two states
    pub visibility: f32,

    // The cells that are currently glitched
    pub glitch_cells: HashSet<(i32, i32)>,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlitchAreaVisibilityDTO {
    time: f32,
    visibility: f32,
    
    // padding to 16 bytes
    padding_0: f32,
    padding_1: f32,
}

impl GlitchAreaVisibilityDTO {
    pub fn new(visibility: f32, time: f32) -> Self {
        Self {
            visibility,
            time,
            padding_0: 0.0,
            padding_1: 0.0,
        }
    }
}

impl From<&GlitchAreaVisibility> for GlitchAreaVisibilityDTO {
    fn from(visibility: &GlitchAreaVisibility) -> Self {
        Self::new(visibility.visibility, TimeKeeper::now() as f32)
    }
}