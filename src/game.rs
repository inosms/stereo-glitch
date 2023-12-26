use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use cgmath::{EuclideanSpace, InnerSpace, One, Rotation3, Vector3};
use rand::seq::IteratorRandom;
use rapier3d::geometry::ColliderHandle;

use crate::{
    game_objects::{
        charge::{
            charge_recharge_system, move_charge_ghost_system, player_charge_depletion_system,
            ChargeGhost, ChargeSpawnArea,
        },
        checkpoint::{
            self, animate_checkpoint_particles_system, set_checkpoint_system,
            spawn_checkpoint_particle_system, Checkpoint,
        },
        constants::TICKS_PER_SECOND,
        dust::animate_dust_particle_system,
        game_system_commands::{GameSystemCommand, GameSystemCommands},
        glitch_area::GlitchAreaVisibility,
        goal::{check_goal_reached_system, Goal},
        input::Input,
        model_manager::ModelManager,
        movable::{animate_moving_objects_system, move_movable_object_with_player_system, Movable, spawn_dust_on_moving_objects_system},
        physics_body::PhysicsBody,
        player::{move_player_system, spawn_dust_on_move_player_system, Player},
        position::Position,
        renderable::Renderable,
        sensor::Sensor,
        time_keeper::TimeKeeper,
    },
    level_loader::{Cell, ParsedLevel},
    model::ModelHandle,
    object_types::{Block, BlockType, Id, LinearEnemyDirection},
    physics::PhysicsSystem,
    stereo_camera::StereoCamera,
};

#[derive(Component)]
struct Door {
    open: bool,
    trigger_id: Id,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Box;

#[derive(Component)]
struct DamageArea {
    damage: f32,
}

#[derive(Component)]
struct LinearEnemy {
    current_movement_direction: Vector3<f32>,
    stuck_counter: u32,
}

// This component is used to make an entity invisible
#[derive(Component)]
pub struct Invisible;

pub struct GameWorld {
    world: World,
    schedule: Schedule,

