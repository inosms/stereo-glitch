use bevy_ecs::component::Component;

use crate::{model::ModelHandle};

#[derive(Component)]
pub struct Renderable {
    pub mesh: ModelHandle,
}
