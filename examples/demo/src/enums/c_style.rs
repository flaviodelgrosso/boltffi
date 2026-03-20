use boltffi::*;

/// Lifecycle status of an entity.
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Active,
    Inactive,
    Pending,
}

#[export]
pub fn echo_status(s: Status) -> Status {
    s
}

#[export]
pub fn status_to_string(s: Status) -> String {
    match s {
        Status::Active => "active".to_string(),
        Status::Inactive => "inactive".to_string(),
        Status::Pending => "pending".to_string(),
    }
}

#[export]
pub fn is_active(s: Status) -> bool {
    matches!(s, Status::Active)
}

#[export]
pub fn echo_vec_status(values: Vec<Status>) -> Vec<Status> {
    values
}

#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    North,
    South,
    East,
    West,
}

#[data(impl)]
impl Direction {
    pub fn new(raw: i32) -> Self {
        match raw {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            3 => Direction::West,
            _ => Direction::North,
        }
    }

    pub fn cardinal() -> Self {
        Direction::North
    }

    pub fn from_degrees(degrees: f64) -> Self {
        let normalized = ((degrees % 360.0) + 360.0) % 360.0;
        if normalized < 45.0 || normalized >= 315.0 {
            Direction::North
        } else if normalized < 135.0 {
            Direction::East
        } else if normalized < 225.0 {
            Direction::South
        } else {
            Direction::West
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::East | Direction::West)
    }

    pub fn label(&self) -> String {
        match self {
            Direction::North => "N".to_string(),
            Direction::South => "S".to_string(),
            Direction::East => "E".to_string(),
            Direction::West => "W".to_string(),
        }
    }

    pub fn count() -> u32 {
        4
    }
}

#[export]
pub fn echo_direction(d: Direction) -> Direction {
    d
}

#[export]
pub fn opposite_direction(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
    }
}
