use std::collections::HashSet;
use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while1, take_while_m_n};
use nom::character::complete::{newline, space1};
use nom::character::is_space;
use nom::combinator::{opt, value};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Id {
    id: String,
}

impl TryFrom<&str> for Id {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Err("Empty string");
        }
        if value.len() > 10 {
            return Err("Too long");
        }
        if value.chars().any(|c| !c.is_ascii_alphanumeric()) {
            return Err("Not alphanumeric");
        }

        Ok(Id {
            id: value.to_string(),
        })
    }
}

pub enum BlockPhysicsType {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Block {
    FloorNormal,
    Player,
    /// A door can be opened by a trigger with the given ID
    Door(Id),
    Empty,
    Goal,
    Wall,
    Box,
    Trigger,
}


#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum BlockType {
    FloorNormal,
    Player,
    Door,
    Empty,
    Goal,
    Wall,
    Box,
    Trigger,
}

impl Block {
    pub fn get_block_type(&self) -> BlockType {
        match self {
            Block::FloorNormal => BlockType::FloorNormal,
            Block::Player => BlockType::Player,
            Block::Door(_) => BlockType::Door,
            Block::Empty => BlockType::Empty,
            Block::Goal => BlockType::Goal,
            Block::Wall => BlockType::Wall,
            Block::Box => BlockType::Box,
            Block::Trigger => BlockType::Trigger,
        }
    }

    pub fn block_height(&self) -> f32 {
        match self {
            Block::FloorNormal => 8.0,
            Block::Player => 1.0,
            Block::Door(_) => 1.0,
            Block::Empty => 1.0,
            Block::Goal => 1.0,
            Block::Wall => 1.0,
            Block::Box => 1.0,
            Block::Trigger => 0.02,
        }
    }

    pub fn get_physics_type(&self) -> BlockPhysicsType {
        match self {
            Block::FloorNormal => BlockPhysicsType::Static,
            Block::Player => BlockPhysicsType::Kinematic,
            Block::Door(_) => BlockPhysicsType::Static,
            Block::Empty => BlockPhysicsType::Static,
            Block::Goal => BlockPhysicsType::Static,
            Block::Wall => BlockPhysicsType::Static,
            Block::Box => BlockPhysicsType::Dynamic,
            Block::Trigger => BlockPhysicsType::Static,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    is_glitch_area: bool,
    block_stack: Vec<(Block, Option<Id>)>,
}

impl Cell {
    pub fn block_stack_iter(&self) -> impl Iterator<Item = &(Block, Option<Id>)> {
        self.block_stack.iter()
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
        let trigger_ids = level.iter_cells().flat_map(|(_pos, cell)| {
            cell.block_stack_iter()
                .filter_map(|(block, id)| match block {
                    Block::Trigger => id.clone(),
                    _ => None,
                })
        }).collect::<HashSet<_>>();
        for (_pos, cell) in level.iter_cells() {
            for (_block_type, id) in cell.block_stack_iter() {
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
        let mut raw_data = vec![
            255;
            ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT
                * ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT
                * 4
        ];

        for ((x, y), cell) in self.iter_cells() {
            let index = (x + y * ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT as i32) as usize * 4;

            // Set the glitch area to black and the rest to white
            let color = if cell.is_glitch_area() { 0 } else { 255 };

            raw_data[index] = color;
            raw_data[index + 1] = color;
            raw_data[index + 2] = color;
            raw_data[index + 3] = 255;
        }

        // debug print
        let (w, h, _d) = self.dimensions();
        let mut output = String::new();
        for y in 0..h {
            for x in 0..w {
                let index = (x + y * ParsedLevel::MAX_LEVEL_WIDTH_AND_HEIGHT) as usize * 4;
                let color = raw_data[index];
                if color == 0 {
                    output.push('X');
                } else {
                    output.push('.');
                }
            }
            output.push('\n');
        }
        log::info!("Glitch area:\n{}", output);

        raw_data
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

// a block is always of the form of a single character and an optional ID
// e.g. N, P, D, X, G, W, N#abc123, etc.
fn parse_block(input: &str) -> IResult<&str, (Block, Option<Id>)> {
    let (rest, block) = alt((
        value(Block::FloorNormal, tag("N")),
        value(Block::Player, tag("P")),
        parse_door,
        value(Block::Empty, tag("X")),
        value(Block::Goal, tag("G")),
        value(Block::Wall, tag("W")),
        value(Block::Box, tag("B")),
        value(Block::Trigger, tag("T")),
    ))(input)?;

    let (rest, id) = opt(parse_id)(rest)?;
    Ok((rest, (block, id)))
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
            block_stack,
        },
    ))
}

fn parse_level_line(input: &str) -> IResult<&str, Vec<Cell>> {
    separated_list1(space1, parse_cell)(input.trim())
}

pub fn parse_level(input: &str) -> anyhow::Result<ParsedLevel> {
    let (rest, parsed) =
        separated_list0(newline, parse_level_line)(input).map_err(|e| e.to_owned())?;

    if rest.len() > 0 {
        return Err(anyhow::anyhow!("Failed to parse level. Rest: {}", rest));
    }

    Ok(ParsedLevel::from(parsed)?)
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
                            block_stack: vec![
                                (Block::FloorNormal, None),
                                (Block::Player, None)
                            ]
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
                            block_stack: vec![
                                (Block::FloorNormal, None),
                                (Block::Player, None)
                            ]
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
