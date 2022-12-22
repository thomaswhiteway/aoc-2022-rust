use crate::{
    common::{Direction, Position, Rotation},
    parsers::signed,
};
use failure::{err_msg, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, value},
    multi::many1,
};

use std::{collections::HashMap, ops::RangeInclusive};

fn parse_directions(input: &str) -> Result<Box<[Movement]>, Error> {
    let rotation = alt((
        value(Rotation::Left, tag("L")),
        value(Rotation::Right, tag("R")),
    ));

    let movement = alt((map(signed, Movement::Move), map(rotation, Movement::Turn)));

    all_consuming(many1(movement))(input)
        .map(|(_, movements)| movements.into_boxed_slice())
        .map_err(|err| err_msg(format!("Failed to parse directions: {}", err)))
}

struct Location {
    position: Position,
    direction: Direction,
}

pub struct Map {
    occupied: HashMap<Position, bool>,
    row_extents: Box<[RangeInclusive<i64>]>,
    col_extents: Box<[RangeInclusive<i64>]>,
}

impl<'a, T: IntoIterator<Item = &'a str>> From<T> for Map {
    fn from(lines: T) -> Self {
        let occupied: HashMap<_, _> = (1..)
            .zip(lines.into_iter())
            .flat_map(|(y, line)| {
                (1..).zip(line.chars()).filter_map(move |(x, c)| {
                    match c {
                        '.' => Some(false),
                        '#' => Some(true),
                        _ => None,
                    }
                    .map(move |occ| (Position { x, y }, occ))
                })
            })
            .collect();

        let max_x = occupied.keys().map(|pos| pos.x).max().unwrap();
        let max_y = occupied.keys().map(|pos| pos.y).max().unwrap();

        let row_extents = (1..=max_y)
            .map(|y| {
                let min = (1..=max_x)
                    .find(|&x| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                let max = (1..=max_x)
                    .rev()
                    .find(|&x| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                min..=max
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let col_extents = (1..=max_x)
            .map(|x| {
                let min = (1..=max_y)
                    .find(|&y| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                let max = (1..=max_y)
                    .rev()
                    .find(|&y| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                min..=max
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Map {
            occupied,
            row_extents,
            col_extents,
        }
    }
}

impl Map {
    fn start_location(&self) -> Location {
        Location {
            position: Position {
                x: *self.row_extents[0].start(),
                y: 1,
            },
            direction: Direction::East,
        }
    }

    fn extent_for_row(&self, pos: Position) -> &RangeInclusive<i64> {
        &self.row_extents[(pos.y - 1) as usize]
    }

    fn extent_for_col(&self, pos: Position) -> &RangeInclusive<i64> {
        &self.col_extents[(pos.x - 1) as usize]
    }

    fn next_step(&self, position: Position, direction: Direction) -> Option<Position> {
        let mut new_position = position.step(direction);
        if !self.occupied.contains_key(&new_position) {
            match direction {
                Direction::North => new_position.y = *self.extent_for_col(new_position).end(),
                Direction::East => new_position.x = *self.extent_for_row(new_position).start(),
                Direction::South => new_position.y = *self.extent_for_col(new_position).start(),
                Direction::West => new_position.x = *self.extent_for_row(new_position).end(),
            };
        }

        if !self.occupied.get(&new_position).unwrap() {
            Some(new_position)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Turn(Rotation),
    Move(i64),
}

impl Movement {
    fn apply(self, map: &Map, location: &mut Location) {
        match self {
            Movement::Turn(rotation) => location.direction = location.direction.rotate(rotation),
            Movement::Move(distance) => {
                for _ in 0..distance {
                    if let Some(position) = map.next_step(location.position, location.direction) {
                        location.position = position;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn score(location: &Location) -> i64 {
    1000 * location.position.y
        + 4 * location.position.x
        + match location.direction {
            Direction::East => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::North => 3,
        }
}

fn find_end_location(map: &Map, directions: &[Movement], mut location: Location) -> Location {
    for movement in directions {
        movement.apply(map, &mut location)
    }

    location
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = (Map, Box<[Movement]>);

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        let mut lines = data.lines().collect::<Vec<_>>();
        let directions = lines.pop().unwrap();
        lines.pop();
        let map = lines.into();
        Ok((map, parse_directions(directions)?))
    }

    fn solve((map, directions): Self::Problem) -> (Option<String>, Option<String>) {
        let end_location = find_end_location(&map, &directions, map.start_location());
        let part_one = score(&end_location).to_string();
        (Some(part_one), None)
    }
}
