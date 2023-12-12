use bevy_ecs::component::Component;
use cgmath::Rotation3;


#[derive(Component, Clone, Copy, Debug)]
pub struct Position {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Position {
    pub fn get_cell(&self) -> (i32, i32) {
        (
            self.position.x.floor() as i32,
            (-self.position.y).floor() as i32,
        )
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
