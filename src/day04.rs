use std::ops::RangeInclusive;

use failure::{err_msg, Error};
use nom::{
    bytes::complete::{tag, take_while1},
    combinator::{map, map_res},
    multi::many1,
    sequence::separated_pair,
    sequence::terminated,
    IResult,
};

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn number(input: &str) -> IResult<&str, u64> {
    map_res(take_while1(is_digit), |val: &str| val.parse())(input)
}

fn range(input: &str) -> IResult<&str, RangeInclusive<u64>> {
    map(separated_pair(number, tag("-"), number), |(start, end)| {
        start..=end
    })(input)
}

fn assignment(input: &str) -> IResult<&str, Assignment> {
    map(separated_pair(range, tag(","), range), |(first, second)| {
        Assignment { first, second }
    })(input)
}

fn assignments(input: &str) -> IResult<&str, Box<[Assignment]>> {
    map(many1(terminated(assignment, tag("\n"))), |assignments| {
        assignments.into_boxed_slice()
    })(input)
}

pub struct Assignment {
    first: RangeInclusive<u64>,
    second: RangeInclusive<u64>,
}

impl Assignment {
    fn duplicate(&self) -> bool {
        subset(&self.first, &self.second) || subset(&self.second, &self.first)
    }

    fn overlaps(&self) -> bool {
        self.first.contains(self.second.start())
            || self.first.contains(self.second.end())
            || subset(&self.first, &self.second)
    }
}

fn subset(first: &RangeInclusive<u64>, second: &RangeInclusive<u64>) -> bool {
    (first.start() >= second.start()) && (first.end() <= second.end())
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Assignment]>;

    fn parse_input(data: &str) -> Result<Self::Problem, Error> {
        assignments(data)
            .map_err(|err| err_msg(format!("Failed to parse assignments: {}", err)))
            .map(|(_, a)| a)
    }

    fn solve(assignments: &Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = assignments
            .iter()
            .filter(|assignment| assignment.duplicate())
            .count()
            .to_string();

        let part_two = assignments
            .iter()
            .filter(|assignment| assignment.overlaps())
            .count()
            .to_string();

        (Some(part_one), Some(part_two))
    }
}
