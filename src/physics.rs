use bevy_ecs::system::Resource;
use cgmath::{EuclideanSpace, InnerSpace, One};
use rapier3d::{
    crossbeam,
    na::{Point3, Vector3},
    prelude::*,
};

use crate::{
    game_objects::position::Position,
    object_types::{Block, BlockType, BoxType, LinearEnemyDirection},
};

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
    event_handler: ChannelEventCollector,
    query_pipeline: QueryPipeline,

    collision_recv: crossbeam::channel::Receiver<CollisionEvent>,

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

        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, _contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

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
            collision_recv,
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

        self.query_pipeline
            .update(&self.rigid_body_set, &self.collider_set);
    }

    pub fn poll_collision_events(&mut self) -> Option<CollisionEvent> {
        self.collision_recv.try_recv().ok()
    }

    pub fn add_object(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        x_extent: f32,
        y_extent: f32,
        z_extent: f32,
        block: &Block,
    ) -> (RigidBodyHandle, Option<ColliderHandle>) {
        let rigid_body: RigidBody = match &block {
            Block::FloorNormal
            | Block::Door(_)
            | Block::Wall
            | Block::Trigger
            | Block::Charge
            | Block::StaticEnemy
            | Block::Checkpoint
            | Block::FireworkEmitter
            | Block::Goal(_) => RigidBodyBuilder::fixed(),
            Block::Empty => unreachable!(),
            Block::Player => {
                // make the player heaver to avoid bouncing
                RigidBodyBuilder::dynamic().locked_axes(LockedAxes::ROTATION_LOCKED)
            }
            Block::LinearEnemy(LinearEnemyDirection::XAxis) => RigidBodyBuilder::dynamic()
                .locked_axes(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED),
            Block::LinearEnemy(LinearEnemyDirection::YAxis) => RigidBodyBuilder::dynamic()
                .locked_axes(LockedAxes::TRANSLATION_LOCKED_X | LockedAxes::ROTATION_LOCKED),
            Block::Box(BoxType::Free) => RigidBodyBuilder::dynamic()
                .additional_mass(5.0)
                .linear_damping(0.5),
            Block::Box(BoxType::XAxis) => RigidBodyBuilder::dynamic()
                .locked_axes(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED)
                .additional_mass(5.0)
                .linear_damping(0.5),
            Block::Box(BoxType::YAxis) => RigidBodyBuilder::dynamic()
                .locked_axes(LockedAxes::TRANSLATION_LOCKED_X | LockedAxes::ROTATION_LOCKED)
                .additional_mass(5.0)
                .linear_damping(0.5),
            Block::Box(BoxType::RotationFixed) => RigidBodyBuilder::dynamic()
                .locked_axes(LockedAxes::ROTATION_LOCKED)
                .additional_mass(5.0)
                .linear_damping(0.5),
        }
        .ccd_enabled(true)
        .translation(vector![x, y, z])
        .build();
        let body_handle = self.rigid_body_set.insert(rigid_body);

        let collider = match &block {
            Block::FloorNormal
            | Block::Door(_)
            | Block::Wall
            | Block::StaticEnemy
            | Block::Box(_) => Some(ColliderBuilder::cuboid(x_extent, y_extent, z_extent).build()),
            Block::Player | Block::LinearEnemy(_) => {
                Some(ColliderBuilder::capsule_z(z_extent / 2.0, x_extent).build())
            }
            Block::Empty
            | Block::Charge
            | Block::Trigger
            | Block::Goal(_)
            | Block::Checkpoint
            | Block::FireworkEmitter => None,
        };
        let collider_handle = collider.map(|collider| {
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set)
        });

        (body_handle, collider_handle)
    }

    pub fn add_sensor_collider(
        &mut self,
        body_handle: RigidBodyHandle,
        x_extent: f32,
        y_extent: f32,
        z_extent: f32,
        x_offset: f32,
        y_offset: f32,
        z_offset: f32,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(x_extent, y_extent, z_extent)
            .sensor(true)
            .translation(vector![x_offset, y_offset, z_offset])
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_FIXED,
            )
            .build();
        self.collider_set
            .insert_with_parent(collider, body_handle, &mut self.rigid_body_set)
    }

    pub fn set_rigid_body_state(&mut self, body_handle: RigidBodyHandle, is_active: bool) {
        let body = self.rigid_body_set.get_mut(body_handle).unwrap();
        body.set_enabled(is_active);
    }

    pub fn get_position(&self, body_handle: RigidBodyHandle) -> Position {
        let body = &self.rigid_body_set[body_handle];
        let pos = body.translation().clone();
        let rot = body.rotation().clone();

        Position {
            position: cgmath::Vector3::new(pos.x, pos.y, pos.z),
            rotation: cgmath::Quaternion::new(rot.w, rot.i, rot.j, rot.k),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
            grabbed_scale_factor: 1.0,
            grabbed_rotation: cgmath::Quaternion::one(),
        }
    }

    pub fn get_velocity_magnitude(&self, body_handle: RigidBodyHandle) -> f32 {
        let body = &self.rigid_body_set[body_handle];
        body.linvel().magnitude()
    }

    pub fn get_velocity(&self, body_handle: RigidBodyHandle) -> cgmath::Vector3<f32> {
        let body = &self.rigid_body_set[body_handle];
        cgmath::Vector3::new(body.linvel().x, body.linvel().y, body.linvel().z)
    }

    pub fn get_user_data(&self, collider_handle: ColliderHandle) -> Option<u128> {
        self.collider_set
            .get(collider_handle)
            .map(|collider| collider.user_data)
    }

    pub fn set_user_data(&mut self, collider_handle: ColliderHandle, user_data: u128) {
        let collider = self.collider_set.get_mut(collider_handle).unwrap();
        collider.user_data = user_data;
    }

    pub fn move_body(
        &mut self,
        body_handle: RigidBodyHandle,
        desired_velocity: cgmath::Vector3<f32>,
        rotate_in_direction_of_movement: bool,
    ) {
        // move body using impulses
        let body = self.rigid_body_set.get_mut(body_handle).unwrap();
        let mass = body.mass();

        let current_velocity = body.linvel().clone();
        let velocity_change = vector![
            desired_velocity.x - current_velocity.x,
            desired_velocity.y - current_velocity.y,
            // don't change z velocity
            // otherwise gravity will stop working properly
            0.0,
        ];
        let impulse = velocity_change * mass;

        body.apply_impulse(impulse, true);

        // if actually moving

        if rotate_in_direction_of_movement && desired_velocity.magnitude2() > 0.001 {
            // The initial alignment of the player is to look along the negative y axis.
            // From this compute the angle of movement around the positive z axis.
            // This is used to rotate the player to face the direction of movement.

            // We don't care about the z direction
            let direction_norm =
                Vector3::new(desired_velocity.x, desired_velocity.y, 0.0).normalize();
            let zero_rotation_direction = Vector3::new(0.0, -1.0, 0.0);
            let axis_of_rotation =
                rapier3d::na::UnitVector3::try_new(Vector3::new(0.0, 0.0, 1.0), 0.1).unwrap();

            // https://math.stackexchange.com/questions/878785/how-to-find-an-angle-in-range0-360-between-2-vectors
            let determinant =
                (direction_norm.cross(&zero_rotation_direction)).dot(&axis_of_rotation);
            let dot = zero_rotation_direction.dot(&direction_norm);
            let angle_of_movement = determinant.atan2(dot);

            let next_rotation = rapier3d::na::UnitQuaternion::from_axis_angle(
                &rapier3d::na::Vector3::z_axis(),
                -angle_of_movement,
            );

            let current_rotation = body.rotation().clone();
            let lerped_rotation = current_rotation.slerp(&next_rotation, 0.5);

            body.set_rotation(lerped_rotation, true);
        }
    }
}
