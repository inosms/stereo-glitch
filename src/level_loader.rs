use std::collections::HashSet;
use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::{newline, space1};

use nom::combinator::{opt, value};
use nom::multi::{separated_list0, separated_list1};

use nom::IResult;

use serde::{Deserialize, Serialize};

use crate::object_types::{Block, BoxType, Id, LinearEnemyDirection};

#[derive(Debug, PartialEq)]
pub struct Cell {
    is_glitch_area: bool,
    block_stack: Vec<(Block, Option<Id>)>,
}

impl Cell {
    pub fn block_stack_iter(&self) -> impl Iterator<Item = &(Block, Option<Id>)> {
        self.block_stack.iter()
    }

    pub fn block_stack_iter_mut(&mut self) -> impl Iterator<Item = &mut (Block, Option<Id>)> {
        self.block_stack.iter_mut()
    }

    pub fn is_glitch_area(&self) -> bool {
        self.is_glitch_area
    }
}

#[derive(PartialEq)]
pub struct ParsedLevel {
    cells: Vec<Vec<Cell>>,
}

impl ParsedLevel {
    pub const MAX_LEVEL_WIDTH_AND_HEIGHT: usize = 256;

    pub fn from(cells: Vec<Vec<Cell>>) -> anyhow::Result<Self> {
        let level = Self { cells };

        // =================================
        // Validate the level
        // =================================

        let (width, height, _depth) = level.dimensions();
        if width > Self::MAX_LEVEL_WIDTH_AND_HEIGHT || height > Self::MAX_LEVEL_WIDTH_AND_HEIGHT {
            return anyhow::bail!("Level is too large");
        }

        let player_count = level
            .iter_cells()
            .filter(|(_pos, cell)| {
                cell.block_stack_iter()
                    .any(|(block_type, _id)| block_type == &Block::Player)
            })
            .count();
        if player_count != 1 {
            return anyhow::bail!("Level must have exactly one player, found {}", player_count);
        }

        // Check uniqueness of IDs
        let mut ids = HashSet::new();
        for (_pos, cell) in level.iter_cells() {
            for (_block_type, id) in cell.block_stack_iter() {
                if let Some(id) = id {
                    if ids.contains(id) {
                        return anyhow::bail!("Duplicate ID");
                    }
                    ids.insert(id);
                }
            }
        }

        // Every door must reference an existing trigger
        let trigger_ids = level
            .iter_cells()
            .flat_map(|(_pos, cell)| {
                cell.block_stack_iter()
                    .filter_map(|(block, id)| match block {
                        Block::Trigger => id.clone(),
                        _ => None,
                    })
            })
            .collect::<HashSet<_>>();
        for (_pos, cell) in level.iter_cells() {
            for (_block_type, _id) in cell.block_stack_iter() {
                if let Block::Door(id) = _block_type {
                    if !trigger_ids.contains(id) {
                        return anyhow::bail!("Door references non-existing trigger");
                    }
                }
            }
        }

        Ok(level)
    }