    model_manager: ModelManager,
    level: Option<ParsedLevel>,
    camera_aspect: f32,
}

// Move the camera to always look at the player
fn move_camera_system(
    mut camera: ResMut<StereoCamera>,
    mut query: Query<(&Position, &PhysicsBody), With<Player>>,
) {
    for (position, _) in &mut query {
        let camera_target_goal = position.position;
        let camera_eye_goal = position.position + cgmath::Vector3::new(-11.0, -11.0, 22.0);

        camera.smooth_set_target(cgmath::Point3::from_vec(camera_target_goal), 0.04, 3.0);
        camera.smooth_set_eye(cgmath::Point3::from_vec(camera_eye_goal), 0.04, 3.0);
    }
}

fn physics_system(
    mut physics_system: ResMut<PhysicsSystem>,
    time_keeper: Res<TimeKeeper>,
    mut query: Query<(&mut Position, &PhysicsBody)>,
    mut trigger_query: Query<&mut Sensor>,
) {
    // Only step physics if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    physics_system.step();
    for (mut position, physics_body) in &mut query {
        let pos = physics_system.get_position(physics_body.body);
        position.position = pos.position;
        position.rotation = pos.rotation;
    }

    // Update the state of triggers according to the collision events

    // This map is used to map from a collider handle to the trigger component (to avoid nested queries)
    let mut handle_to_trigger_map = trigger_query
        .iter_mut()
        .map(|trigger| (trigger.collider, trigger))
        .collect::<HashMap<_, _>>();
    while let Some(collision_event) = physics_system.poll_collision_events() {
        let triggered = collision_event.started();
        let colliders_involved = vec![
            (collision_event.collider1(), collision_event.collider2()),
            (collision_event.collider2(), collision_event.collider1()),
        ];
        for (this_collider, other_collider) in colliders_involved {
            if let Some(user_data) = physics_system.get_user_data(other_collider) {
                let entity = Entity::from_bits(user_data as u64);
                if let Some(trigger) = handle_to_trigger_map.get_mut(&this_collider) {
                    if triggered {
                        trigger.triggered_by.insert(entity);
                    } else {
                        trigger.triggered_by.remove(&entity);
                    }
                    trigger.triggered = !trigger.triggered_by.is_empty();
                }
            }
        }
    }
}

fn fixed_update_system(mut time_keeper: ResMut<TimeKeeper>) {
    time_keeper.tick();
}

fn check_player_dead_system(mut query: Query<(&Position, &mut Player), With<Player>>) {
    for (position, mut player) in &mut query {
        if position.position.z < -1.0 {
            player.dead = true;
        }
        if player.charge < 0.0 {
            player.dead = true;
        }
    }
}

fn door_system(
    mut commands: Commands,
    mut query: Query<(&mut Door, Entity, &PhysicsBody)>,
    trigger_query: Query<&Sensor>,
    mut physics_system: ResMut<PhysicsSystem>,
) {
    let triggered_trigger_ids = trigger_query
        .iter()
        .filter(|trigger| trigger.triggered)
        .filter_map(|trigger| trigger.id.clone())
        .collect::<HashSet<_>>();

    for (mut door, entity, body) in &mut query {
        let open = triggered_trigger_ids.contains(&door.trigger_id);
        if open != door.open {
            door.open = open;

            if door.open {
                commands.entity(entity).insert(Invisible);
                physics_system.set_rigid_body_state(body.body, false);
                log::info!("Open door {:?}", door.trigger_id);
            } else {
                commands.entity(entity).remove::<Invisible>();
                physics_system.set_rigid_body_state(body.body, true);
                log::info!("Close door {:?}", door.trigger_id);
            }
        }
    }
}

// when a player triggers a sensor that is also attached to a damage area the player takes damage
// and is pushed away from the damage area
fn damage_area_system(
    time_keeper: Res<TimeKeeper>,
    mut input: ResMut<Input>,
    mut query: Query<(&DamageArea, &Sensor, &Position)>,
    mut player_query: Query<(&mut Player, &Position, &PhysicsBody)>,
    mut physics_system: ResMut<PhysicsSystem>,
) {
    // Only deplete charge if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    for (damage_area, sensor, sensor_position) in &mut query {
        for &triggering_entity in &sensor.triggered_by {
            // Only damage the player
            if let Ok((mut player, player_position, player_physics_body)) =
                player_query.get_mut(triggering_entity)
            {
                player.charge -= damage_area.damage;

                // push the player away from the damage area
                let player_position = player_position.position;
                let damage_area_position = sensor_position.position;
                let direction = (player_position - damage_area_position).normalize() * 5.0;
                physics_system.move_body(player_physics_body.body, direction, false);
                // otherwise the player might get stuck in the damage area
                input.player_paralized_cooldown = 0.2;
            }
        }
    }
}

// Linear enemies move along their defined axis until they hit an obstacle which is when they turn around
// An obstacle is something that causes the enemy to get stuck (e.g. a wall or another enemy).
// This is measured by the linear velocity of the enemy. If the linear velocity is almost 0 the enemy is stuck.
fn move_linear_enemy_system(
    time_keeper: Res<TimeKeeper>,
    mut query: Query<(&mut LinearEnemy, &PhysicsBody)>,
    mut physics_system: ResMut<PhysicsSystem>,
) {
    // Only move enemies if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.is_in_fixed_tick() {
        return;
    }

    for (mut enemy, body) in &mut query {
        let velocity = physics_system.get_velocity_magnitude(body.body);
        if velocity < 0.1 {
            enemy.stuck_counter += 1;

            // Wait a bit before turning around
            let stuck_counter_max = 20;
            if enemy.stuck_counter > stuck_counter_max {
                enemy.current_movement_direction = -enemy.current_movement_direction;
                enemy.stuck_counter = 0;
            }
        } else {
            enemy.stuck_counter = 0;
        }

        let enemy_max_speed = 4.0;
        let direction = enemy.current_movement_direction * enemy_max_speed;
        physics_system.move_body(body.body, direction, true);
    }
}

