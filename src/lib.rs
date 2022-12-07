use aocf::Aoc;
use failure::Error;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day20;
mod day21;
mod day22;
mod day23;
mod day24;
mod day25;

#[derive(Debug, Eq, PartialEq)]
pub enum Part {
    One,
    Two,
}

impl FromStr for Part {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "one" => Ok(Part::One),
            "two" => Ok(Part::Two),
            _ => Err(format!("Unknown part {}", s)),
        }
    }
}

pub trait Solver {
    type Problem;

    fn parse_input(data: String) -> Result<Self::Problem, Error>;
    fn solve(problem: Self::Problem) -> (Option<String>, Option<String>);
}

fn read_from_server(aoc: &mut Aoc) -> Result<String, Error> {
    aoc.get_input(false)
}

pub fn read_input<P: AsRef<Path>>(path: Option<P>, aoc: &mut Aoc) -> Result<String, Error> {
    if let Some(path) = &path {
        Ok(read_to_string(path)?.to_string())
    } else {
        read_from_server(aoc)
    }
}

pub fn solve<S: Solver>(data: String, aoc: &mut Aoc, submit: Option<Part>) -> Result<(), Error> {
    let problem = S::parse_input(data)?;
    let (part_one, part_two) = S::solve(problem);

    if let Some(solution) = part_one {
        println!("Part 1: {}", solution);

        if submit == Some(Part::One) {
            let outcome = (*aoc).submit(&solution)?;
            println!("{}", outcome);
        }
    }

    if let Some(solution) = part_two {
        println!("Part 2: {}", solution);

        if submit == Some(Part::Two) {
            let outcome = aoc.submit(&solution)?;
            println!("{}", outcome);
        }
    }

    Ok(())
}

pub fn solve_day(day: u32, data: String, aoc: &mut Aoc, submit: Option<Part>) -> Result<(), Error> {
    match day {
        1 => solve::<day01::Solver>(data, aoc, submit),
        2 => solve::<day02::Solver>(data, aoc, submit),
        3 => solve::<day03::Solver>(data, aoc, submit),
        4 => solve::<day04::Solver>(data, aoc, submit),
        5 => solve::<day05::Solver>(data, aoc, submit),
        6 => solve::<day06::Solver>(data, aoc, submit),
        7 => solve::<day07::Solver>(data, aoc, submit),
        8 => solve::<day08::Solver>(data, aoc, submit),
        9 => solve::<day09::Solver>(data, aoc, submit),
        10 => solve::<day10::Solver>(data, aoc, submit),
        11 => solve::<day11::Solver>(data, aoc, submit),
        12 => solve::<day12::Solver>(data, aoc, submit),
        13 => solve::<day13::Solver>(data, aoc, submit),
        14 => solve::<day14::Solver>(data, aoc, submit),
        15 => solve::<day15::Solver>(data, aoc, submit),
        16 => solve::<day16::Solver>(data, aoc, submit),
        17 => solve::<day17::Solver>(data, aoc, submit),
        18 => solve::<day18::Solver>(data, aoc, submit),
        19 => solve::<day19::Solver>(data, aoc, submit),
        20 => solve::<day20::Solver>(data, aoc, submit),
        21 => solve::<day21::Solver>(data, aoc, submit),
        22 => solve::<day22::Solver>(data, aoc, submit),
        23 => solve::<day23::Solver>(data, aoc, submit),
        24 => solve::<day24::Solver>(data, aoc, submit),
        25 => solve::<day25::Solver>(data, aoc, submit),
        _ => Err(failure::err_msg(format!("Invalid day {}", day))),
    }
}
