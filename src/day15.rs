use crate::{common::Position, parsers::signed};
use failure::{err_msg, Error};
use nom::{
    bytes::complete::tag,
    character::complete::newline,
    combinator::{all_consuming, map},
    multi::many1,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::{
    cmp::{max, min},
    collections::HashSet,
    ops::RangeInclusive,
};

fn parse_input(data: &str) -> Result<Box<[Sensor]>, Error> {
    fn position(input: &str) -> IResult<&str, Position> {
        map(
            separated_pair(
                preceded(tag("x="), signed),
                tag(", "),
                preceded(tag("y="), signed),
            ),
            |pos: (i64, i64)| pos.into(),
        )(input)
    }
    let sensor = map(
        tuple((
            preceded(tag("Sensor at "), position),
            preceded(tag(": closest beacon is at "), position),
        )),
        |(position, beacon)| Sensor { position, beacon },
    );
    let sensors = map(many1(terminated(sensor, newline)), Vec::into_boxed_slice);

    all_consuming(sensors)(data)
        .map(|(_, sensors)| sensors)
        .map_err(|err| err_msg(format!("Failed to parse sensors: {}", err)))
}

fn intersect(x: RangeInclusive<i64>, y: RangeInclusive<i64>) -> Option<RangeInclusive<i64>> {
    let start = max(x.start(), y.start());
    let end = min(x.end(), y.end());
    if *start <= *end {
        Some(*start..=*end)
    } else {
        None
    }
}

pub struct Sensor {
    position: Position,
    beacon: Position,
}

impl Sensor {
    fn empty_range_on_row(
        &self,
        y: i64,
        x_range: RangeInclusive<i64>,
    ) -> Option<RangeInclusive<i64>> {
        let radius = self.position.manhattan_distance_to(&self.beacon) as i64;
        let dy = y - self.position.y;
        let min_x = self.position.x - radius + dy.abs();
        let max_x = self.position.x + radius - dy.abs();
        if min_x <= max_x {
            intersect(min_x..=max_x, x_range)
        } else {
            None
        }
    }
}

fn count_beacons_on_row(sensors: &[Sensor], y: i64) -> usize {
    sensors
        .iter()
        .filter_map(|sensor| {
            if sensor.beacon.y == y {
                Some(sensor.beacon.x)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .len()
}

fn collapse_ranges(ranges: &mut Vec<RangeInclusive<i64>>) {
    ranges.sort_by_key(|range| (*range.start(), *range.end()));

    let mut index = 0;
    while index + 1 < ranges.len() {
        if ranges[index].contains(ranges[index + 1].start()) {
            ranges[index] =
                *ranges[index].start()..=max(*ranges[index + 1].end(), *ranges[index].end());
            ranges.remove(index + 1);
        } else {
            index += 1;
        }
    }
}

fn scanned_ranges_on_row(
    sensors: &[Sensor],
    y: i64,
    x_range: RangeInclusive<i64>,
) -> Vec<RangeInclusive<i64>> {
    let mut ranges = sensors
        .iter()
        .filter_map(|sensor| sensor.empty_range_on_row(y, x_range.clone()))
        .collect::<Vec<_>>();
    collapse_ranges(&mut ranges);
    ranges
}

fn count_empty_spaces_on_row(sensors: &[Sensor], y: i64) -> usize {
    let ranges = scanned_ranges_on_row(sensors, y, i64::MIN..=i64::MAX);
    let num_beacons = count_beacons_on_row(sensors, y);
    ranges
        .iter()
        .map(|range| range.end() - range.start() + 1)
        .sum::<i64>() as usize
        - num_beacons
}

fn find_beacon(
    sensors: &[Sensor],
    x_range: RangeInclusive<i64>,
    y_range: RangeInclusive<i64>,
) -> Option<Position> {
    for y in y_range {
        let ranges = scanned_ranges_on_row(sensors, y, x_range.clone());
        if ranges != vec![x_range.clone()] {
            for x in x_range.clone() {
                if ranges.iter().all(|range| !range.contains(&x)) {
                    return Some(Position { x, y });
                }
            }
        }
    }
    None
}

fn get_tuning_frequency(position: Position) -> i64 {
    position.x * 4000000 + position.y
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Sensor]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(sensors: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = count_empty_spaces_on_row(&sensors, 2_000_000).to_string();
        let part_two = get_tuning_frequency(
            find_beacon(&sensors, 0..=4000000, 0..=4000000).expect("Failed to solve part two"),
        )
        .to_string();
        (Some(part_one), Some(part_two))
    }
}
