use std::collections::HashSet;

use crate::{common::Vector, parsers::signed};
use failure::{err_msg, Error};
use nom::{
    bytes::complete::tag,
    character::complete::newline,
    combinator::{all_consuming, map},
    multi::many1,
    sequence::{terminated, tuple},
};

fn parse_input(data: &str) -> Result<Box<[Vector<i64, 3>]>, Error> {
    let vector = map(
        tuple((signed, tag(","), signed, tag(","), signed)),
        |(x, _, y, _, z)| [x, y, z].into(),
    );
    let vectors = map(many1(terminated(vector, newline)), Vec::into_boxed_slice);
    all_consuming(vectors)(data)
        .map(|(_, vs)| vs)
        .map_err(|err| err_msg(format!("Failed to parse vectors: {}", err)))
}

fn find_surface_area(positions: &[Vector<i64, 3>]) -> usize {
    let occupied = positions.iter().cloned().collect::<HashSet<_>>();

    positions
        .iter()
        .flat_map(|pos| pos.adjacent())
        .filter(|adj| !occupied.contains(adj))
        .count()
}
pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Vector<i64, 3>]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(positions: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_surface_area(&positions).to_string();
        (Some(part_one), None)
    }
}