    /// Returns an iterator over all cells in the level.
    /// The iterator returns a tuple of the form ((x,y), &cell)
    pub fn iter_cells(&self) -> impl Iterator<Item = ((i32, i32), &Cell)> {
        self.cells.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, cell)| ((x as i32, y as i32), cell))
        })
    }

    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = ((i32, i32), &mut Cell)> {
        self.cells.iter_mut().enumerate().flat_map(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(x, cell)| ((x as i32, y as i32), cell))
        })
    }

    /// Returns the dimensions of the level in the form of (x, y, z)
    pub fn dimensions(&self) -> (usize, usize, usize) {
        let y = self.cells.len();
        let x = self.cells.iter().map(|row| row.len()).max().unwrap_or(0);
        let z = self
            .cells
            .iter()
            .flat_map(|row| row.iter().map(|cell| cell.block_stack.len()))
            .max()
            .unwrap_or(0);

        (x, y, z)
    }

    /// Converts a level into raw RGBA8 data
    pub fn to_glitch_raw_rgba8(&self) -> Vec<u8> {
        // Create a new raw texture with the same dimensions as the level
        let mut image_buffer = image::ImageBuffer::new(
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32,
        );

        image_buffer.fill(255);

        for ((x, y), cell) in self.iter_cells() {
            // Set the glitch area to black and the rest to white
            let color = if cell.is_glitch_area() { 0 } else { 255u8 };

            image_buffer.put_pixel(x as u32, y as u32, image::Rgba([color, color, color, 255]));
        }

        // 4x upscaling
        let upsized = image::DynamicImage::ImageRgba8(image_buffer).resize(
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as u32 * 4,
            image::imageops::FilterType::Nearest,
        );

        // blur
        let blurred = upsized.blur(3.0);

        blurred.to_rgba8().into_raw()
    }

    /// Converts the given block to a player block and the given player block to a Checkpoint block
    pub(crate) fn set_checkpoint(&mut self, id: Id) {
        for (_pos, cell) in self.iter_cells_mut() {
            for (block, block_id_opt) in cell.block_stack_iter_mut() {
                if let Some(block_id) = block_id_opt {
                    if *block_id == id {
                        *block = Block::Player;
                    } else if *block == Block::Player {
                        *block = Block::Checkpoint;
                    }
                }
            }
        }
    }
}

impl Debug for ParsedLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y, z) = self.dimensions();
        write!(f, "Dimensions: ({}, {}, {})\n", x, y, z)?;

        for ((x, y), cell) in self.iter_cells() {
            write!(f, "Cell at ({},{}): {:?}\n", x, y, cell)?;
        }
        Ok(())
    }
}

// Ids are always of the form of #<id> where <id> is a string of length 1-10
// and is alphanumeric.
fn parse_id(input: &str) -> IResult<&str, Id> {
    let (rest, _) = tag("#")(input)?;
    let (rest, id) = take_while_m_n(1, 10, |c: char| c.is_ascii_alphanumeric())(rest)?;
    Ok((rest, Id::try_from(id).unwrap()))
}

// A door is of the form D(<id>)
fn parse_door(input: &str) -> IResult<&str, Block> {
    let (rest, _) = tag("D(")(input)?;
    let (rest, id) = parse_id(rest)?;
    let (rest, _) = tag(")")(rest)?;
    Ok((rest, Block::Door(id)))
}

// upper letter (A-Z)*
fn parse_goal_text(input: &str) -> IResult<&str, String> {
    let (rest, text) = take_while_m_n(1, 64, |c: char| c.is_ascii_uppercase())(input)?;
    Ok((rest, text.to_string()))
}

fn parse_goal(input: &str) -> IResult<&str, Block> {
    let (rest, _) = tag("G(")(input)?;
    let (rest, text) = parse_goal_text(rest)?;
    let (rest, _) = tag(")")(rest)?;
    Ok((rest, Block::Goal(text)))
}

// a block can be multiplicated in the z direction by adding x<amount> to the end
// e.g. Nx2, Px3, etc.
fn parse_multiplicator(input: &str) -> IResult<&str, usize> {
    let (rest, _) = tag("x")(input)?;
    let (rest, multiplicator) = take_while_m_n(1, 2, |c: char| c.is_ascii_digit())(rest)?;
    Ok((rest, multiplicator.parse::<usize>().unwrap()))
}

