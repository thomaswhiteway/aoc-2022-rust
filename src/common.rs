#![allow(unused)]

use std::array;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Div, Index, Mul, RangeInclusive, Sub};

pub struct Vector<T, const S: usize>([T; S]);

impl<T: Clone, const S: usize> Clone for Vector<T, S> {
    fn clone(&self) -> Self {
        Vector(self.0.clone())
    }
}

impl<T: Hash, const S: usize> Hash for Vector<T, S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: PartialEq, const S: usize> PartialEq for Vector<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: Eq, const S: usize> Eq for Vector<T, S> {}

impl<const S: usize> Vector<i64, S> {
    pub fn adjacent(&self) -> impl Iterator<Item = Vector<i64, S>> + '_ {
        (0..S).flat_map(move |axis| {
            [-1, 1].into_iter().map(move |offset| {
                let mut pos = self.0;
                pos[axis] += offset;
                Vector(pos)
            })
        })
    }

    pub fn length(&self) -> i64 {
        self.0.iter().map(|d| d.abs()).sum()
    }
}

impl<T, const S: usize> From<[T; S]> for Vector<T, S> {
    fn from(pos: [T; S]) -> Self {
        Vector(pos)
    }
}

impl<T: Add + Copy, const S: usize> Add for Vector<T, S> {
    type Output = Vector<T::Output, S>;
    fn add(self, rhs: Self) -> Self::Output {
        Vector(array::from_fn(move |i| self.0[i] + rhs.0[i]))
    }
}

impl<T, const S: usize> Index<usize> for Vector<T, S> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: PartialOrd, const S: usize> Vector<RangeInclusive<T>, S> {
    pub fn contains(&self, position: &Vector<T, S>) -> bool {
        self.0
            .iter()
            .zip(position.0.iter())
            .all(|(range, d)| range.contains(d))
    }
}

impl<T: Debug, const S: usize> Debug for Vector<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (index, element) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            element.fmt(f)?;
        }

        write!(f, "}}")
    }
}

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

pub fn div_ceil(lhs: u64, rhs: u64) -> u64 {
    (lhs / rhs) + if lhs % rhs == 0 { 0 } else { 1 }
}
