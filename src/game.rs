use std::collections::HashMap;

use bevy_ecs::prelude::*;
use cgmath::Rotation3;

use crate::{
    level_loader::{BlockType, Cell},
    mesh::{Handle, Mesh},
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

pub struct GameWorld {
    world: World,
    schedule: Schedule,

    handle_store: HashMap<BlockType, Handle>,
}

// This system moves each entity with a Position and Velocity component
fn logger(query: Query<(&Position, &Player)>) {
    // for (position, _) in &query {
    //     log::info!("Player at {:?}", position.position);
    // }
}

impl GameWorld {
    pub fn new(handle_store: HashMap<BlockType, Handle>) -> Self {
        let world = World::default();
        let mut schedule = Schedule::default();
        schedule.add_systems(logger);

        Self {
            world,
            schedule,
            handle_store,
        }
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }

    pub fn clear(&mut self) {
        self.world.clear_all();
    }

    pub fn add_cell(&mut self, x: i32, y: i32, cell: &Cell) {
        let mut z = 0;
        for (block_type, _) in cell.block_stack_iter() {
            if block_type != &BlockType::Empty {
                let mut entity = self.world.spawn((
                    // position the object in the middle of the cell
                    Position {
                        position: cgmath::Vector3::new(
                            x as f32 + 0.5,
                            -y as f32 + 0.5,
                            z as f32,
                        ),
                        rotation: cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        ),
                    },
                ));

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

    pub(crate) fn iter_instances(&mut self, mesh_handle: Handle) -> Vec<&Position> {
        let mut query = self.world.query::<(&Position, &Renderable)>();
        query
            .iter(&self.world)
            .filter(move |(_, renderable)| renderable.mesh == mesh_handle)
            .map(|(position, _)| position)
            .collect()
    }
}
