use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use cgmath::{EuclideanSpace, InnerSpace, Rotation3};
use rapier3d::geometry::ColliderHandle;

use crate::{
    glitch_area::GlitchAreaVisibility,
    level_loader::{Cell, ParsedLevel},
    mesh::Handle,
    object_types::{Block, BlockType, Id},
    physics::PhysicsSystem,
    stereo_camera::StereoCamera,
    time_keeper::TimeKeeper,
};

const TICKS_PER_SECOND: u32 = 60;

#[derive(Component, Clone, Copy, Debug)]
pub struct Position {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Position {
    fn get_cell(&self) -> (i32, i32) {
        (
            self.position.x.floor() as i32,
            (-self.position.y).floor() as i32,
        )
    }
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
struct Player {
    dead: bool,

    // the objects the player is currently pulling
    pulled_objects: Vec<Entity>,

    charge: f32,
}

#[derive(Component)]
struct Carryable;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct Door {
    open: bool,
    trigger_id: Id,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Charge {
    cooldown_left: f32,
}

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Box;

#[derive(Component)]
struct Sensor {
    collider: ColliderHandle,
    triggered: bool,
    id: Option<Id>,
    triggered_by: HashSet<Entity>,
}

#[derive(Component)]
struct DamageArea {
    damage: f32,
}

#[derive(Component)]
pub struct Renderable {
    mesh: Handle,
}

// This component is used to make an entity invisible
#[derive(Component)]
pub struct Invisible;

#[derive(Component)]
struct PhysicsBody {
    body: rapier3d::dynamics::RigidBodyHandle,
}

pub struct GameWorld {
    world: World,
    schedule: Schedule,

