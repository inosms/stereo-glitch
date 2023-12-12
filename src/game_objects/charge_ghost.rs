use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};
use cgmath::InnerSpace;

use super::{position::{Position, self}, time_keeper::TimeKeeper, player::Player};

#[derive(Component)]
pub struct ChargeGhost {
    // The charge the ghost carries
    charge: f32,

    // When animating the ghost the charge value is smoothly interpolated
    // This is the current value of the interpolation
    animation_charge_value: f32,

    // The ghost can be in 'stationary' or 'following' state
    // In stationary state the ghost is not moving (just up and down)
    // In following state the ghost is following the player and is moving up and down
    follow_entity: Option<Entity>,

    // The distance the ghost keeps from the follow_entity
    following_distance: f32,

    // The current center position of the ghost
    // The ghost if floating up and down around this position
    center_position: cgmath::Vector3<f32>,

    // When in stationary state the ghost despawns after it has been picked up
    is_despawning: bool,
}

impl ChargeGhost {
    pub fn new(
        charge: f32,
        follow_entity: Option<Entity>,
        following_distance: f32,
        center_position: cgmath::Vector3<f32>,
    ) -> Self {
        Self {
            charge,
            animation_charge_value: 0.0,
            follow_entity,
            following_distance,
            center_position,
            is_despawning: false,
        }
    }

    // Returns whether the ghost is dead
    pub fn is_dead(&self) -> bool {
        self.animation_charge_value <= 0.0 && self.is_despawning
    }

    // Initiates the despawn of the ghost
    // After the despawn animation is finished the ghost is removed automatically
    pub fn initiate_despawn(&mut self) {
        self.is_despawning = true;
        self.charge = 0.0;
    }
}

// This system does the following:
// - If the ghost is dead then it is removed
// - The ghost is animated by interpolating the charge value
// - If the ghost is following an entity then it is moved towards the entity
// - The ghost is moved up and down
pub fn move_ghost_system(
    mut commands: Commands,
    time: Res<TimeKeeper>,
    mut query: Query<(&mut ChargeGhost, Entity)>,
    mut position_query: Query<&mut Position>,
    mut player_query: Query<&Player>,
) {
    for (mut ghost, entity) in query.iter_mut() {
        if ghost.is_dead() {
            commands.entity(entity).despawn();
        }

        // Animate the ghost
        let animation_delta = ghost.charge - ghost.animation_charge_value;
        let animation_speed = 5.0;
        ghost.animation_charge_value += time.delta_seconds() * animation_delta * animation_speed;

        // If the ghost is following an entity then move it towards the entity
        if let Some(follow_entity) = ghost.follow_entity {

            // sync charge with player charge
            let player = player_query.get(follow_entity).unwrap();
            ghost.charge = player.charge;

            let follow_position = position_query.get(follow_entity).unwrap();
            let distance_vec = follow_position.position - ghost.center_position;
            let distance = distance_vec.magnitude();
            let direction: cgmath::Vector3<f32> = distance_vec.normalize();
            let follow_distance = ghost.following_distance;
            if distance > follow_distance {
                let speed = 2.0;
                ghost.center_position += direction * time.delta_seconds() * speed * (distance / follow_distance);
            } else if distance.is_normal(){
                // keep the ghost at the same distance from the player
                ghost.center_position = follow_position.position - direction * follow_distance;
            }
        }

        // Move the ghost up and down
        let speed = 2.0;
        let amplitude = 0.25;
        let time = time.now() as f32;
        let offset = (time * speed).sin() * amplitude;
        let mut ghost_position = position_query.get_mut(entity).unwrap();
        ghost_position.position.z = ghost.center_position.z + offset;

        // Update the position of the ghost
        ghost_position.position.x = ghost.center_position.x;
        ghost_position.position.y = ghost.center_position.y;
        let max_scale = 0.7;
        let max_charge = 100.0;
        ghost_position.scale = cgmath::Vector3::new(1.0, 1.0, 1.0) * (ghost.animation_charge_value / max_charge).sqrt() * max_scale;
    }
}
