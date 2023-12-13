use bevy_ecs::component::Component;

use crate::mesh::Handle;

#[derive(Component)]
pub struct Renderable {
    pub mesh: Handle,
}