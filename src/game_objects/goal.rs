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
    let mut output = String::new();
    for line in input.lines() {
        output.push_str(&line_to_level(line));
        output.push_str("\n");
    }
    // add the player "+P" at the first cell (that is before the first whitespace following a character)
    let mut output = output.trim().to_string();
    let first_whitespace_index = output.find(' ').unwrap();
    output.insert_str(first_whitespace_index, "+P");
    output
}

fn line_to_level(input: &str) -> String {
    let mut output_lines = vec![String::new(); 7];

    for block in input.chars() {
        let block = letter_to_level_block(block);
        let block_length = block.lines().next().unwrap().trim().split_ascii_whitespace().count() + 2;
        let padding = " N+N ".repeat(block_length);
        // add padding to the beginning of the line
        output_lines[0].push_str(&padding);

        let lines = block.lines();
        for (i, line) in lines.enumerate() {
            // remove newline character and add
            let trimmed_line = line.trim();
            // with a one in 10 chance add a firework emitter
            if rand::random::<f32>() < 0.1 {
                output_lines[i+1].push_str(" N+N+F ");
            } else {
                output_lines[i+1].push_str(" N+N ");
            }
            output_lines[i+1].push_str(trimmed_line);
            output_lines[i+1].push_str(" N+N ");

        }
        // add padding to the end of the line 
        output_lines[6].push_str(&padding);
    }

    // concatenate all lines to a single string
    output_lines.join("\n")
}


fn letter_to_level_block(c: char) -> String {
    match c {
        'A' => "N+N N+W N+W N+N\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W".to_string(),
        'B' => "N+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W".to_string(),
        'C' => "N+N N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+N N+N N+N\nN+W N+N N+N N+N\nN+N N+W N+W N+W".to_string(),
        'D' => "N+W N+W N+W N+N\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+N".to_string(),
        'E' => "N+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+W N+W N+W".to_string(),
        'F' => "N+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+N N+N N+N".to_string(),
        'G' => "N+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+N N+W N+W\nN+W N+N N+W N+W\nN+W N+W N+W N+W".to_string(),
        'H' => "N+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W".to_string(),
        'I' => "N+N N+W N+N N+N\nN+N N+W N+N N+N\nN+N N+W N+N N+N\nN+N N+W N+N N+N\nN+N N+W N+N N+N".to_string(),
        'J' => "N+N N+N N+N N+W\nN+N N+N N+N N+W\nN+N N+N N+N N+W\nN+W N+N N+N N+W\nN+N N+W N+W N+N".to_string(),
        'K' => "N+W N+N N+N N+W\nN+W N+N N+W N+N\nN+W N+W N+N N+N\nN+W N+N N+W N+N\nN+W N+N N+N N+W".to_string(),
        'L' => "N+W N+N N+N N+N\nN+W N+N N+N N+N\nN+W N+N N+N N+N\nN+W N+N N+N N+N\nN+W N+W N+W N+W".to_string(),
        'M' => "N+W N+N N+N N+N N+W\nN+W N+W N+N N+W N+W\nN+W N+N N+W N+N N+W\nN+W N+N N+N N+N N+W\nN+W N+N N+N N+N N+W".to_string(),
        'N' => "N+W N+N N+N N+W\nN+W N+W N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+W N+W\nN+W N+N N+N N+W".to_string(),
        'O' => "N+N N+W N+W N+N\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+N N+W N+W N+N".to_string(),
        'P' => "N+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+N N+N N+N".to_string(),
        'Q' => "N+N N+W N+W N+N\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+N N+W N+W N+W".to_string(),
        'R' => "N+W N+W N+W N+W\nN+W N+N N+N N+W\nN+W N+W N+W N+W\nN+W N+N N+W N+N\nN+W N+N N+N N+W".to_string(),
        'S' => "N+W N+W N+W N+W\nN+W N+N N+N N+N\nN+W N+W N+W N+W\nN+N N+N N+N N+W\nN+W N+W N+W N+W".to_string(),
        'T' => "N+W N+W N+W\nN+N N+W N+N\nN+N N+W N+N\nN+N N+W N+N\nN+N N+W N+N".to_string(),
        'U' => "N+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+N N+W N+W N+N".to_string(),
        'V' => "N+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+W N+N N+N N+W\nN+N N+W N+W N+N".to_string(),
        'W' => "N+W N+N N+N N+N N+W\nN+W N+N N+N N+N N+W\nN+W N+N N+N N+N N+W\nN+W N+N N+W N+N N+W\nN+N N+W N+N N+W N+N".to_string(),
        'X' => "N+W N+N N+N N+W\nN+N N+W N+W N+N\nN+N N+W N+W N+N\nN+N N+W N+W N+N\nN+W N+N N+N N+W".to_string(),
        'Y' => "N+W N+N N+W\nN+W N+N N+W\nN+W N+W N+W\nN+N N+W N+N\nN+N N+W N+N".to_string(),
        'Z' => "N+W N+W N+W N+W\nN+N N+N N+W N+N\nN+N N+W N+N N+N\nN+W N+N N+N N+N\nN+W N+W N+W N+W".to_string(),
        _ => "N+N N+N\nN+N N+N\nN+N N+N\nN+N N+N\nN+N N+N".to_string(),
    }
}