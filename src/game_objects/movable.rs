use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Query, Res, ResMut},
};
use cgmath::Rotation3;

use crate::physics::PhysicsSystem;

use super::{
    physics_body::PhysicsBody, player::Player, position::Position, time_keeper::TimeKeeper,
};

#[derive(Component)]
pub struct Movable {}

impl Default for Movable {
    fn default() -> Self {
        Self {}
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

pub fn animate_moving_objects_system(
    time_keeper: Res<TimeKeeper>,
    player_query: Query<&Player>,
    mut movable_query: Query<(&mut Position, Entity), With<Movable>>,
) {
    if !time_keeper.is_in_fixed_tick() {
        return;
    }
    // get all physics bodies the player is moving
    let currently_moving_entities = player_query
        .iter()
        .next()
        .unwrap()
        .pulled_objects
        .iter()
        .collect::<HashSet<_>>();

    for (mut position, entity) in &mut movable_query {
        let desired_scale = if currently_moving_entities.contains(&entity) {
            0.8
        } else {
            1.0
        };

        // smoothly animate the scale
        let scale_difference = desired_scale - position.grabbed_scale_factor;
        let animation_speed = 20.0;
        position.grabbed_scale_factor +=
            scale_difference * animation_speed * time_keeper.delta_seconds();

        let wobble_scale = (1.0 - position.grabbed_scale_factor) as f64;
        let wobble_speed = 25.0;
        let x_wobble = (TimeKeeper::now() * wobble_speed).sin() * wobble_scale * 40.0;
        let y_wobble = (TimeKeeper::now() * wobble_speed).cos() * wobble_scale * 40.0;
        position.grabbed_rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0 as f32))
                * cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_x(),
                    cgmath::Deg(x_wobble as f32),
                )
                * cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(y_wobble as f32),
                );
    }
}
