#![allow(unused)]

use std::array;
use std::cmp::{max, min, Ordering};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Div, Index, Mul, RangeInclusive, Sub};

use failure::{err_msg, Error};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds(Option<NonEmptyBounds>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonEmptyBounds {
    pub top_left: Position,
    pub bottom_right: Position,
}

impl Bounds {
    pub const EMPTY: Self = Bounds(None);

    pub fn width(&self) -> i64 {
        self.0.map(|bounds| bounds.width()).unwrap_or_default()
    }

    pub fn height(&self) -> i64 {
        self.0.map(|bounds| bounds.height()).unwrap_or_default()
    }

    fn extend(&self, other: Position) -> NonEmptyBounds {
        match self.0 {
            None => other.into(),
            Some(bounds) => bounds.extend(other),
        }
    }

    pub fn non_empty(&self) -> Option<&NonEmptyBounds> {
        self.0.as_ref()
    }
}

impl From<NonEmptyBounds> for Bounds {
    fn from(bounds: NonEmptyBounds) -> Self {
        Bounds(Some(bounds))
    }
}

impl<I: IntoIterator<Item = Position>> From<I> for Bounds {
    fn from(iter: I) -> Self {
        iter.into_iter().fold(Bounds::EMPTY, |bounds, position| {
            bounds.extend(position).into()
        })
    }
}

impl NonEmptyBounds {
    pub fn width(&self) -> i64 {
        1 + self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> i64 {
        1 + self.bottom_right.y - self.top_left.y
    }

    fn extend(&self, other: Position) -> Self {
        NonEmptyBounds {
            top_left: Position {
                x: min(self.top_left.x, other.x),
                y: min(self.top_left.y, other.y),
            },
            bottom_right: Position {
                x: max(self.bottom_right.x, other.x),
                y: max(self.bottom_right.y, other.y),
            },
        }
    }

    pub fn iter_x(&self) -> impl Iterator<Item = i64> {
        self.top_left.x..=self.bottom_right.x
    }

    pub fn iter_y(&self) -> impl Iterator<Item = i64> {
        self.top_left.y..=self.bottom_right.y
    }
}

impl From<Position> for NonEmptyBounds {
    fn from(position: Position) -> Self {
        NonEmptyBounds {
            top_left: position,
            bottom_right: position,
        }
    }
}

impl Position {
    pub const ORIGIN: Position = Position { x: 0, y: 0 };

    pub fn step(self, direction: Direction) -> Self {
        self + direction.delta()
    }

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

    pub fn surrounding(&self) -> impl Iterator<Item = Position> + '_ {
        (-1..=1).flat_map(move |dx| {
            (-1..=1).filter_map(move |dy| {
                let delta = Position { x: dx, y: dy };
                if delta != Self::ORIGIN {
                    Some(*self + delta)
                } else {
                    None
                }
            })
        })
    }

    pub fn is_in_direction(&self, other: Position, direction: Direction) -> bool {
        match direction {
            Direction::North => other.y < self.y,
            Direction::East => other.x > self.x,
            Direction::South => other.y > self.y,
            Direction::West => other.x < self.x,
        }
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

    pub fn bounds(self, other: Position) -> NonEmptyBounds {
        NonEmptyBounds {
            top_left: Position {
                x: min(self.x, other.x),
                y: min(self.y, other.y),
            },
            bottom_right: Position {
                x: max(self.x, other.x),
                y: max(self.y, other.y),
            },
        }
    }

    pub fn rotate(self, rotation: Rotation) -> Position {
        match (rotation.0 % 4) {
            0 => self,
            1 => Position {
                x: self.y,
                y: -self.x,
            },
            2 => Position {
                x: -self.x,
                y: -self.y,
            },
            3 => Position {
                x: -self.y,
                y: self.x,
            },
            _ => unreachable!(),
        }
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

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
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

impl TryFrom<u8> for Direction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Direction::North),
            1 => Ok(Direction::East),
            2 => Ok(Direction::South),
            3 => Ok(Direction::West),
            _ => Err(err_msg(format!("Invalid direction: {}", value))),
        }
    }
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
            South => 'v',
            West => '<',
        }
    }

    pub fn rotate(self, rot: Rotation) -> Self {
        Direction::try_from((self as u8 + rot.0) % 4).unwrap()
    }

    pub fn rotation_to(self, direction: Direction) -> Rotation {
        let d1 = self as u8;
        let d2 = direction as u8;
        Rotation(if d1 <= d2 { d2 - d1 } else { d2 + 4 - d1 })
    }

    pub fn opposite(self) -> Self {
        self.rotate(Rotation::HALF)
    }

    pub fn delta(self) -> Position {
        match self {
            Direction::North => (0, -1).into(),
            Direction::East => (1, 0).into(),
            Direction::South => (0, 1).into(),
            Direction::West => (-1, 0).into(),
        }
    }
}

impl TryFrom<usize> for Direction {
    type Error = Error;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Direction::North),
            1 => Ok(Direction::East),
            2 => Ok(Direction::South),
            3 => Ok(Direction::West),
            _ => Err(err_msg(format!("Invalid direction: {}", value))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rotation(pub u8);

impl Rotation {
    pub const NONE: Rotation = Rotation(0);
    pub const RIGHT: Rotation = Rotation(1);
    pub const HALF: Rotation = Rotation(2);
    pub const LEFT: Rotation = Rotation(3);

    pub fn inverse(self) -> Rotation {
        Rotation((4 - self.0) % 4)
    }
}

pub fn div_ceil(lhs: u64, rhs: u64) -> u64 {
    (lhs / rhs) + if lhs % rhs == 0 { 0 } else { 1 }
}

pub fn int_sqrt(val: u64) -> Option<u64> {
    for x in (0..) {
        match (x * x).cmp(&val) {
            Ordering::Equal => return Some(x),
            Ordering::Greater => break,
            _ => {}
        }
    }
    None
}
