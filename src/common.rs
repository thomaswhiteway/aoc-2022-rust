#![allow(unused)]

use std::ops::{Add, Div, Mul, Sub};

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

    pub fn length(&self) -> i64 {
        self.x.abs() + self.y.abs()
    }

    pub fn points_to(self, other: Position) -> impl Iterator<Item = Position> {
        let diff = other - self;
        assert!(diff.x == 0 || diff.y == 0);
        let distance = diff.length();
        let delta = diff / distance;
        (0..distance).map(move |index| self + delta * index)
    }
}

impl From<(i64, i64)> for Position {
    fn from((x, y): (i64, i64)) -> Self {
        Position { x, y }
    }
}

impl Add for Position {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<i64> for Position {
    type Output = Self;
    fn div(self, rhs: i64) -> Self::Output {
        Position {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Mul<i64> for Position {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
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
