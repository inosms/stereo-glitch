use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};
use cgmath::InnerSpace;
use rand::Rng;

use crate::model;

use super::{
    model_manager::ModelManager, position::Position, renderable::Renderable,
    time_keeper::TimeKeeper,
};

#[derive(Component)]
pub struct FireworkParticle {
    color: cgmath::Vector4<f32>,
    velocity: cgmath::Vector3<f32>,
    gravity_factor: f32,
    lifetime: f32,
    max_lifetime: f32,
    scale: f32,
}

impl FireworkParticle {
    pub fn new(
        color: cgmath::Vector4<f32>,
        velocity: cgmath::Vector3<f32>,
        gravity_factor: f32,
        lifetime: f32,
        scale: f32,
    ) -> Self {
        Self {
            color,
            velocity,
            gravity_factor: gravity_factor.powf(3.0),
            lifetime,
            max_lifetime: lifetime,
            scale,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.velocity += cgmath::Vector3::new(0.0, 0.0, -1.0) * dt * self.gravity_factor;
        self.velocity *= 0.995;
        self.lifetime -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }

    pub fn scale(&self) -> cgmath::Vector3<f32> {
        let scale = self.lifetime / self.max_lifetime;
        let scale = scale.powf(0.2);
        cgmath::Vector3::new(scale, scale, scale) * self.scale
    }
}

#[derive(Component)]
pub struct FireworkEmitter {
    countdown: f32,
}

pub fn firework_particle_system(
    time_keeper: Res<TimeKeeper>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut FireworkParticle, &mut Position)>,
) {
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    for (entity, mut particle, mut position) in particles.iter_mut() {
        if !particle.is_alive() {
            commands.entity(entity).despawn();
            continue;
        }

        particle.update(time_keeper.delta_seconds());
        position.position += particle.velocity * time_keeper.delta_seconds();
        position.color = particle.color;
        position.scale = particle.scale();
    }
}

impl FireworkEmitter {
    pub fn new() -> Self {
        Self { countdown: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.countdown -= dt;
    }

    pub fn is_ready(&self) -> bool {
        self.countdown <= 0.0
    }

    pub fn reset(&mut self) {
        self.countdown = rand::thread_rng().gen_range(0.2..10.0);
    }
}

pub fn firework_emitter_system(
    time_keeper: Res<TimeKeeper>,
    model_manager: Res<ModelManager>,
    mut commands: Commands,
    mut firework_emitters: Query<(&mut FireworkEmitter, &Position)>,
) {
    for (mut emitter, position) in firework_emitters.iter_mut() {
        emitter.update(time_keeper.delta_seconds());
        if emitter.is_ready() {
            emitter.reset();
            let mut rng = rand::thread_rng();

            let random_center = cgmath::Vector3::new(
                rng.gen_range(-0.5..0.5),
                rng.gen_range(-0.5..0.5),
                rng.gen_range(3.0..5.0),
            );

            let particle_count = 64;
            let trail_length = 16;
            let colors = [
                cgmath::Vector3::new(235.0 / 255.0, 137.0 / 255.0, 52.0 / 255.0),
                cgmath::Vector3::new(22.0 / 255.0, 245.0 / 255.0, 200.0 / 255.0),
                cgmath::Vector3::new(210.0 / 255.0, 34.0 / 255.0, 245.0 / 255.0),
                cgmath::Vector3::new(34.0 / 255.0, 245.0 / 255.0, 122.0 / 255.0),
                cgmath::Vector3::new(101.0 / 255.0, 34.0 / 255.0, 245.0 / 255.0),
            ];
            let random_color = colors[rng.gen_range(0..colors.len())] * 4.0; // make the color brighter
            let max_velocity = rng.gen_range(1.5..4.0);
            let max_lifetime = rng.gen_range(2.0..4.0);
            for _ in 0..particle_count {
                // sample a velocity randomly in a sphere.
                // for this sample three gaussian random variables
                // and normalize the resulting vector
                let x: f32 = rng.sample(rand_distr::StandardNormal);
                let y: f32 = rng.sample(rand_distr::StandardNormal);
                let z: f32 = rng.sample(rand_distr::StandardNormal);

                let velocity = cgmath::Vector3::new(x, y, z).normalize() * max_velocity;

                // the trail is a series of particles trailing the main particle with decreasing velocity and brightness
                for trail in 0..trail_length {
                    let falloff = 0.92_f32.powf(trail as f32);
                    let mut pos = Position::default();
                    pos.position = position.position + random_center;
                    pos.color = cgmath::Vector4::new(
                        random_color.x * falloff.powf(4.0),
                        random_color.y * falloff.powf(4.0),
                        random_color.z * falloff.powf(4.0),
                        1.0,
                    );
                    commands.spawn((
                        FireworkParticle::new(
                            pos.color,
                            velocity * falloff,
                            falloff,
                            max_lifetime * falloff.powf(0.5),
                            0.1 * falloff,
                        ),
                        pos,
                        Renderable {
                            mesh: model_manager
                                .get_handle(&crate::object_types::BlockType::Cube)
                                .unwrap(),
                        },
                    ));
                }
            }
        }
    }
}
