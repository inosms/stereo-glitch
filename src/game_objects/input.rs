use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct Input {
    pub player_movement: Option<cgmath::Vector3<f32>>, // if consumed  set to None
    pub player_paralized_cooldown: f32,
}