impl GameWorld {
    pub fn new(handle_store: HashMap<BlockType, Vec<ModelHandle>>) -> Self {
        let mut game_world = Self {
            world: World::default(),
            schedule: Schedule::default(),
            model_manager: ModelManager::new(handle_store),
            level: None,
            camera_aspect: 1.0,
        };
        game_world.init();
        game_world
    }

    fn init(&mut self) {
        self.world.insert_resource(PhysicsSystem::new());
        self.world.insert_resource(Input {
            player_movement: None,
            player_paralized_cooldown: 0.0,
        });
        self.world.insert_resource(StereoCamera::new(
            (0.0, -10.0, 0.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_z(),
            self.camera_aspect,
            20.0,
            0.1,
            50.0,
            -3.0, // view cross-eyed
        ));
        self.world.insert_resource(self.model_manager.clone());
        self.world
            .insert_resource(TimeKeeper::new(TICKS_PER_SECOND));
        self.world.insert_resource(GlitchAreaVisibility {
            visibility: 0.0,
            glitch_cells: HashSet::new(),
        });
        self.world.insert_resource(GameSystemCommands::new());
        // The physics system needs to run after the player system so that the player can move
        self.schedule.add_systems(
            (
                fixed_update_system,
                move_player_system,
                move_movable_object_with_player_system,
                damage_area_system,
                physics_system,
                charge_recharge_system,
                player_charge_depletion_system,
                move_linear_enemy_system,
                move_charge_ghost_system,
                animate_moving_objects_system,
                spawn_checkpoint_particle_system,
                animate_checkpoint_particles_system,
                animate_dust_particle_system,
                spawn_dust_on_move_player_system,
                spawn_dust_on_moving_objects_system,
            )
                .chain(),
        );
        self.schedule.add_systems(move_camera_system);
        self.schedule.add_systems(check_player_dead_system);
        self.schedule.add_systems(door_system);
        self.schedule.add_systems(check_goal_reached_system);
        self.schedule.add_systems(set_checkpoint_system);
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);

        // if player is dead, reset the level
        let dead_player = self
            .world
            .query::<&Player>()
            .iter(&self.world)
            .filter(|player| player.dead)
            .count();

        if dead_player > 0 {
            self.reset_level();
        }

        while let Some(command) = self
            .world
            .get_resource_mut::<GameSystemCommands>()
            .unwrap()
            .commands
            .pop()
        {
            match command {
                GameSystemCommand::LoadLevel(level) => {
                    self.load_level(level);
                }
                GameSystemCommand::SetCheckpoint(id) => {
                    log::info!("Set checkpoint {:?}", id);
                    self.level.as_mut().map(|level| {
                        level.set_checkpoint(id);
                    });
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.world.clear_all();
        self.init();
    }

    pub fn reset_level(&mut self) {
        self.clear();
        if let Some(level) = self.level.take() {
            let mut glitch_area = HashSet::new();
            for ((x, y), cell) in level.iter_cells() {
                self.add_cell(x, y, cell);
                if cell.is_glitch_area() {
                    glitch_area.insert((x, y));
                }
            }
            self.world
                .get_resource_mut::<GlitchAreaVisibility>()
                .unwrap()
                .glitch_cells = glitch_area;
            self.level = Some(level);
        }
    }

    pub fn load_level(&mut self, level: ParsedLevel) {
        self.level = Some(level);
        self.reset_level();
    }

    fn add_cell(&mut self, x: i32, y: i32, cell: &Cell) {
        let mut z = 0.0;
        for (block, id) in cell.block_stack_iter() {
            if block != &Block::Empty {
                let position = Position {
                    position: cgmath::Vector3::new(
                        x as f32 + 0.5,
                        -y as f32 - 0.5,
                        z as f32 + block.block_height() / 2.0,
                    ),
                    rotation: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                    color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                    scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    grabbed_scale_factor: 1.0,
                    grabbed_rotation: cgmath::Quaternion::one(),
                };

                // to prevent the boxes from getting stuck in each other
                let offset = 0.94;
                let xy_size = match block.get_block_type() {
                    BlockType::Player => 0.5 * 0.99,
                    _ => 0.5,
                };
                let (body_handle, collider_handle) =
                    self.world.resource_mut::<PhysicsSystem>().add_object(
                        position.position.x,
                        position.position.y,
                        position.position.z,
                        offset * xy_size,
                        offset * xy_size,
                        offset * block.block_height() / 2.0,
                        &block,
                    );

                // Add sensor
                let sensor_trigger = match block.get_block_type() {
                    BlockType::Trigger => Some(
                        self.world
                            .resource_mut::<PhysicsSystem>()
                            .add_sensor_collider(body_handle, 0.5, 0.5, 0.2, 0.0, 0.0, 0.05),
                    ),
                    BlockType::Charge => Some(
                        self.world
                            .resource_mut::<PhysicsSystem>()
                            .add_sensor_collider(body_handle, 0.25, 0.25, 0.5, 0.0, 0.0, 0.0),
                    ),
                    BlockType::StaticEnemy
                    | BlockType::LinearEnemy
                    | BlockType::Goal
                    | BlockType::Checkpoint => Some(
                        self.world
                            .resource_mut::<PhysicsSystem>()
                            .add_sensor_collider(body_handle, 0.55, 0.55, 0.55, 0.0, 0.0, 0.0),
                    ),
                    _ => None,
                };

                let mut entity = self
                    .world
                    .spawn((position, PhysicsBody { body: body_handle }));

                match block {
                    Block::Player => {
                        entity.insert(Player {
                            dead: false,
                            pulled_objects: Vec::new(),
                            charge: 0.0,
                        });
                    }
                    Block::Goal(text) => {
                        entity.insert((
                            Goal {
                                goal_level_text: text.clone(),
                            },
                            Sensor {
                                collider: sensor_trigger.unwrap(),
                                triggered: false,
                                id: id.clone(),
                                triggered_by: HashSet::new(),
                            },
                        ));
                    }
                    Block::Door(trigger_id) => {
                        entity.insert(Door {
                            open: false,
                            trigger_id: trigger_id.clone(),
                        });
                    }
                    Block::Wall => {
                        entity.insert(Wall);
                    }
                    Block::FloorNormal => {
                        entity.insert(Floor);
                    }
                    Block::Box(_) => {
                        entity.insert((Box, Movable::default()));
                    }
                    Block::Trigger => {
                        entity.insert(Sensor {
                            collider: sensor_trigger.unwrap(),
                            triggered: false,
                            id: id.clone(),
                            triggered_by: HashSet::new(),
                        });
                    }
                    Block::Charge => {
                        entity.insert((
                            Sensor {
                                collider: sensor_trigger.unwrap(),
                                triggered: false,
                                id: None,
                                triggered_by: HashSet::new(),
                            },
                            ChargeSpawnArea::new(),
                            Invisible,
                        ));
                    }
                    Block::StaticEnemy => {
                        entity.insert((
                            Sensor {
                                collider: sensor_trigger.unwrap(),
                                triggered: false,
                                id: None,
                                triggered_by: HashSet::new(),
                            },
                            DamageArea { damage: 10.0 },
                        ));
                    }
                    Block::LinearEnemy(direction) => {
                        entity.insert((
                            Sensor {
                                collider: sensor_trigger.unwrap(),
                                triggered: false,
                                id: None,
                                triggered_by: HashSet::new(),
                            },
                            LinearEnemy {
                                current_movement_direction: match direction {
                                    LinearEnemyDirection::XAxis => Vector3::unit_x(),
                                    LinearEnemyDirection::YAxis => Vector3::unit_y(),
                                },
                                stuck_counter: 0,
                            },
                            DamageArea { damage: 10.0 },
                            Movable::default(),
                        ));
                    }
                    Block::Empty => {}
                    Block::Checkpoint => {
                        entity.insert((
                            Sensor {
                                collider: sensor_trigger.unwrap(),
                                triggered: false,
                                id: id.clone(),
                                triggered_by: HashSet::new(),
                            },
                            Checkpoint::new(id.clone().unwrap()),
                        ));
                    }
                }
                // we need that later
                let added_entity_id = entity.id();

                if block.get_block_type() != BlockType::Checkpoint {
                    // TODO: refactor
                    match self.model_manager.get_handle(&block.get_block_type()) {
                        Some(handle) => {
                            entity.insert(Renderable { mesh: handle });
                        }
                        None => {
                            log::warn!("No mesh for block type {:?}", block.get_block_type());
                        }
                    }
                }

                if let Some(collider_handle) = collider_handle {
                    let entity_id = entity.id().to_bits() as u128;
                    self.world
                        .resource_mut::<PhysicsSystem>()
                        .set_user_data(collider_handle, entity_id);
                    // Do not set user data to the sensor collider
                    // For our purposes if a collider is a sensor it is not a physical object
                    // By doing that we can distinguish between sensors and other colliders
                }

                if block.get_block_type() == BlockType::Player {
                    // spawn a ghost following the player
                    self.world.spawn((
                        Position {
                            position: Vector3::new(
                                position.position.x,
                                position.position.y,
                                position.position.z + 0.5,
                            ),
                            rotation: position.rotation,
                            scale: position.scale,
                            color: position.color,
                            grabbed_scale_factor: position.grabbed_scale_factor,
                            grabbed_rotation: cgmath::Quaternion::one(),
                        },
                        Renderable {
                            mesh: self.model_manager.get_handle(&BlockType::Ghost).unwrap(),
                        },
                        ChargeGhost::new_following(added_entity_id, 1.4, position.position),
                    ));
                }
            }

            z += block.block_height();
        }
    }

    pub fn move_player(&mut self, direction: cgmath::Vector3<f32>) {
        self.world
            .get_resource_mut::<Input>()
            .unwrap()
            .player_movement = Some(direction);
    }

    pub(crate) fn iter_instances(&mut self, model_handle: ModelHandle) -> Vec<&Position> {
        let mut query = self
            .world
            .query_filtered::<(&Position, &Renderable), Without<Invisible>>();
        query
            .iter(&self.world)
            .filter(move |(_, renderable)| renderable.mesh == model_handle)
            .map(|(position, _)| position)
            .collect()
    }

    pub fn set_camera_aspect(&mut self, aspect: f32) {
        self.camera_aspect = aspect;
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

    pub fn player_grab_action(&mut self) {
        let player_position = self
            .world
            .query_filtered::<&Position, With<Player>>()
            .iter(&self.world)
            .next()
            .unwrap()
            .position;

        let grab_area_extent = 3.0;
        // find all Entities that are within [-PULL_AREA_EXTENT, PULL_AREA_EXTENT] of the player in x, y and z
        let query = self
            .world
            .query_filtered::<(&Position, Entity, &PhysicsBody), With<Movable>>()
            .iter(&self.world)
            .filter(|(position, _, _)| {
                (position.position.x - player_position.x).abs() < grab_area_extent
                    && (position.position.y - player_position.y).abs() < grab_area_extent
                    && (position.position.z - player_position.z).abs() < grab_area_extent
            })
            .map(|(_, entity, _)| entity)
            .collect::<Vec<_>>();

        let mut player = self
            .world
            .query_filtered::<&mut Player, With<Player>>()
            .iter_mut(&mut self.world)
            .next()
            .unwrap();
        player.pulled_objects = query;
    }

    pub fn release_player_grab_action(&mut self) {
        self.world
            .query_filtered::<&mut Player, With<Player>>()
            .iter_mut(&mut self.world)
            .next()
            .unwrap()
            .pulled_objects
            .clear();
    }

    pub fn ref_glitch_area_visibility(&self) -> &GlitchAreaVisibility {
        self.world.resource::<GlitchAreaVisibility>()
    }
}
