use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use cgmath::Rotation3;
use rand::Rng;

use crate::object_types::{Id, BlockType};

use super::{
    constants::TICKS_PER_SECOND, game_system_commands::GameSystemCommands, player::Player,
    position::Position, sensor::Sensor, time_keeper::TimeKeeper, model_manager, renderable::Renderable,
};

#[derive(Component)]
pub struct Checkpoint {
    id: Id,
    spawn_cool_down: f32,
    is_active: bool,
}

impl Checkpoint {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            spawn_cool_down: 0.0,
            is_active: false,
        }
    }
}

pub fn set_checkpoint_system(
    mut game_system_commands: ResMut<GameSystemCommands>,
    mut checkpoint_sensor_query: Query<(&mut Checkpoint, &Sensor)>,
    player_query: Query<Entity, With<Player>>,
) {
    let mut triggered_id = None;
    for (checkpoint, sensor) in checkpoint_sensor_query.iter() {
        for triggered_by in &sensor.triggered_by {
            if player_query.get(*triggered_by).is_ok() {
                game_system_commands.set_checkpoint(checkpoint.id.clone());
                triggered_id = Some(checkpoint.id.clone());
            }
        }
    }

    if let Some(id) = triggered_id {
        for (mut checkpoint, _) in checkpoint_sensor_query.iter_mut() {
            checkpoint.is_active = checkpoint.id == id;
        }
    }
}

#[derive(Component)]
pub struct CheckpointParticle {
    start_time: f32,
    lifetime: f32,
    start_z: f32,
}

pub fn spawn_checkpoint_particle_system(
    mut commands: Commands,
    time: Res<TimeKeeper>,
    model_manager: Res<model_manager::ModelManager>,
    mut query: Query<(&mut Checkpoint, &Position)>,
) {
    if !time.is_in_fixed_tick() {
        return;
    }

    for (mut checkpoint, position) in &mut query {
        if checkpoint.spawn_cool_down > 0.0 {
            checkpoint.spawn_cool_down -= 1.0 / TICKS_PER_SECOND as f32;
            continue;
        }

        checkpoint.spawn_cool_down = 0.2;

        let mut rng = rand::thread_rng();
        let mut pos = Position::default();
        pos.position = position.position;
        pos.position.x = pos.position.x + rng.gen::<f32>() * 1.0 - 0.5;
        pos.position.y = pos.position.y + rng.gen::<f32>() * 1.0 - 0.5;
        pos.scale = cgmath::Vector3::new(0.0, 0.0, 0.0);
        if checkpoint.is_active {
            pos.color = cgmath::Vector4::new(3.0 / 255.0, 252.0 / 255.0, 202.0 / 255.0, 1.0);
        } else {
            pos.color = cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0);
        }
        commands.spawn((
            CheckpointParticle {
                start_time: TimeKeeper::now() as f32,
                lifetime: 2.0 + rng.gen::<f32>() * 1.0,
                start_z: pos.position.z,
            },
            pos,
            Renderable { mesh: model_manager.get_handle(&BlockType::Checkpoint).unwrap() },
        ));
    }
}


pub fn animate_checkpoint_particles_system(
    mut commands: Commands,
    time: Res<TimeKeeper>,
    mut query: Query<(&mut CheckpointParticle, &mut Position, Entity)>,
) {
    if !time.is_in_fixed_tick() {
        return;
    }

    for (mut checkpoint_particle, mut position, entity) in &mut query {
        let diff_time = TimeKeeper::now() as f32 - checkpoint_particle.start_time;
        let t = diff_time / checkpoint_particle.lifetime;
        if t > 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // scale is 0 at t=1, 1 at t=0.5, 0 at t=0
        let scale = 1.0 - (t - 0.5).abs() * 2.0;
        let scale = scale.powf(0.5);
        let scale = scale * 0.2;
        position.scale = cgmath::Vector3::new(scale, scale, scale);
        position.position.z = checkpoint_particle.start_z + diff_time;
        
        // slowly rotate the particle
        let rotation_speed = 1.0 * 1.0 / checkpoint_particle.lifetime;
        position.rotation = cgmath::Quaternion::from_axis_angle(
            cgmath::Vector3::unit_z(),
            cgmath::Deg(diff_time * rotation_speed * 360.0),
        );
    }
}