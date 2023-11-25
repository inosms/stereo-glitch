use std::ops::Add;

use bevy_ecs::system::Resource;
use rapier3d::{control::KinematicCharacterController, prelude::*};

use crate::{game::Position, level_loader::BlockPhysicsType};

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
    query_pipeline: QueryPipeline,

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
        let query_pipeline = QueryPipeline::new();
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
            query_pipeline,

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

        self.query_pipeline.update(&self.rigid_body_set, &self.collider_set);
    }

    pub fn add_object(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        x_extent: f32,
        y_extent: f32,
        z_extent: f32,
        block_physics_type: BlockPhysicsType,
    ) -> RigidBodyHandle {
        let rigid_body = match block_physics_type {
            BlockPhysicsType::Static => RigidBodyBuilder::fixed(),
            BlockPhysicsType::Kinematic => RigidBodyBuilder::kinematic_position_based(),
            BlockPhysicsType::Dynamic => RigidBodyBuilder::dynamic(),
        }
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

    pub fn move_body(&mut self, body_handle: RigidBodyHandle, direction: cgmath::Vector3<f32>) {
        let body = self.rigid_body_set.get(body_handle).unwrap();
        let collider_handle = body.colliders().first().unwrap().clone();
        let collider = self.collider_set.get(collider_handle).unwrap();
        let shape = collider.shape();
        let pos = body.position();
        let mass = body.mass();

        let desired_translation = vector![direction.x, direction.y, direction.z];
        let mut character_controller = KinematicCharacterController::default();
        character_controller.up = Vector::z_axis();

        let mut collisions = vec![];
        let corrected_movement = character_controller.move_shape(
            self.integration_parameters.dt,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            shape,
            pos,
            desired_translation,
            QueryFilter::default().exclude_rigid_body(body_handle), // .exclude_collider(collider_handle),
            |c| collisions.push(c),
        );

        for collision in &collisions {
            character_controller.solve_character_collision_impulses(
                self.integration_parameters.dt,
                &mut self.rigid_body_set,
                &self.collider_set,
                &self.query_pipeline,
                shape,
                mass,
                collision,
                QueryFilter::new().exclude_rigid_body(body_handle), // .exclude_collider(collider_handle),
            )
        }

        let body = self.rigid_body_set.get_mut(body_handle).unwrap();
        body.set_next_kinematic_translation(body.translation() + corrected_movement.translation);
    }
}
