use lazy_static::lazy_static;
use bracket_lib::prelude::*;

pub const HEIGHT: usize = 64;
pub const WIDTH: usize = 64;
pub const HALLWAY_CHANCE: f32 = 0.5;
pub const VERTICAL_CORRIDOR_MIN_LENGTH: i32 = 2;
pub const VERTICAL_CORRIDOR_MAX_LENGTH: i32 = 9;
pub const HORIZONTAL_CORRIDOR_MIN_LENGTH: i32 = 5;
pub const HORIZONTAL_CORRIDOR_MAX_LENGTH: i32 = 15;

pub enum Operator {
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
    EqualTo,
}

impl Operator {
    pub fn eval(&self, a: i32, b: i32) -> bool {
        match self {
            Operator::LessThan => a < b,
            Operator::GreaterThan => a > b,
            Operator::LessThanEqualTo => a <= b,
            Operator::GreaterThanEqualTo => a >= b,
            Operator::EqualTo => a == b,
        }
    }
    pub fn string(&self) -> &str {
        match self {
            Operator::LessThan => "<",
            Operator::GreaterThan => ">",
            Operator::LessThanEqualTo => "<=",
            Operator::GreaterThanEqualTo => ">=",
            Operator::EqualTo => "==",
        }
    }
}

pub struct CellRules {
    pub adjacent_type: i32,
    pub into: i32,
    pub operator: Operator,
    pub n: i32,
}

impl CellRules {
    const fn new(adjacent_type: i32, into: i32, operator: Operator, n: i32) -> CellRules {
        CellRules {
            adjacent_type,
            into,
            operator,
            n,
        }
    }
}

lazy_static! {
    pub static ref CA: Vec<Vec<CellRules>> = vec![
        vec![CellRules::new(1, 1, Operator::GreaterThanEqualTo, 4)],
        vec![
            CellRules::new(0, 0, Operator::GreaterThanEqualTo, 5),
            CellRules::new(1, 0, Operator::LessThan, 2)
        ]
    ];
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    NoDir = -1,
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    pub fn transform(&self) -> Point {
        match self {
            Direction::NoDir => unreachable!("Direction::NoDir should never be transformed"),
            Direction::North => Point::new(0, -1),
            Direction::East => Point::new(1, 0),
            Direction::South => Point::new(0, 1),
            Direction::West => Point::new(-1, 0),
        }
    }
    pub fn opposite_dir(&self) -> Direction {
        match self {
            Direction::NoDir => unreachable!("Direction::NoDir has no opposite."),
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

pub struct DirectionIterator {
    current: Direction,
}

impl DirectionIterator {
    pub fn new() -> DirectionIterator {
        DirectionIterator {
            current: Direction::North,
        }
    }
}

impl Iterator for DirectionIterator {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        use Direction::*;
        let next_direction = match self.current {
            North => East,
            East => South,
            South => West,
            West => {
                return None;
            }
            NoDir => unreachable!("Direction::NoDir should never be iterated over."),
        };
        self.current = next_direction;
        Some(next_direction)
    }
}
