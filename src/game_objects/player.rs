use bevy_ecs::{component::Component, entity::Entity, system::{ResMut, Query, Res}, query::With};
use cgmath::{InnerSpace, Rotation3};

use crate::{physics::PhysicsSystem, stereo_camera::StereoCamera};

use super::{time_keeper::TimeKeeper, constants::TICKS_PER_SECOND, input::Input, physics_body::PhysicsBody, position::Position};

#[derive(Component)]
pub struct Player {
    pub dead: bool,

    // the objects the player is currently pulling
    pub pulled_objects: Vec<Entity>,

    pub charge: f32,
}


pub fn move_player_system(
    mut physics_system: ResMut<PhysicsSystem>,
    mut input: ResMut<Input>,
    camera: Res<StereoCamera>,
    time_keeper: Res<TimeKeeper>,
    mut query: Query<(&mut Position, &PhysicsBody), With<Player>>,
) {
    // Only move the player if we are in a physics tick
    // Otherwise the player will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    if input.player_paralized_cooldown > 0.0 {
        input.player_paralized_cooldown -= 1.0 / TICKS_PER_SECOND as f32;
        return;
    }

    let requested_movement = input
        .player_movement
        .take()
        .unwrap_or(cgmath::Vector3::new(0.0, 0.0, 0.0));
    let camera_look_direction = camera.get_camera_view_direction_projected_to_ground();

    // Get a matrix that rotates the world y axis to the camera look direction
    // We need this to transform the requested movement vector so that the player moves in the direction the camera is looking
    let camera_look_direction_rotation_matrix = cgmath::Matrix3::from_cols(
        camera_look_direction
            .cross(cgmath::Vector3::unit_z())
            .normalize(),
        camera_look_direction,
        cgmath::Vector3::unit_z(),
    );

    let mut direction = requested_movement;
    if direction.magnitude() > 1.0 {
        direction = direction.normalize();
    }
    let player_max_speed = 7.0;
    let direction = camera_look_direction_rotation_matrix * direction * player_max_speed;

    for (mut position, physics_body) in &mut query {
        
        // get the player speed before applying the impulse so that we don't wiggle when running into a wall
        let player_velocity_magnitude = physics_system.get_velocity_magnitude(physics_body.body);

        physics_system.move_body(physics_body.body, direction, true);

        let wobble_scale = player_velocity_magnitude.sqrt() as f64 / player_max_speed as f64;
        let wobble_speed = 25.0;
        let x_wobble = (TimeKeeper::now() * wobble_speed).sin() * wobble_scale * 30.0;
        let y_wobble = (TimeKeeper::now() * wobble_speed).cos() * wobble_scale * 30.0;
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
