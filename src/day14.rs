use crate::{common::Position, parsers::signed};
use failure::{err_msg, Error};
use itertools::{chain, Itertools};
use nom::{
    bytes::complete::tag,
    character::complete::newline,
    combinator::{all_consuming, map},
    multi::{many1, separated_list1},
    sequence::{separated_pair, terminated},
};
use std::collections::HashMap;

fn parse_input(input: &str) -> Result<Box<[Path]>, Error> {
    let point = map(separated_pair(signed, tag(","), signed), Position::from);

    let path = map(
        map(separated_list1(tag(" -> "), point), Vec::into_boxed_slice),
        |points| Path { points },
    );

    let paths = map(many1(terminated(path, newline)), Vec::into_boxed_slice);

    all_consuming(paths)(input)
        .map(|(_, paths)| paths)
        .map_err(|err| err_msg(format!("Failed to parse paths: {}", err)))
}

pub struct Path {
    points: Box<[Position]>,
}

impl Path {
    fn positions(&self) -> impl Iterator<Item = Position> + '_ {
        chain(
            self.points
                .iter()
                .cloned()
                .tuple_windows()
                .flat_map(|(p1, p2)| p1.points_to(p2)),
            [*self.points.last().unwrap()],
        )
    }
}

struct Contents {
    contents: HashMap<Position, Filler>,
    max_y: i64,
}

impl Contents {
    fn new(rocks: HashMap<Position, Filler>) -> Self {
        let max_y = rocks.keys().map(|p| p.y).max().unwrap();
        Contents {
            contents: rocks,
            max_y,
        }
    }

    fn add_grain(&mut self, position: Position) {
        self.contents.insert(position, Filler::Sand);
    }

    fn is_out_of_bounds(&self, position: Position) -> bool {
        position.y > self.max_y
    }

    fn is_occupied(&self, position: Position) -> bool {
        self.contents.contains_key(&position)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Filler {
    Rock,
    Sand,
}

fn draw_paths(paths: &[Path]) -> Contents {
    let rocks = paths
        .iter()
        .flat_map(|path| path.positions())
        .map(|point| (point, Filler::Rock))
        .collect();
    Contents::new(rocks)
}

fn next_step(contents: &Contents, position: Position) -> Option<Position> {
    [0, -1, 1]
        .into_iter()
        .map(|dx| position + (dx, 1).into())
        .find(|pos| !contents.is_occupied(*pos))
}

fn drop_grain(contents: &mut Contents) -> Option<Position> {
    let mut position = Position { x: 500, y: 0 };

    while let Some(next_position) = next_step(contents, position) {
        position = next_position;

        if contents.is_out_of_bounds(position) {
            return None;
        }
    }

    Some(position)
}

fn fill_sand(contents: &mut Contents) -> usize {
    for index in 0.. {
        if let Some(position) = drop_grain(contents) {
            contents.add_grain(position);
        } else {
            return index;
        }
    }
    0
}

fn num_grains_to_stick(paths: &[Path]) -> usize {
    let mut contents = draw_paths(paths);
    fill_sand(&mut contents)
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Path]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(paths: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = num_grains_to_stick(&paths).to_string();
        (Some(part_one), None)
    }
}
