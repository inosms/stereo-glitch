use bevy_ecs::{component::Component, system::{ResMut, Query}, entity::Entity, query::With};

use crate::object_types::Id;

use super::{game_system_commands::GameSystemCommands, sensor::Sensor, player::Player};

#[derive(Component)]
pub struct Checkpoint {
    id: Id
}

impl Checkpoint {
    pub fn new(id: Id) -> Self {
        Self {
            id
        }
    }
}

pub fn set_checkpoint_system(
    mut game_system_commands: ResMut<GameSystemCommands>,
    checkpoint_sensor_query: Query<(&Checkpoint, &Sensor)>,
    player_query: Query<Entity, With<Player>>,
){

    for (checkpoint, sensor) in checkpoint_sensor_query.iter() {
        for triggered_by in &sensor.triggered_by {
            if player_query.get(*triggered_by).is_ok() {
                game_system_commands.set_checkpoint(checkpoint.id.clone());
            }
        }
    }
}