    handle_store: HashMap<BlockType, Handle>,
    level: Option<ParsedLevel>,
}

#[derive(Resource)]
struct Input {
    player_movement: Option<cgmath::Vector3<f32>>, // if consumed  set to None
    player_paralized_cooldown: f32,
}

fn move_player_system(
    // keyboard_input: Res<Input<bevy::input::keyboard::KeyCode>>,
    mut physics_system: ResMut<PhysicsSystem>,
    mut input: ResMut<Input>,
    camera: Res<StereoCamera>,
    time_keeper: Res<TimeKeeper>,
    mut query: Query<&PhysicsBody, With<Player>>,
    physics_body_query: Query<&PhysicsBody>,
    player_query: Query<&Player>,
) {
    // Only move the player if we are in a physics tick
    // Otherwise the player will be frame rate dependent
    if !time_keeper.peek() {
        return;
    }

    if input.player_paralized_cooldown > 0.0 {
        input.player_paralized_cooldown -= 1.0 / TICKS_PER_SECOND as f32;
        return;
    }

    let requested_movement = input
        .player_movement
        .take()
        .unwrap_or(cgmath::Vector3::new(0.0, 0.0, 0.0));
    let camera_look_direction = camera.get_camera_view_direction_projected_to_ground();

    // Get a matrix that rotates the world y axis to the camera look direction
    // We need this to transform the requested movement vector so that the player moves in the direction the camera is looking
    let camera_look_direction_rotation_matrix = cgmath::Matrix3::from_cols(
        camera_look_direction
            .cross(cgmath::Vector3::unit_z())
            .normalize(),
        camera_look_direction,
        cgmath::Vector3::unit_z(),
    );

    let mut direction = requested_movement;
    if direction.magnitude() > 1.0 {
        direction = direction.normalize();
    }
    let player_max_speed = 16.0;

    let direction = camera_look_direction_rotation_matrix * direction * player_max_speed;

    for physics_body in &mut query {
        physics_system.move_body(physics_body.body, direction, true);
    }

    // get all physics bodies the player is pulling
    let mut pulled_bodies = Vec::new();
    for player in &player_query {
        pulled_bodies = player
            .pulled_objects
            .iter()
            .filter_map(|entity| physics_body_query.get(*entity).ok())
            .collect();
    }
    for physics_body in pulled_bodies {
        physics_system.move_body(physics_body.body, direction, false);
    }
}

// Move the camera to always look at the player
fn move_camera_system(
    mut camera: ResMut<StereoCamera>,
    mut query: Query<(&Position, &PhysicsBody), With<Player>>,
) {
    for (position, _) in &mut query {
        let camera_target_goal = position.position;
        let camera_eye_goal = position.position + cgmath::Vector3::new(-35.0, -35.0, 35.0);

        camera.smooth_set_target(cgmath::Point3::from_vec(camera_target_goal), 0.02);
        camera.smooth_set_eye(cgmath::Point3::from_vec(camera_eye_goal), 0.02);
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
    if !time_keeper.peek() {
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

fn charge_recharge_system(
    mut commands: Commands,
    mut time_keeper: ResMut<TimeKeeper>,
    mut query: Query<(&mut Charge, &Sensor, Entity)>,
    mut player_query: Query<&mut Player>,
    mut physics_system: ResMut<PhysicsSystem>,
) {
    // Only recharge charge if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.peek() {
        return;
    }

    for (mut charge, sensor, sensor_entity) in &mut query {
        let triggering_player_entity = sensor
            .triggered_by
            .iter()
            .filter(|&entity| player_query.get_mut(*entity).is_ok())
            .collect::<Vec<_>>();

        let triggered_by_player = !triggering_player_entity.is_empty();
        let can_recharge = charge.cooldown_left <= 0.0;
        if triggered_by_player && can_recharge {
            for player_entity in triggering_player_entity {
                if let Ok(mut player) = player_query.get_mut(*player_entity) {
                    player.charge = (player.charge.max(0.0) + 20.0).min(100.0);
                }
            }

            charge.cooldown_left = 15.0;
        } else {
            charge.cooldown_left -= 1.0 / TICKS_PER_SECOND as f32;
        }

        if charge.cooldown_left <= 0.0 {
            commands.entity(sensor_entity).remove::<Invisible>();
        } else {
            commands.entity(sensor_entity).insert(Invisible);
        }
    }
}

// When the player is in a glitch area deplete the charge over time
// If the charge reaches 0 the player dies
fn player_charge_depletion_system(
    mut time_keeper: ResMut<TimeKeeper>,
    mut player_query: Query<(&mut Player, &Position)>,
    mut glitch_area_visibility: ResMut<GlitchAreaVisibility>,
) {
    // Only deplete charge if we are in a physics tick
    // Otherwise the physics system will be frame rate dependent
    if !time_keeper.peek() {
        return;
    }

    let deplete_per_second = 1.0;
    let deplete_per_tick = deplete_per_second / TICKS_PER_SECOND as f32;

    for (mut player, pos) in &mut player_query {
        let player_x_y_cell = pos.get_cell();
        let is_in_glitch_area = glitch_area_visibility
            .glitch_cells
            .contains(&player_x_y_cell);

        if is_in_glitch_area {
            player.charge -= deplete_per_tick;
        }

        let player_charge = if player.charge > 10.0 {
            1.0
        } else if player.charge > 0.0 {
            0.2
        } else {
            0.0
        };
        // smooth interpolation between 0 and 1
        let alpha = 0.99;
        glitch_area_visibility.visibility =
            glitch_area_visibility.visibility * alpha + player_charge * (1.0 - alpha);
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
    if !time_keeper.peek() {
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
                let direction = (player_position - damage_area_position).normalize() * 20.0;
                physics_system.move_body(player_physics_body.body, direction, false);
                // otherwise the player might get stuck in the damage area
                input.player_paralized_cooldown = 0.2;
            }
        }
    }
}

impl GameWorld {
    pub fn new(handle_store: HashMap<BlockType, Handle>) -> Self {
        let mut game_world = Self {
            world: World::default(),
            schedule: Schedule::default(),
            handle_store,
            level: None,
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
            (0.0, -10.0, 00.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_z(),
            1.0,
            10.0,
            0.1,
            50.0,
            -5.0, // view cross-eyed
        ));
        self.world
            .insert_resource(TimeKeeper::new(TICKS_PER_SECOND));
        self.world.insert_resource(GlitchAreaVisibility {
            visibility: 0.0,
            glitch_cells: HashSet::new(),
        });
        // The physics system needs to run after the player system so that the player can move
        self.schedule.add_systems(
            (
                move_player_system,
                damage_area_system,
                physics_system,
                charge_recharge_system,
                player_charge_depletion_system,
                fixed_update_system,
            )
                .chain(),
        );
        self.schedule.add_systems(move_camera_system);
        self.schedule.add_systems(check_player_dead_system);
        self.schedule.add_systems(door_system);
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
                };

                let (body_handle, collider_handle) =
                    self.world.resource_mut::<PhysicsSystem>().add_object(
                        position.position.x,
                        position.position.y,
                        position.position.z,
                        0.5,
                        0.5,
                        block.block_height() / 2.0,
                        block.get_block_type(),
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
                    BlockType::StaticEnemy => Some(
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
                    Block::Goal => {
                        entity.insert(Goal);
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
                    Block::Box => {
                        entity.insert((Box, Carryable));
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
                            Charge { cooldown_left: 0.0 },
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
                    Block::Empty => {}
                }

                match self.handle_store.get(&block.get_block_type()) {
                    Some(handle) => {
                        entity.insert(Renderable { mesh: *handle });
                    }
                    None => {
                        log::warn!("No mesh for block type {:?}", block.get_block_type());
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

    pub(crate) fn iter_instances(&mut self, mesh_handle: Handle) -> Vec<&Position> {
        let mut query = self
            .world
            .query_filtered::<(&Position, &Renderable), Without<Invisible>>();
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

    pub fn player_pull_action(&mut self) {
        let player_position = self
            .world
            .query_filtered::<&Position, With<Player>>()
            .iter(&self.world)
            .next()
            .unwrap()
            .position;

        let pull_area_extent = 2.0;
        // find all Entities that are within [-PULL_AREA_EXTENT, PULL_AREA_EXTENT] of the player in x, y and z
        let query = self
            .world
            .query_filtered::<(&Position, Entity), With<PhysicsBody>>()
            .iter(&self.world)
            .filter(|(position, _)| {
                (position.position.x - player_position.x).abs() < pull_area_extent
                    && (position.position.y - player_position.y).abs() < pull_area_extent
                    && (position.position.z - player_position.z).abs() < pull_area_extent
            })
            .map(|(_, entity)| entity)
            .collect::<Vec<_>>();

        let mut player = self
            .world
            .query_filtered::<&mut Player, With<Player>>()
            .iter_mut(&mut self.world)
            .next()
            .unwrap();
        player.pulled_objects = query;
    }

    pub fn release_player_pull_action(&mut self) {
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
