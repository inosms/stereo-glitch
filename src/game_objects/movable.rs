use bevy_ecs::{component::Component, system::{ResMut, Query}, entity::Entity};

use crate::physics::PhysicsSystem;

use super::{player::Player, physics_body::PhysicsBody};

#[derive(Component)]
pub struct Movable {
}

impl Default for Movable {
    fn default() -> Self {
        Self {
        }
    }
}

pub fn move_movable_object_with_player_system(
    mut physics_system: ResMut<PhysicsSystem>,
    player_query: Query<(Entity, &Player)>,
    physics_body_query: Query<&PhysicsBody>,
) {
    let (player_entity, _) = player_query.single();
    let player_physics_body = physics_body_query.get(player_entity).unwrap();

    // get all physics bodies the player is moving
    let mut moved_bodies = Vec::new();
    for (_, player) in &player_query {
        moved_bodies = player
            .pulled_objects
            .iter()
            .filter_map(|entity| physics_body_query.get(*entity).ok())
            .collect();
    }
    
    let player_velocity = physics_system.get_velocity(player_physics_body.body);
    for physics_body in moved_bodies {
        physics_system.move_body(physics_body.body, player_velocity, false);
    }
}