// a block is always of the form of a single character and an optional ID
// e.g. N, P, D, X, G, W, N#abc123, etc.
fn parse_block(input: &str) -> IResult<&str, Vec<(Block, Option<Id>)>> {
    let (rest, block) = alt((
        value(Block::FloorNormal, tag("N")),
        value(Block::Player, tag("P")),
        parse_door,
        value(Block::Empty, tag("X")),
        parse_goal,
        value(Block::Wall, tag("W")),
        value(Block::Box(BoxType::Free), tag("BF")),
        value(Block::Box(BoxType::XAxis), tag("BX")),
        value(Block::Box(BoxType::YAxis), tag("BY")),
        value(Block::Box(BoxType::RotationFixed), tag("BRF")),
        value(Block::Trigger, tag("T")),
        value(Block::Charge, tag("C")),
        value(Block::StaticEnemy, tag("E1")),
        value(Block::LinearEnemy(LinearEnemyDirection::XAxis), tag("E2X")),
        value(Block::LinearEnemy(LinearEnemyDirection::YAxis), tag("E2Y")),
        value(Block::Checkpoint, tag("S")),
        value(Block::FireworkEmitter, tag("F")),
    ))(input)?;

    let (rest, multiplicator) = opt(parse_multiplicator)(rest)?;
    // can only specify id if multiplicator is 1
    let (rest, id) = if multiplicator.is_none() || multiplicator.unwrap() == 1 {
        opt(parse_id)(rest)?
    } else {
        (rest, None)
    };

    // duplicate the block if multiplicator is specified
    let mut block_stack = Vec::new();
    for _ in 0..multiplicator.unwrap_or(1) {
        block_stack.push((block.clone(), id.clone().or_else(|| Some(Id::random()))));
    }

    Ok((rest, block_stack))
}

// A cell is of the form [_](N|P|D|X|G|W|...)(#[a-zA-Z0-9]{1,10})?
fn parse_cell(input: &str) -> IResult<&str, Cell> {
    let (rest, glitch_area_tag) = opt(tag("_"))(input)?;
    let (rest, block_stack) = separated_list0(tag("+"), parse_block)(rest)?;

    let is_glitch_area = glitch_area_tag.is_some();

    Ok((
        rest,
        Cell {
            is_glitch_area,
            block_stack: block_stack.into_iter().flatten().collect(),
        },
    ))
}

fn parse_level_line(input: &str) -> IResult<&str, Vec<Cell>> {
    separated_list1(space1, parse_cell)(input.trim())
}

pub fn parse_level(input: &str) -> Result<ParsedLevel, LevelParseError> {
    let (rest, parsed) = separated_list0(newline, parse_level_line)(input)
        .map_err(|e| LevelParseError::ParseFailed {
            rest: input.to_string(),
        })?;

    if rest.len() > 0 {
        return Err(LevelParseError::ParseFailed {
            rest: rest.to_string(),
        });
    }

    match ParsedLevel::from(parsed) {
        Ok(level) => Ok(level),
        Err(e) => Err(LevelParseError::ValidationError {
            message: e.to_string(),
        }),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LevelParseError {
    ParseFailed { rest: String },
    ValidationError { message: String },
}

impl std::fmt::Display for LevelParseError {
    // displays as JSON
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level() {
        assert_eq!(
            parse_level("N+P").unwrap(),
            ParsedLevel {
                cells: vec![vec![Cell {
                    is_glitch_area: false,
                    block_stack: vec![(Block::FloorNormal, None), (Block::Player, None)]
                }]]
            }
        );
        assert_eq!(
            parse_level("  N+P").unwrap(),
            ParsedLevel {
                cells: vec![vec![Cell {
                    is_glitch_area: false,
                    block_stack: vec![(Block::FloorNormal, None), (Block::Player, None)]
                }]]
            }
        );
        assert_eq!(
            parse_level("  N+P  ").unwrap(),
            ParsedLevel {
                cells: vec![vec![Cell {
                    is_glitch_area: false,
                    block_stack: vec![(Block::FloorNormal, None), (Block::Player, None)]
                }]]
            }
        );
        assert_eq!(
            parse_level("N N N\nN N+P N").unwrap(),
            ParsedLevel {
                cells: vec![
                    vec![
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        }
                    ],
                    vec![
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None), (Block::Player, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        }
                    ]
                ]
            }
        );

        assert_eq!(
            parse_level("N N     N\nN    N+P        N   \n\n").unwrap(),
            ParsedLevel {
                cells: vec![
                    vec![
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        }
                    ],
                    vec![
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None), (Block::Player, None)]
                        },
                        Cell {
                            is_glitch_area: false,
                            block_stack: vec![(Block::FloorNormal, None)]
                        }
                    ]
                ]
            }
        );
    }
}
