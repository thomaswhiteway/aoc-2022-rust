use failure::{err_msg, Error};
use itertools::Either;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    character::complete::newline,
    combinator::{all_consuming, map, map_res, opt, recognize, value},
    multi::many1,
    sequence::{pair, preceded, terminated},
    IResult,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    Noop,
    Add(i64),
}

impl Command {
    fn apply(&self, x: &mut i64) {
        if let Command::Add(dx) = self {
            *x += dx;
        }
    }
}

fn number(input: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(tag("-")), digit1)), |val: &str| {
        val.parse()
    })(input)
}

fn noop_command(input: &str) -> IResult<&str, Command> {
    value(Command::Noop, tag("noop"))(input)
}

fn add_command(input: &str) -> IResult<&str, Command> {
    map(preceded(tag("addx "), number), Command::Add)(input)
}

fn command(input: &str) -> IResult<&str, Command> {
    alt((noop_command, add_command))(input)
}

fn commands(input: &str) -> IResult<&str, Box<[Command]>> {
    map(many1(terminated(command, newline)), Vec::into_boxed_slice)(input)
}

fn as_single_cycle(commands: &[Command]) -> impl Iterator<Item = Command> + '_ {
    commands.iter().flat_map(|command| match command {
        Command::Noop => Either::Left([Command::Noop].into_iter()),
        Command::Add(val) => Either::Right([Command::Noop, Command::Add(*val)].into_iter()),
    })
}

fn signal_strength(cycle: i64, x: i64) -> i64 {
    cycle * x
}

fn record_signal_strength(cycle: i64) -> bool {
    (cycle - 20) % 40 == 0
}

fn total_signal_strength(commands: &[Command]) -> i64 {
    (2..)
        .zip(as_single_cycle(commands).scan(1, |x, command| {
            command.apply(x);
            Some(*x)
        }))
        .filter_map(|(cycle, x)| {
            if record_signal_strength(cycle) {
                Some(signal_strength(cycle, x))
            } else {
                None
            }
        })
        .sum()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Command]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        all_consuming(commands)(&data)
            .map_err(|err| err_msg(format!("Failed to parse commands: {}", err)))
            .map(|(_, commands)| commands)
    }

    fn solve(commands: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = total_signal_strength(&commands).to_string();
        (Some(part_one), None)
    }
}
