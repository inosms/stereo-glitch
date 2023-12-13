use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res, ResMut},
};
use cgmath::{InnerSpace, Vector3};


use super::{
    player::Player,
    position::{self, Position},
    time_keeper::TimeKeeper, sensor::Sensor, constants::TICKS_PER_SECOND, renderable::Renderable, glitch_area::GlitchAreaVisibility,
};

#[derive(Component)]
pub struct Charge {
    cooldown_left: f32,

    spawned_ghost: Option<Entity>,
}

impl Charge {
    pub fn new() -> Self {
        Self {
            cooldown_left: 0.0,
            spawned_ghost: None,
        }
    }
}

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
    // Creates a new ghost that follows an entity
    pub fn new_following(
        follow_entity: Entity,
        following_distance: f32,
        initial_position: cgmath::Vector3<f32>,
    ) -> Self {
        Self {
            charge: 0.0,
            animation_charge_value: 0.0,
            follow_entity: Some(follow_entity),
            following_distance,
            center_position: initial_position,
            is_despawning: false,
        }
    }

    // Creates a new ghost that is stationary
    pub fn new_stationary(charge: f32, initial_position: cgmath::Vector3<f32>) -> Self {
        Self {
            charge,
            animation_charge_value: 0.0,
            follow_entity: None,
            following_distance: 0.0,
            center_position: initial_position,
            is_despawning: false,
        }
    }

    // Returns whether the ghost is dead
    pub fn is_dead(&self) -> bool {
        self.animation_charge_value <= 0.05 && self.is_despawning
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
pub fn move_charge_ghost_system(
    mut commands: Commands,
    time: Res<TimeKeeper>,
    mut query: Query<(&mut ChargeGhost, Entity)>,
    mut position_query: Query<&mut Position>,
    mut player_query: Query<&Player>,
) {
    for (mut ghost, entity) in query.iter_mut() {
        if ghost.is_dead() {
            commands.entity(entity).despawn();
            continue;
        }

        // Animate the ghost
        let animation_delta = (ghost.charge - ghost.animation_charge_value).min(1.0);
        let animation_speed = 30.0;
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
                ghost.center_position +=
                    direction * time.delta_seconds() * speed * (distance / follow_distance);
            } else if distance.is_normal() {
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

        let is_in_following_mode = ghost.follow_entity.is_some();
        let max_scale = 0.7;
        let min_scale = if is_in_following_mode { 0.2 } else { 0.0 };
        let max_charge = 100.0;
        ghost_position.scale = cgmath::Vector3::new(1.0, 1.0, 1.0)
            * ((ghost.animation_charge_value / max_charge).sqrt() * max_scale).max(min_scale);
    }
}

pub fn charge_recharge_system(
    mut commands: Commands,
    mut time_keeper: ResMut<TimeKeeper>,
    mut query: Query<(&mut Charge, &Sensor, Entity, &Position)>,
    mut player_query: Query<&mut Player>,
    mut ghost_query: Query<&mut ChargeGhost>,
    renderable_query: Query<&Renderable>,
) {
    // Only recharge charge if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    let charge_added = 20.0;

    for (mut charge, sensor, sensor_entity, position) in &mut query {
        let triggering_player_entity = sensor
            .triggered_by
            .iter()
            .filter(|&entity| player_query.get_mut(*entity).is_ok())
            .collect::<Vec<_>>();

        let triggered_by_player = !triggering_player_entity.is_empty();
        let can_recharge = charge.cooldown_left <= 0.0;
        if triggered_by_player && can_recharge {
            for player_entity in triggering_player_entity {
                if let Ok(mut player) = player_query.get_mut(*player_entity) {
                    player.charge = (player.charge.max(0.0) + charge_added).min(100.0);
                    if let Ok(mut ghost) = ghost_query.get_mut(charge.spawned_ghost.unwrap()) {
                        ghost.initiate_despawn();
                    }
                    charge.spawned_ghost = None;
                }
            }

            charge.cooldown_left = 10.0;
        } else {
            charge.cooldown_left -= 1.0 / TICKS_PER_SECOND as f32;
        }

        if charge.cooldown_left <= 0.0 && charge.spawned_ghost.is_none() {
            let ghost = commands.spawn((
                Position {
                    position: Vector3::new(
                        position.position.x,
                        position.position.y,
                        position.position.z + 0.5,
                    ),
                    rotation: position.rotation,
                    scale: Vector3::new(0.0, 0.0, 0.0),
                },
                Renderable {
                    mesh: renderable_query.get(sensor_entity).unwrap().mesh.clone(),
                },
                ChargeGhost::new_stationary(charge_added, position.position),
            ));
            charge.spawned_ghost = Some(ghost.id());
        }
    }
}



// When the player is in a glitch area deplete the charge over time
// If the charge reaches 0 the player dies
pub fn player_charge_depletion_system(
    mut time_keeper: ResMut<TimeKeeper>,
    mut player_query: Query<(&mut Player, &Position)>,
    mut glitch_area_visibility: ResMut<GlitchAreaVisibility>,
) {
    // Only deplete charge if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    let deplete_per_second = 1.0;
    let deplete_per_tick = deplete_per_second / TICKS_PER_SECOND as f32;

    for (mut player, pos) in &mut player_query {
        let player_x_y_cell = pos.get_cell();
        let is_in_glitch_area = glitch_area_visibility
            .glitch_cells
            .contains(&player_x_y_cell);

        if is_in_glitch_area {
            player.charge -= deplete_per_tick;
        }

        let player_charge = if player.charge > 10.0 {
            1.0
        } else if player.charge > 0.0 {
            0.2
        } else {
            0.0
        };
        // smooth interpolation between 0 and 1
        let alpha = 0.99;
        glitch_area_visibility.visibility =
            glitch_area_visibility.visibility * alpha + player_charge * (1.0 - alpha);
    }
}
