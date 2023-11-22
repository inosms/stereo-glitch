use std::{sync::Mutex, collections::VecDeque};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum Command {
    LoadLevel(String),
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
    COMMANDS.push(Command::LoadLevel(level.to_string()));

    // TODO: add channel and send result back to JS
    Ok(()) 
}