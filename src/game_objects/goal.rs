use bevy_ecs::{component::Component, system::{Query, ResMut}, entity::Entity, query::With};

use crate::level_loader::{ParsedLevel, parse_level};

use super::{sensor::Sensor, player::Player, game_system_commands::GameSystemCommands};

#[derive(Component)]
pub struct Goal {
    pub goal_level_text: String,
}


pub fn check_goal_reached_system(
    mut game_system_commands: ResMut<GameSystemCommands>,
    goal_sensor_query: Query<(&Goal, &Sensor)>,
    player_query: Query<Entity, With<Player>>,
) {
    // If a player has triggered a goal sensor, the level is finished
    let mut level_finished = false;
    let mut goal_level = String::new();
    'outer: for (goal, sensor) in goal_sensor_query.iter() {
        for triggered_by in &sensor.triggered_by {
            if player_query.get(*triggered_by).is_ok() {
                level_finished = true;
                goal_level = goal.goal_level_text.clone();
                break 'outer;
            }
        }
    }

    if level_finished {
        game_system_commands.load_level(parse_level(&string_to_level(&goal_level)).expect("Failed to parse level"));
    }
}

fn string_to_level(input: &str) -> String {
    let mut output = player_start_pos();
    for line in input.lines() {
        output.push_str(&line_to_level(line));
        output.push_str("\n");
    }
    output
}

fn line_to_level(input: &str) -> String {
    let mut output_lines = vec![String::new(); 6];

    for block in input.chars() {
        let block = letter_to_level_block(block);
        let lines = block.lines();
        for (i, line) in lines.enumerate() {
            // remove newline character and add
            let trimmed_line = line.trim();
            output_lines[i].push_str(trimmed_line);
            output_lines[i].push_str(" N+W ");

        }
        // add padding to the end of the line 
        let block_length = block.lines().next().unwrap().trim().split_ascii_whitespace().count() + 1;
        let padding = " N+W ".repeat(block_length);
        output_lines[5].push_str(&padding);
    }

    // concatenate all lines to a single string
    output_lines.join("\n")
}

fn player_start_pos() -> String {
    "N+W N+W N+W\nN+W N+W N+W\nN+W N+W N+W+P\nN+W N+W N+W\nN+W N+W N+W\n".to_string()
}

fn letter_to_level_block(c: char) -> String {
    match c {
        'A' => "N+W N+BRF N+BRF N+W\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF".to_string(),
        'B' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        'C' => "N+W N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W\nN+W N+BRF N+BRF N+BRF".to_string(),
        'D' => "N+BRF N+BRF N+BRF N+W\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+W".to_string(),
        'E' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        'F' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W".to_string(),
        'G' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+W N+BRF N+BRF\nN+BRF N+W N+BRF N+BRF\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        'H' => "N+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF".to_string(),
        'I' => "N+W N+BRF N+W N+W\nN+W N+BRF N+W N+W\nN+W N+BRF N+W N+W\nN+W N+BRF N+W N+W\nN+W N+BRF N+W N+W".to_string(),
        'J' => "N+W N+W N+W N+BRF\nN+W N+W N+W N+BRF\nN+W N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+W N+BRF N+BRF N+W".to_string(),
        'K' => "N+BRF N+W N+W N+BRF\nN+BRF N+W N+BRF N+W\nN+BRF N+BRF N+W N+W\nN+BRF N+W N+BRF N+W\nN+BRF N+W N+W N+BRF".to_string(),
        'L' => "N+BRF N+W N+W N+W\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        'M' => "N+BRF N+W N+W N+W N+BRF\nN+BRF N+BRF N+W N+BRF N+BRF\nN+BRF N+W N+BRF N+W N+BRF\nN+BRF N+W N+W N+W N+BRF\nN+BRF N+W N+W N+W N+BRF".to_string(),
        'N' => "N+BRF N+W N+W N+BRF\nN+BRF N+BRF N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF".to_string(),
        'O' => "N+W N+BRF N+BRF N+W\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+W N+BRF N+BRF N+W".to_string(),
        'P' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+W N+W N+W".to_string(),
        'Q' => "N+W N+BRF N+BRF N+W\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+W N+BRF N+BRF N+BRF".to_string(),
        'R' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+BRF N+W\nN+BRF N+W N+W N+BRF".to_string(),
        'S' => "N+BRF N+BRF N+BRF N+BRF\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF\nN+W N+W N+W N+BRF\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        'T' => "N+BRF N+BRF N+BRF\nN+W N+BRF N+W\nN+W N+BRF N+W\nN+W N+BRF N+W\nN+W N+BRF N+W".to_string(),
        'U' => "N+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+W N+BRF N+BRF N+W".to_string(),
        'V' => "N+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+BRF N+W N+W N+BRF\nN+W N+BRF N+BRF N+W".to_string(),
        'W' => "N+BRF N+W N+W N+W N+BRF\nN+BRF N+W N+W N+W N+BRF\nN+BRF N+W N+W N+W N+BRF\nN+BRF N+W N+BRF N+W N+BRF\nN+W N+BRF N+W N+BRF N+W".to_string(),
        'X' => "N+BRF N+W N+W N+BRF\nN+W N+BRF N+BRF N+W\nN+W N+BRF N+BRF N+W\nN+W N+BRF N+BRF N+W\nN+BRF N+W N+W N+BRF".to_string(),
        'Y' => "N+BRF N+W N+BRF\nN+BRF N+W N+BRF\nN+BRF N+BRF N+BRF\nN+W N+BRF N+W\nN+W N+BRF N+W".to_string(),
        'Z' => "N+BRF N+BRF N+BRF N+BRF\nN+W N+W N+BRF N+W\nN+W N+BRF N+W N+W\nN+BRF N+W N+W N+W\nN+BRF N+BRF N+BRF N+BRF".to_string(),
        _ => "N+W N+W\nN+W N+W\nN+W N+W\nN+W N+W\nN+W N+W".to_string(),
    }
}