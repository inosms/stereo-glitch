use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use cgmath::Rotation3;
use rand::Rng;

use crate::{object_types::BlockType, physics::PhysicsSystem};

use super::{
    dust::DustParticle, model_manager::ModelManager, physics_body::PhysicsBody, player::Player,
    position::Position, renderable::Renderable, time_keeper::TimeKeeper,
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

pub fn spawn_dust_on_moving_objects_system(
    mut commands: Commands,
    time_keeper: Res<TimeKeeper>,
    physics_system: Res<PhysicsSystem>,
    query: Query<(&Position, &PhysicsBody), With<Movable>>,
    model_manager: Res<ModelManager>,
) {
    // Only move the player if we are in a physics tick
    // Otherwise the player will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    // randomly only spawn every 3th tick
    if rand::thread_rng().gen_range(0..3) != 0 {
        return;
    }

    for (position, physics_body) in &query {
        // get the player speed before applying the impulse so that we don't wiggle when running into a wall
        let player_velocity_magnitude = physics_system.get_velocity_magnitude(physics_body.body);

        if player_velocity_magnitude > 2.0 {
            let mut rng = rand::thread_rng();
            let player_position = position.position;
            let range = 0.25;
            let random_point =
                cgmath::Vector3::new(rng.gen_range(0.0..range), rng.gen_range(0.0..range), -0.5);

            let random_velocity = cgmath::Vector3::new(0.0, 0.0, 0.2);

            let random_color = cgmath::Vector3::new(
                rng.gen_range(0.8..1.0),
                rng.gen_range(0.8..1.0),
                rng.gen_range(0.8..1.0),
            );

            let pos = Position {
                position: player_position + random_point,
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: cgmath::Vector3::new(0.0, 0.0, 0.0),
                color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                grabbed_scale_factor: 1.0,
                grabbed_rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
            };

            commands.spawn((
                DustParticle::new(random_velocity, random_color, 0.1, 1.5),
                pos,
                Renderable {
                    mesh: model_manager.get_handle(&BlockType::Cube).unwrap(),
                },
            ));
        }
    }
}

#[derive(Component)]
pub struct GrabContractionAnimation {
    start_time: f64,
    duration: f64,
}

impl GrabContractionAnimation {
    pub fn new(delay: f64, duration: f64) -> Self {
        Self {
            start_time: TimeKeeper::now() + delay,
            duration,
        }
    }

    pub fn is_finished(&self) -> bool {
        TimeKeeper::now() > self.start_time + self.duration
    }

    pub fn get_scale(&self) -> f32 {
        let t = ((TimeKeeper::now() - self.start_time) / self.duration).clamp(0.0, 1.0);
        let t = t.powf(0.5);
        let t = (t - 0.5).powi(2) * 4.0;
        let t = 1.0 - t;
        1.0 - t as f32 * 0.3
    }
}

pub fn animate_grab_contraction_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &GrabContractionAnimation)>,
) {
    for (entity, mut position, grab_animation) in &mut query {
        if grab_animation.is_finished() {
            // remove the animation
            commands.entity(entity).remove::<GrabContractionAnimation>();
        }

        let scale = grab_animation.get_scale();
        position.grabbed_scale_factor = scale;
    }
}