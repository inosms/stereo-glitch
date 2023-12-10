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
    Charge,
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
    Charge,
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
            Block::Charge => BlockType::Charge,
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
            Block::Charge => 1.0,
        }
    }
}