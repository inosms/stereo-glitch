use std::collections::HashMap;

use bevy_ecs::prelude::*;
use cgmath::{InnerSpace, Rotation3, EuclideanSpace, Point3};

use crate::{
    level_loader::{BlockType, Cell},
    mesh::{Handle, Mesh},
    physics::PhysicsSystem, stereo_camera::StereoCamera,
};

#[derive(Component, Clone, Copy, Debug)]
pub struct Position {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct Door;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Floor;

#[derive(Component)]
pub struct Renderable {
    mesh: Handle,
}

#[derive(Component)]
struct PhysicsBody {
    body: rapier3d::dynamics::RigidBodyHandle,
}

pub struct GameWorld {
    world: World,
    schedule: Schedule,

    handle_store: HashMap<BlockType, Handle>,
}

#[derive(Resource)]
struct Input {
    player_movement: Option<cgmath::Vector3<f32>>, // if consumed  set to None
}

fn move_player_system(
    // keyboard_input: Res<Input<bevy::input::keyboard::KeyCode>>,
    mut physics_system: ResMut<PhysicsSystem>,
    mut input: ResMut<Input>,
    mut camera: ResMut<StereoCamera>,
    mut query: Query<(&mut Position, &PhysicsBody), With<Player>>,
) {
    if let Some(requested_movement) = input.player_movement.take() {
        let camera_look_direction = camera.get_camera_view_direction_projected_to_ground();

        // Get a matrix that rotates the world y axis to the camera look direction
        // We need this to transform the requested movement vector so that the player moves in the direction the camera is looking
        let camera_look_direction_rotation_matrix = cgmath::Matrix3::from_cols(
            camera_look_direction.cross(cgmath::Vector3::unit_z()).normalize(),
            camera_look_direction,
            cgmath::Vector3::unit_z(),
        );

        for (mut position, physics_body) in &mut query {
            let mut direction = requested_movement;
            if direction.magnitude() > 0.0 {
                direction = direction.normalize();
            }
            let player_max_speed = 0.1;
            physics_system.move_body(physics_body.body, camera_look_direction_rotation_matrix * direction * player_max_speed);

            let pos = physics_system.get_position(physics_body.body);
            position.position = pos.position;
            position.rotation = pos.rotation;
        }
    }
}

// Move the camera to always look at the player
fn move_camera_system(
    mut camera: ResMut<StereoCamera>,
    mut query: Query<(&Position, &PhysicsBody), With<Player>>,
) {
    for (position, _) in &mut query {

        let camera_target_goal = position.position;
        let camera_eye_goal = position.position + cgmath::Vector3::new(-15.0, -15.0, 15.0);

        camera.smooth_set_target(Point3::from_vec(camera_target_goal), 0.02);
        camera.smooth_set_eye(Point3::from_vec(camera_eye_goal), 0.02);
    }
}

fn physics_system(
    mut physics_system: ResMut<PhysicsSystem>,
    mut query: Query<(&mut Position, &PhysicsBody)>,
) {
    physics_system.step();
    for (mut position, physics_body) in &mut query {
        let pos = physics_system.get_position(physics_body.body);
        position.position = pos.position;
        position.rotation = pos.rotation;
    }
}

impl GameWorld {
    pub fn new(handle_store: HashMap<BlockType, Handle>) -> Self {
        let mut game_world = Self {
            world: World::default(),
            schedule: Schedule::default(),
            handle_store,
        };
        game_world.init();
        game_world
    }

    fn init(&mut self) {
        self.world.insert_resource(PhysicsSystem::new());
        self.world.insert_resource(Input {
            player_movement: None,
        });
        self.world.insert_resource(StereoCamera::new(
            (9.0, 0.0, 8.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_z(),
            1.0,
            20.0,
            0.1,
            50.0,
            -3.0, // view cross-eyed
        ));
        self.schedule.add_systems(physics_system);
        self.schedule.add_systems(move_player_system);
        self.schedule.add_systems(move_camera_system);
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }

    pub fn clear(&mut self) {
        self.world.clear_all();
        self.init();
    }

    pub fn add_cell(&mut self, x: i32, y: i32, cell: &Cell) {
        let mut z = 0;
        for (block_type, _) in cell.block_stack_iter() {
            if block_type != &BlockType::Empty {
                let position = Position {
                    position: cgmath::Vector3::new(x as f32 + 0.5, -y as f32 - 0.5, z as f32),
                    rotation: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                };

                let body_handle = self.world.resource_mut::<PhysicsSystem>().add_object(
                    x as f32 + 0.5,
                    -y as f32 - 0.5,
                    z as f32 + block_type.block_height() as f32 / 2.0,
                    0.5,
                    0.5,
                    block_type.block_height() as f32 / 2.0,
                    block_type.get_physics_type(),
                );

                let mut entity = self
                    .world
                    .spawn((position, PhysicsBody { body: body_handle }));

                match block_type {
                    BlockType::Player => {
                        entity.insert(Player);
                    }
                    BlockType::Goal => {
                        entity.insert(Goal);
                    }
                    BlockType::Door => {
                        entity.insert(Door);
                    }
                    BlockType::Wall => {
                        entity.insert(Wall);
                    }
                    BlockType::FloorNormal => {
                        entity.insert(Floor);
                    }
                    BlockType::Empty => {}
                }

                match self.handle_store.get(block_type) {
                    Some(handle) => {
                        entity.insert(Renderable { mesh: *handle });
                    }
                    None => {
                        log::warn!("No mesh for block type {:?}", block_type);
                    }
                }
            }

            z += block_type.block_height();
        }
    }

    pub fn move_player(&mut self, direction: cgmath::Vector3<f32>) {
        self.world
            .get_resource_mut::<Input>()
            .unwrap()
            .player_movement = Some(direction);
    }

    pub(crate) fn iter_instances(&mut self, mesh_handle: Handle) -> Vec<&Position> {
        let mut query = self.world.query::<(&Position, &Renderable)>();
        query
            .iter(&self.world)
            .filter(move |(_, renderable)| renderable.mesh == mesh_handle)
            .map(|(position, _)| position)
            .collect()
    }

    pub fn set_camera_aspect(&mut self, aspect: f32) {
        self.world
            .get_resource_mut::<StereoCamera>()
            .unwrap()
            .set_aspect(aspect);
    }

    pub fn set_eye_distance(&mut self, eye_distance: f32) {
        self.world
            .get_resource_mut::<StereoCamera>()
            .unwrap()
            .set_eye_distance(eye_distance);
    }

    pub fn get_camera(&self) -> &StereoCamera {
        self.world.resource::<StereoCamera>()
    }
}
