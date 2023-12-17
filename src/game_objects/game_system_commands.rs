use bevy_ecs::system::Resource;

use crate::level_loader::ParsedLevel;

pub enum GameSystemCommand {
    LoadLevel(ParsedLevel)
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
}