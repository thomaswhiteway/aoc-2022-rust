#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn manhattan_distance_to(&self, other: &Self) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    pub fn adjacent(&self) -> impl Iterator<Item = Position> + '_ {
        [(1, 0), (0, 1), (-1, 0), (0, -1)]
            .into_iter()
            .map(|(dx, dy)| Position {
                x: self.x + dx,
                y: self.y + dy,
            })
    }

    pub fn direction_to(&self, other: &Self) -> Option<Direction> {
        match (other.x - self.x, other.y - self.y) {
            (0, -1) => Some(Direction::North),
            (1, 0) => Some(Direction::East),
            (0, 1) => Some(Direction::South),
            (-1, 0) => Some(Direction::West),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn all() -> impl Iterator<Item = Self> {
        use Direction::*;
        [North, East, South, West].into_iter()
    }

    pub fn as_char(&self) -> char {
        use Direction::*;
        match self {
            North => '^',
            East => '>',
            South => 'V',
            West => '<',
        }
    }
}
