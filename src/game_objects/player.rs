use bevy_ecs::{component::Component, entity::Entity};

#[derive(Component)]
pub struct Player {
    pub dead: bool,

    // the objects the player is currently pulling
    pub pulled_objects: Vec<Entity>,

    pub charge: f32,
}