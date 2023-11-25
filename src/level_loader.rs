use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while1, take_while_m_n};
use nom::character::complete::{newline, space1};
use nom::character::is_space;
use nom::combinator::{opt, value};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

#[derive(Debug, PartialEq)]
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
pub enum BlockType {
    FloorNormal,
    Player,
    Door,
    Empty,
    Goal,
    Wall,
}

impl BlockType {
    pub fn block_height(&self) -> i32 {
        match self {
            BlockType::FloorNormal => 1,
            BlockType::Player => 1,
            BlockType::Door => 1,
            BlockType::Empty => 1,
            BlockType::Goal => 1,
            BlockType::Wall => 2,
        }
    }

    pub fn get_physics_type(&self) -> BlockPhysicsType {
        match self {
            BlockType::FloorNormal => BlockPhysicsType::Static,
            BlockType::Player => BlockPhysicsType::Kinematic,
            BlockType::Door => BlockPhysicsType::Dynamic,
            BlockType::Empty => BlockPhysicsType::Static,
            BlockType::Goal => BlockPhysicsType::Static,
            BlockType::Wall => BlockPhysicsType::Static,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    is_glitch_area: bool,
    block_stack: Vec<(BlockType, Option<Id>)>,
}

impl Cell {
    pub fn block_stack_iter(&self) -> impl Iterator<Item = &(BlockType, Option<Id>)> {
        self.block_stack.iter()
    }
}

pub struct ParsedLevel {
    cells: Vec<Vec<Cell>>,
}

impl ParsedLevel {
    pub fn new(cells: Vec<Vec<Cell>>) -> Self {
        Self { cells }
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
}

impl Debug for ParsedLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ((x,y), cell) in self.iter_cells() {
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

// a block is always of the form of a single character and an optional ID
// e.g. N, P, D, X, G, W, N#abc123, etc.
fn parse_block(input: &str) -> IResult<&str, (BlockType, Option<Id>)> {
    let (rest, block) = alt((
        value(BlockType::FloorNormal, tag("N")),
        value(BlockType::Player, tag("P")),
        value(BlockType::Door, tag("D")),
        value(BlockType::Empty, tag("X")),
        value(BlockType::Goal, tag("G")),
        value(BlockType::Wall, tag("W")),
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
    separated_list1(space1, parse_cell)(input)
}

pub fn parse_level(input: &str) -> IResult<&str, ParsedLevel> {
    let (_rest, parsed) = separated_list0(newline, parse_level_line)(input)?;
    log::info!("Rest: {:?}", _rest);
    Ok((_rest, ParsedLevel::new(parsed)))
}
