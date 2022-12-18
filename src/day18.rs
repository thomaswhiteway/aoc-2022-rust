use std::{array, collections::HashSet, ops::RangeInclusive};

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

fn find_total_surface_area<'a, T: Iterator<Item = &'a Vector<i64, 3>> + Clone>(
    positions: T,
) -> usize {
    let occupied = positions.clone().cloned().collect::<HashSet<_>>();

    positions
        .flat_map(|pos| pos.adjacent())
        .filter(|adj| !occupied.contains(adj))
        .count()
}

fn find_dimensions(positions: &[Vector<i64, 3>]) -> Vector<RangeInclusive<i64>, 3> {
    array::from_fn(|axis| {
        let min = positions.iter().map(|pos| pos[axis]).min().unwrap();
        let max = positions.iter().map(|pos| pos[axis]).max().unwrap();
        min..=max
    })
    .into()
}

fn surface_area_of_box(ranges: Vector<RangeInclusive<i64>, 3>) -> usize {
    let dimensions: [usize; 3] =
        array::from_fn(|i| (ranges[i].end() - ranges[i].start() + 1) as usize);

    2 * (dimensions[0] * dimensions[1]
        + dimensions[0] * dimensions[2]
        + dimensions[1] * dimensions[2])
}

fn find_external_surface_area(positions: &[Vector<i64, 3>]) -> usize {
    let dimensions = find_dimensions(positions);
    let scan_ranges: Vector<_, 3> =
        array::from_fn(|axis| dimensions[axis].start() - 1..=dimensions[axis].end() + 1).into();

    let occupied = positions.iter().collect::<HashSet<_>>();

    let start: Vector<i64, 3> = array::from_fn(|axis| *scan_ranges[axis].start()).into();
    let mut to_check: Vec<Vector<i64, 3>> = vec![start.clone()];
    let mut found = HashSet::from([start]);

    while let Some(position) = to_check.pop() {
        for adjacent in position.adjacent() {
            if scan_ranges.contains(&adjacent)
                && !found.contains(&adjacent)
                && !occupied.contains(&adjacent)
            {
                found.insert(adjacent.clone());
                to_check.push(adjacent);
            }
        }
    }

    find_total_surface_area(found.iter()) - surface_area_of_box(scan_ranges)
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Vector<i64, 3>]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(positions: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_total_surface_area(positions.iter()).to_string();
        let part_two = find_external_surface_area(&positions).to_string();
        (Some(part_one), Some(part_two))
    }
}
