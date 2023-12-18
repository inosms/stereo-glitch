use bevy_ecs::system::Resource;

use crate::{level_loader::ParsedLevel, object_types::Id};

pub enum GameSystemCommand {
    LoadLevel(ParsedLevel),
    SetCheckpoint(Id)
}

#[derive(Resource)]
pub struct GameSystemCommands {
    pub commands: Vec<GameSystemCommand>,
}

impl GameSystemCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn load_level(&mut self, level: ParsedLevel) {
        self.commands.push(GameSystemCommand::LoadLevel(level));
    }

    pub fn set_checkpoint(&mut self, id: Id) {
        self.commands.push(GameSystemCommand::SetCheckpoint(id));
    }
}