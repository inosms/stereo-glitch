use std::{sync::Mutex, collections::VecDeque};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::level_loader;

#[derive(Debug)]
pub enum Command {
    LoadLevel(level_loader::ParsedLevel),
    SetEyeDistance(f32),
}

pub struct CommandQueue {
    commands: Mutex<VecDeque<Command>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, command: Command) {
        self.commands.lock().unwrap().push_back(command);
    }

    pub fn pop(&self) -> Option<Command> {
        self.commands.lock().unwrap().pop_front()
    }
}

lazy_static::lazy_static! {
    pub static ref COMMANDS: CommandQueue = CommandQueue::new();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn load_level(level: &str) -> Result<(), String>{
    match level_loader::parse_level(level) {
        Ok(parsed) => {
            COMMANDS.push(Command::LoadLevel(parsed));
            return Ok(());
        },
        Err(e) => {
            log::info!("Error: {:?}", e);
            return Err(e.to_string());
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_eye_distance(distance: f32) {
    COMMANDS.push(Command::SetEyeDistance(distance));
}