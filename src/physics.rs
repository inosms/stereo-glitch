use bevy_ecs::system::Resource;
use cgmath::{EuclideanSpace, InnerSpace};
use rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterController},
    crossbeam,
    na::{UnitVector3, Vector3, Isometry3, Translation3},
    prelude::*,
};

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

        let gravity = vector![0.0, 0.0, -39.81];
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
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
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
        block_physics_type: BlockPhysicsType,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rigid_body = match block_physics_type {
            BlockPhysicsType::Static => RigidBodyBuilder::fixed(),
            BlockPhysicsType::Kinematic => RigidBodyBuilder::kinematic_position_based(),
            BlockPhysicsType::Dynamic => RigidBodyBuilder::dynamic(),
        }
        .ccd_enabled(true)
        .translation(vector![x, y, z])
        .build();
        let collider = ColliderBuilder::cuboid(x_extent, y_extent, z_extent).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        let collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
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

    pub fn set_collider_state(&mut self, collider_handle: ColliderHandle, is_active: bool) {
        let collider = self.collider_set.get_mut(collider_handle).unwrap();
        collider.set_enabled(is_active);
    }

    pub fn set_collider_do_not_collide_with_kinetic(
        &mut self,
        collider_handle: ColliderHandle,
        collide_with_kinetic: bool,
    ) {
        let collider = self.collider_set.get_mut(collider_handle).unwrap();
        let mut active_collision_types = collider.active_collision_types();
        if collide_with_kinetic {
            active_collision_types |= ActiveCollisionTypes::DYNAMIC_KINEMATIC;
        } else {
            active_collision_types &= !ActiveCollisionTypes::DYNAMIC_KINEMATIC;
        }
        collider.set_active_collision_types(active_collision_types);
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
        direction: cgmath::Vector3<f32>,
        exclude_handles: &[ColliderHandle],
    ) {
        let body = self.rigid_body_set.get(body_handle).unwrap();
        let collider_handle = body.colliders().first().unwrap().clone();
        let collider = self.collider_set.get(collider_handle).unwrap();
        let shape = collider.shape();
        let pos = body.position();
        let mass = body.mass();

        let desired_translation = vector![direction.x, direction.y, direction.z];
        let gravity = vector![0.0, 0.0, -0.2];
        let desired_translation = desired_translation + gravity;
        let mut character_controller = KinematicCharacterController::default();
        character_controller.up = Vector::z_axis();
        character_controller.offset = CharacterLength::Absolute(0.11);
        character_controller.autostep = Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(0.2),
            min_width: CharacterLength::Absolute(0.0),
            include_dynamic_bodies: true,
        });

        let mut query_filter = QueryFilter::default()
            .exclude_rigid_body(body_handle)
            .exclude_collider(collider_handle);
        for handle in exclude_handles {
            query_filter = query_filter.exclude_collider(*handle);
        }

        let mut collisions = vec![];
        let corrected_movement = character_controller.move_shape(
            self.integration_parameters.dt,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            shape,
            pos,
            desired_translation,
            query_filter,
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
                query_filter,
            )
        }
        let body = self.rigid_body_set.get_mut(body_handle).unwrap();

        // if actually moving
        let next_rotation = if direction.magnitude2() > 0.001 {
            // The initial alignment of the player is to look along the negative y axis.
            // From this compute the angle of movement around the positive z axis.
            // This is used to rotate the player to face the direction of movement.

            // We don't care about the z direction
            let direction_norm = Vector3::new(direction.x, direction.y, 0.0).normalize();
            let zero_rotation_direction = Vector3::new(0.0, -1.0, 0.0);
            let axis_of_rotation =
                rapier3d::na::UnitVector3::try_new(Vector3::new(0.0, 0.0, 1.0), 0.1).unwrap();

            // https://math.stackexchange.com/questions/878785/how-to-find-an-angle-in-range0-360-between-2-vectors
            let determinant =
                (direction_norm.cross(&zero_rotation_direction)).dot(&axis_of_rotation);
            let dot = zero_rotation_direction.dot(&direction_norm);
            let angle_of_movement = determinant.atan2(dot);

            log::info!("angle_of_movement {}", angle_of_movement);
            let rotation = rapier3d::na::UnitQuaternion::from_axis_angle(
                &rapier3d::na::Vector3::x_axis(), // for some reason this must be x not y otherwise the rotation will be weird (???)
                angle_of_movement,
            );
            rotation
        } else {
            body.rotation().clone()
        };

        let next_translation = body.translation() + corrected_movement.translation;
        let next_position = Isometry3::from_parts(
            Translation3::new(next_translation.x, next_translation.y, next_translation.z),
            next_rotation,
        );
        body.set_next_kinematic_position(next_position);
    }

    pub fn build_fixed_joint(
        &mut self,
        body1_handle: RigidBodyHandle,
        body2_handle: RigidBodyHandle,
        anchor1: cgmath::Vector3<f32>,
        anchor2: cgmath::Vector3<f32>,
    ) -> ImpulseJointHandle {
        let joint = FixedJointBuilder::new()
            .local_anchor1(point![anchor1.x, anchor1.y, anchor1.z])
            .local_anchor2(point![anchor2.x, anchor2.y, anchor2.z])
            .build();
        self.impulse_joint_set
            .insert(body1_handle, body2_handle, joint, true)
    }

    pub fn remove_joint(&mut self, joint_handle: ImpulseJointHandle) {
        self.impulse_joint_set.remove(joint_handle, true);
    }
}
