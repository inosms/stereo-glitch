use std::collections::HashSet;

use bevy_ecs::{component::Component, entity::Entity};
use rapier3d::geometry::ColliderHandle;

use crate::object_types::Id;

#[derive(Component)]
pub struct Sensor {
    pub collider: ColliderHandle,
    pub triggered: bool,
    pub id: Option<Id>,
    pub triggered_by: HashSet<Entity>,
}
