use bevy_ecs::{component::Component, system::{Commands, Res, Query}, entity::Entity};
use cgmath::{Vector3, Vector4, Quaternion, Rotation3};
use rand::Rng;

use crate::{stereo_camera::{self, StereoCamera}, object_types::BlockType};

use super::{renderable::Renderable, time_keeper::TimeKeeper, model_manager::ModelManager, position::Position, constants::TICKS_PER_SECOND};
#[derive(Component)]
pub struct DustParticle {
    velocity: Vector3<f32>,
    color: Vector3<f32>,
    size: f32,
    life: f32,
    life_time: f32,
}

impl DustParticle {
    pub fn new(velocity: Vector3<f32>, color: Vector3<f32>, size: f32, life: f32) -> Self {
        Self {
            velocity,
            color,
            size,
            life,
            life_time: life,
        }
    }
}

pub fn animate_dust_particle_system(
    mut commands: Commands,
    time_keeper: Res<TimeKeeper>,
    mut query: Query<(Entity, &mut DustParticle, &mut Position)>,
) {
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    for (entity, mut dust_particle, mut position) in &mut query {
        dust_particle.life -= 1.0 / TICKS_PER_SECOND as f32;
        if dust_particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let t = dust_particle.life / dust_particle.life_time;
        let t = (t - 0.5).powi(2) * 4.0;
        let t = 1.0 - t;
        let t = t.powf(0.5);

        let new_position = position.position + dust_particle.velocity * 1.0 / TICKS_PER_SECOND as f32;
        position.position = new_position;
        position.scale = Vector3::new(dust_particle.size, dust_particle.size, dust_particle.size) * t;
        position.color = Vector4::new(dust_particle.color.x, dust_particle.color.y, dust_particle.color.z, 1.0);
        // slowly rotate the dust particle around the z axis
        position.rotation = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), cgmath::Deg(dust_particle.life * 360.0));
    }
}