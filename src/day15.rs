use std::collections::HashSet;

use crate::{common::Position, parsers::signed};
use failure::{err_msg, Error};
use nom::{
    bytes::complete::tag,
    character::complete::newline,
    combinator::{all_consuming, map},
    multi::many1,
    sequence::{preceded, separated_pair, terminated, tuple}, IResult,
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


pub struct Sensor {
    position: Position,
    beacon: Position,
}

impl Sensor {
    fn empty_spaces_on_row(&self, y: i64) -> impl Iterator<Item=Position> + '_ {
        let radius = self.position.manhattan_distance_to(&self.beacon) as i64;
        let dy = y - self.position.y;
        let min_x = self.position.x - radius + dy.abs();
        let max_x = self.position.x + radius - dy.abs();
        (min_x..=max_x).map(move |x| Position { x, y}).filter(|position| *position != self.beacon)
    }
}


fn count_empty_spaces_on_row(sensors: &[Sensor], y: i64) -> usize {
    sensors.iter().flat_map(|sensor| sensor.empty_spaces_on_row(y)).collect::<HashSet<_>>().len()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Sensor]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(sensors: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = count_empty_spaces_on_row(&sensors, 2_000_000).to_string();
        (Some(part_one), None)
    }
}
