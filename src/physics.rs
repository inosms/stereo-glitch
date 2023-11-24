use bevy_ecs::system::Resource;
use rapier3d::prelude::*;

use crate::game::Position;

#[derive(Resource)]
pub struct PhysicsSystem {
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    gravity: Vector<f32>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let gravity = vector![0.0, 0.0, -9.81];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            physics_hooks,
            event_handler,

            rigid_body_set,
            collider_set,

            gravity,
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn add_immovable(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        x_extent: f32,
        y_extent: f32,
        z_extent: f32,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::cuboid(x_extent, y_extent, z_extent).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        body_handle
    }

    pub fn add_movable(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        x_extent: f32,
        y_extent: f32,
        z_extent: f32,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::cuboid(x_extent, y_extent, z_extent).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        body_handle
    }

    pub fn get_position(&self, body_handle: RigidBodyHandle) -> Position {
        let body = &self.rigid_body_set[body_handle];
        let pos = body.translation().clone();
        let rot = body.rotation().clone();

        Position {
            position: cgmath::Vector3::new(pos.x, pos.y, pos.z),
            rotation: cgmath::Quaternion::new(rot.i, rot.j, rot.k, rot.w),
        }
    }
}

// fn main() {
//     let mut rigid_body_set = RigidBodySet::new();
//     let mut collider_set = ColliderSet::new();

//     /* Create the ground. */
//     let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
//     collider_set.insert(collider);

//     /* Create the bounding ball. */
//     let rigid_body = RigidBodyBuilder::dynamic()
//         .translation(vector![0.0, 10.0, 0.0])
//         .build();
//     let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
//     let ball_body_handle = rigid_body_set.insert(rigid_body);
//     collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

//     /* Create other structures necessary for the simulation. */
//     let gravity = vector![0.0, -9.81, 0.0];
//     let integration_parameters = IntegrationParameters::default();
//     let mut physics_pipeline = PhysicsPipeline::new();
//     let mut island_manager = IslandManager::new();
//     let mut broad_phase = BroadPhase::new();
//     let mut narrow_phase = NarrowPhase::new();
//     let mut impulse_joint_set = ImpulseJointSet::new();
//     let mut multibody_joint_set = MultibodyJointSet::new();
//     let mut ccd_solver = CCDSolver::new();
//     let physics_hooks = ();
//     let event_handler = ();

//     /* Run the game loop, stepping the simulation once per frame. */
//     for _ in 0..200 {
//         physics_pipeline.step(
//             &gravity,
//             &integration_parameters,
//             &mut island_manager,
//             &mut broad_phase,
//             &mut narrow_phase,
//             &mut rigid_body_set,
//             &mut collider_set,
//             &mut impulse_joint_set,
//             &mut multibody_joint_set,
//             &mut ccd_solver,
//             None,
//             &physics_hooks,
//             &event_handler,
//         );

//         let ball_body = &rigid_body_set[ball_body_handle];
//         println!("Ball altitude: {}", ball_body.translation().y);
//     }
// }
