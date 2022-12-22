use crate::{
    common::{int_sqrt, Direction, Position, Rotation},
    parsers::signed,
};
use failure::{err_msg, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, value},
    multi::many1,
};

use std::{
    array,
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::{stdout, Write},
    ops::RangeInclusive,
};

fn parse_directions(input: &str) -> Result<Box<[Movement]>, Error> {
    let rotation = alt((
        value(Rotation::LEFT, tag("L")),
        value(Rotation::RIGHT, tag("R")),
    ));

    let movement = alt((map(signed, Movement::Move), map(rotation, Movement::Turn)));

    all_consuming(many1(movement))(input)
        .map(|(_, movements)| movements.into_boxed_slice())
        .map_err(|err| err_msg(format!("Failed to parse directions: {}", err)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

struct FlatLocation {
    position: Position,
    direction: Direction,
}

impl Location for FlatLocation {
    fn turn(&mut self, rotation: Rotation) {
        self.direction = self.direction.rotate(rotation)
    }
}

pub struct FlatMap {
    width: u64,
    height: u64,
    occupied: HashMap<Position, bool>,
    row_extents: Box<[RangeInclusive<i64>]>,
    col_extents: Box<[RangeInclusive<i64>]>,
}

impl<'a, T: IntoIterator<Item = &'a str>> From<T> for FlatMap {
    fn from(lines: T) -> Self {
        let occupied: HashMap<_, _> = lines
            .into_iter()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars().enumerate().filter_map(move |(x, c)| {
                    match c {
                        '.' => Some(false),
                        '#' => Some(true),
                        _ => None,
                    }
                    .map(move |occ| {
                        (
                            Position {
                                x: x as i64,
                                y: y as i64,
                            },
                            occ,
                        )
                    })
                })
            })
            .collect();

        let max_x = occupied.keys().map(|pos| pos.x).max().unwrap();
        let max_y = occupied.keys().map(|pos| pos.y).max().unwrap();

        let row_extents = (0..=max_y)
            .map(|y| {
                let min = (0..=max_x)
                    .find(|&x| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                let max = (0..=max_x)
                    .rev()
                    .find(|&x| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                min..=max
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let col_extents = (1..=max_x)
            .map(|x| {
                let min = (0..=max_y)
                    .find(|&y| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                let max = (0..=max_y)
                    .rev()
                    .find(|&y| occupied.contains_key(&Position { x, y }))
                    .unwrap();
                min..=max
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        FlatMap {
            occupied,
            width: max_x as u64 + 1,
            height: max_y as u64 + 1,
            row_extents,
            col_extents,
        }
    }
}

impl FlatMap {
    fn extent_for_row(&self, pos: Position) -> &RangeInclusive<i64> {
        &self.row_extents[pos.y as usize]
    }

    fn extent_for_col(&self, pos: Position) -> &RangeInclusive<i64> {
        &self.col_extents[pos.x as usize]
    }
}

impl Map for FlatMap {
    type Location = FlatLocation;

    fn start_location(&self) -> FlatLocation {
        FlatLocation {
            position: Position {
                x: *self.row_extents[0].start(),
                y: 0,
            },
            direction: Direction::East,
        }
    }

    fn flatten(&self, location: FlatLocation) -> FlatLocation {
        location
    }

    fn next_step(&self, loc: FlatLocation) -> FlatLocation {
        let mut position = loc.position.step(loc.direction);

        if !self.occupied.contains_key(&position) {
            match loc.direction {
                Direction::North => position.y = *self.extent_for_col(position).end(),
                Direction::East => position.x = *self.extent_for_row(position).start(),
                Direction::South => position.y = *self.extent_for_col(position).start(),
                Direction::West => position.x = *self.extent_for_row(position).end(),
            };
        }

        FlatLocation {
            position,
            direction: loc.direction,
        }
    }

    fn occupied(&self, loc: Self::Location) -> bool {
        *self.occupied.get(&loc.position).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Turn(Rotation),
    Move(i64),
}

impl Movement {
    fn apply<M: Map>(self, map: &M, location: &mut M::Location) {
        match self {
            Movement::Turn(rotation) => location.turn(rotation),
            Movement::Move(distance) => {
                for _ in 0..distance {
                    let new_location = map.next_step(*location);
                    if !map.occupied(new_location) {
                        *location = new_location
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn score(location: FlatLocation) -> i64 {
    1000 * (location.position.y + 1)
        + 4 * (location.position.x + 1)
        + match location.direction {
            Direction::East => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::North => 3,
        }
}

fn find_end_location<M: Map>(map: &M, directions: &[Movement]) -> FlatLocation {
    let mut location = map.start_location();

    for movement in directions {
        movement.apply(map, &mut location)
    }

    map.flatten(location)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CubeLocation {
    side: u8,
    location: FlatLocation,
}

impl Location for CubeLocation {
    fn turn(&mut self, rotation: Rotation) {
        self.location.turn(rotation)
    }
}

struct CubeMap {
    map: FlatMap,
    side_length: u64,

    // CubeLocations use the following canonical layout of sides.
    //  0
    // 415
    //  2
    //  3
    // In `sides` each entry is the offset of the top-left corner of that side
    // in the flat map, and the direction in the flat map that corresponds to
    // north in the canonical layout.
    sides: [(Position, Direction); 6],
}

impl CubeMap {
    fn rotate(
        &self,
        FlatLocation {
            position,
            direction,
        }: FlatLocation,
        rotation: Rotation,
    ) -> FlatLocation {
        let furthest = Position {
            x: self.side_length as i64 - 1,
            y: self.side_length as i64 - 1,
        }
        .rotate(rotation.inverse());
        let top_left = Position::ORIGIN.bounds(furthest).top_left;
        FlatLocation {
            position: position.rotate(rotation.inverse()) - top_left,
            direction: direction.rotate(rotation),
        }
    }

    fn adjacent_side(side: u8, direction: Direction) -> (u8, Rotation) {
        match side {
            0 | 1 | 2 | 3 => match direction {
                Direction::North => ((side + 3) % 4, Rotation::NONE),
                Direction::South => ((side + 1) % 4, Rotation::NONE),
                Direction::East => (5, Rotation((3 + side) % 4)),
                Direction::West => (4, Rotation((5 - side) % 4)),
            },
            4 => match direction {
                Direction::North => (0, Rotation::LEFT),
                Direction::East => (1, Rotation::NONE),
                Direction::South => (2, Rotation::RIGHT),
                Direction::West => (3, Rotation::HALF),
            },
            5 => match direction {
                Direction::North => (0, Rotation::RIGHT),
                Direction::East => (3, Rotation::HALF),
                Direction::South => (2, Rotation::LEFT),
                Direction::West => (1, Rotation::NONE),
            },
            _ => unreachable!(),
        }
    }

    fn find_sides(map: &FlatMap, side_length: u64) -> [(Position, Direction); 6] {
        let side_0_pos = Position {
            x: *map.extent_for_row(Position::ORIGIN).start(),
            y: 0,
        };

        let mut found_positions = HashMap::new();
        let mut added = HashSet::new();

        let mut stack = vec![];

        stack.push((0, side_0_pos, Direction::North));
        added.insert(side_0_pos);

        while let Some((side, position, up)) = stack.pop() {
            found_positions.insert(side, (position, up));
            added.insert(position);

            for direction in Direction::all() {
                let next_pos = position + direction.delta() * side_length as i64;
                if map.occupied.contains_key(&next_pos) && !added.contains(&next_pos) {
                    let rotation = up.rotation_to(Direction::North);
                    let (next_side, next_rotation) =
                        Self::adjacent_side(side, direction.rotate(rotation));
                    let next_up = up.rotate(next_rotation);

                    stack.push((next_side, next_pos, next_up));
                    added.insert(next_pos);
                }
            }
        }

        array::from_fn(|side| {
            *found_positions
                .get(&(side as u8))
                .unwrap_or_else(|| panic!("Failed to find side {}", side))
        })
    }
}

impl Map for CubeMap {
    type Location = CubeLocation;

    fn start_location(&self) -> Self::Location {
        CubeLocation {
            side: 0,
            location: FlatLocation {
                position: Position::ORIGIN,
                direction: Direction::East,
            },
        }
    }

    fn flatten(&self, location: Self::Location) -> FlatLocation {
        let (offset, direction) = self.sides[location.side as usize];
        let mut rotated = self.rotate(location.location, Direction::North.rotation_to(direction));
        rotated.position += offset;
        rotated
    }

    fn occupied(&self, loc: Self::Location) -> bool {
        self.map.occupied(self.flatten(loc))
    }

    fn next_step(&self, loc: Self::Location) -> Self::Location {
        let position = loc.location.position.step(loc.location.direction);

        let edge = if position.x < 0 {
            Some(Direction::West)
        } else if position.x >= self.side_length as i64 {
            Some(Direction::East)
        } else if position.y < 0 {
            Some(Direction::North)
        } else if position.y >= self.side_length as i64 {
            Some(Direction::South)
        } else {
            None
        };

        if let Some(edge) = edge {
            let (new_side, rotation) = Self::adjacent_side(loc.side, edge);
            let new_position = match edge {
                Direction::North => Position {
                    x: position.x,
                    y: self.side_length as i64 - 1,
                },
                Direction::East => Position {
                    x: 0,
                    y: position.y,
                },
                Direction::South => Position {
                    x: position.x,
                    y: 0,
                },
                Direction::West => Position {
                    x: self.side_length as i64 - 1,
                    y: position.y,
                },
            };

            let location = FlatLocation {
                position: new_position,
                direction: loc.location.direction,
            };

            CubeLocation {
                side: new_side,
                location: self.rotate(location, rotation.inverse()),
            }
        } else {
            CubeLocation {
                side: loc.side,
                location: FlatLocation {
                    position,
                    direction: loc.location.direction,
                },
            }
        }
    }

    fn draw<W: Write>(&self, mut writer: W, location: Option<Self::Location>) {
        let side_positions = self
            .sides
            .iter()
            .enumerate()
            .map(|(side, (position, direction))| (*position, (side as u8, *direction)))
            .collect::<HashMap<_, _>>();

        let grid = (0..self.map.height)
            .step_by(self.side_length as usize)
            .map(|y| {
                (0..self.map.width)
                    .step_by(self.side_length as usize)
                    .map(|x| {
                        side_positions.get(&Position {
                            x: x as i64,
                            y: y as i64,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut display = HashMap::new();
        let grid_width = self.map.width / self.side_length;
        let grid_height = self.map.height / self.side_length;

        // Draw rows
        for y in (0..grid_height * 6 + 1).step_by(6) {
            for x in 1..6 * grid_width {
                display.insert((x, y), '-');
            }
        }

        // Draw columns
        for x in (0..grid_width * 6 + 1).step_by(6) {
            for y in 1..6 * grid_height {
                display.insert((x, y), '|');
            }
        }

        // Draw corners
        for x in (0..grid_width * 6 + 1).step_by(6) {
            for y in (0..grid_height * 6 + 1).step_by(6) {
                display.insert((x, y), '+');
            }
        }

        for x in 0..grid_width {
            for y in 0..grid_height {
                if let Some((side, direction)) = grid[y as usize][x as usize] {
                    display.insert(
                        (x * 6 + 2, y * 6 + 3),
                        char::from_digit(*side as u32, 10).unwrap(),
                    );

                    display.insert((x * 6 + 4, y * 6 + 3), direction.as_char());
                }
            }
        }

        if let Some(loc) = location {
            let loc = self.flatten(loc);
            let mut x = loc.position.x / 10;
            x += x / 5 + 1;
            let mut y = loc.position.y / 10;
            y += y / 5 + 1;

            display.insert((x as u64, y as u64), loc.direction.as_char());
        }

        for y in 0..grid_height * 6 + 1 {
            for x in 0..grid_height * 6 + 1 {
                write!(writer, "{}", display.get(&(x, y)).unwrap_or(&' ')).unwrap();
            }
            writeln!(writer).unwrap();
        }
    }
}

impl From<FlatMap> for CubeMap {
    fn from(map: FlatMap) -> Self {
        let side_length = int_sqrt(map.occupied.len() as u64 / 6).expect("Not a cube");
        let sides = Self::find_sides(&map, side_length);

        CubeMap {
            map,
            side_length,
            sides,
        }
    }
}

trait Location: Clone + Copy {
    fn turn(&mut self, rotation: Rotation);
}

trait Map {
    type Location: Location + Debug;

    fn start_location(&self) -> Self::Location;
    fn next_step(&self, loc: Self::Location) -> Self::Location;
    fn occupied(&self, loc: Self::Location) -> bool;
    fn flatten(&self, location: Self::Location) -> FlatLocation;
    fn draw<W: Write>(&self, _: W, _: Option<Self::Location>) {}
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = (FlatMap, Box<[Movement]>);

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        let mut lines = data.lines().collect::<Vec<_>>();
        let directions = lines.pop().unwrap();
        lines.pop();
        let map = lines.into();
        Ok((map, parse_directions(directions)?))
    }

    fn solve((map, directions): Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = score(find_end_location(&map, &directions)).to_string();

        let cube_map = CubeMap::from(map);
        cube_map.draw(stdout(), None);

        let part_two = score(find_end_location(&cube_map, &directions)).to_string();
        (Some(part_one), Some(part_two))
    }
}
