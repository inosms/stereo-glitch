use rand::Rng;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Id {
    id: String,
}

impl Id {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let id: String = std::iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(10)
            .map(char::from)
            .collect();

        Self { id }
    }
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
    Goal(String),
    Wall,
    Box(BoxType),
    Trigger,
    Charge,
    StaticEnemy,
    LinearEnemy(LinearEnemyDirection),
    Checkpoint,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum LinearEnemyDirection{
    XAxis,
    YAxis,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum BoxType{
    XAxis,
    YAxis,
    RotationFixed,
    Free,
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
    StaticEnemy,
    LinearEnemy,
    Checkpoint,
}

impl Block {
    pub fn get_block_type(&self) -> BlockType {
        match self {
            Block::FloorNormal => BlockType::FloorNormal,
            Block::Player => BlockType::Player,
            Block::Door(_) => BlockType::Door,
            Block::Empty => BlockType::Empty,
            Block::Goal(_) => BlockType::Goal,
            Block::Wall => BlockType::Wall,
            Block::Box(_) => BlockType::Box,
            Block::Trigger => BlockType::Trigger,
            Block::Charge => BlockType::Charge,
            Block::StaticEnemy => BlockType::StaticEnemy,
            Block::LinearEnemy(_) => BlockType::LinearEnemy,
            Block::Checkpoint => BlockType::Checkpoint,
        }
    }

    pub fn block_height(&self) -> f32 {
        match self {
            Block::FloorNormal => 8.0,
            Block::Player => 1.0,
            Block::Door(_) => 1.0,
            Block::Empty => 1.0,
            Block::Goal(_) => 1.0,
            Block::Wall => 1.0,
            Block::Box(_) => 1.0,
            Block::Trigger => 0.0001,
            Block::Charge => 1.0,
            Block::StaticEnemy => 1.0,
            Block::LinearEnemy(_) => 1.0,
            Block::Checkpoint => 1.0,
        }
    }
}