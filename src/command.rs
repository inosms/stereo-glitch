use std::{collections::VecDeque, sync::Mutex};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{level_compressor, level_loader};

#[derive(Debug)]
pub enum Command {
    LoadLevel(level_loader::ParsedLevel),
    SetEyeDistance(f32),
    // SetSize(width, height, pixel_ratio (dpi))
    SetSize(u32, u32, f32),
    JoystickInput(f32, f32), // input as a vector (x, y)
    ActionButtonPressed,
    ActionButtonReleased,
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
pub fn load_level(level: &str) -> Result<(), String> {
    match level_loader::parse_level(level) {
        Ok(parsed) => {
            COMMANDS.push(Command::LoadLevel(parsed));
            return Ok(());
        }
        Err(e) => {
            log::info!("Error: {:?}", e);
            return Err(e.to_string());
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn check_level(level: &str) -> String {
    match level_loader::parse_level(level) {
        Ok(_) => {
            return String::from("{ \"result\": \"ok\" }");
        }
        Err(e) => {
            return format!(
                "{{ \"result\": \"error\", \"contents\": {} }}",
                e.to_string()
            );
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_eye_distance(distance: f32) {
    COMMANDS.push(Command::SetEyeDistance(distance));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_size(width: u32, height: u32, pixel_ratio: f32) {
    COMMANDS.push(Command::SetSize(width, height, pixel_ratio));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn joystick_input(x: f32, y: f32) {
    COMMANDS.push(Command::JoystickInput(x, y));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn action_button_pressed() {
    COMMANDS.push(Command::ActionButtonPressed);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn action_button_released() {
    COMMANDS.push(Command::ActionButtonReleased);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compress_level_to_url(level: &str) -> String {
    level_compressor::compress_level(level)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn decompress_level_from_url(level: &str) -> Result<String, String> {
    level_compressor::decompress_level(level)
}
