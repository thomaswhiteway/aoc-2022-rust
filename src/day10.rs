mod parse {
    use failure::{err_msg, Error};
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

    use super::Command;

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

    pub fn parse_input(input: &str) -> Result<Box<[Command]>, Error> {
        all_consuming(commands)(&input)
            .map_err(|err| err_msg(format!("Failed to parse commands: {}", err)))
            .map(|(_, commands)| commands)
    }

}


use failure::Error;
use itertools::{chain, Either, Itertools};

use self::parse::parse_input;

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

fn as_single_cycle(commands: &[Command]) -> impl Iterator<Item = Command> + '_ {
    commands.iter().flat_map(|command| match command {
        Command::Noop => Either::Left([Command::Noop].into_iter()),
        Command::Add(val) => Either::Right([Command::Noop, Command::Add(*val)].into_iter()),
    })
}

struct Screen<const W: usize, const H: usize> {
    pixels: [[char; W]; H],
}

impl<const W: usize, const H: usize> Default for Screen<W, H> {
    fn default() -> Self {
        Screen {
            pixels: [[' '; W]; H],
        }
    }
}

impl<const W: usize, const H: usize> Screen<W, H> {
    fn get_draw_position(&self, cycle: i64) -> (usize, usize) {
        let index = cycle as usize - 1;
        let x = index % W;
        let y = (index / W) % H;
        (x, y)
    }

    fn render(&self) -> String {
        self.pixels
            .iter()
            .map(|row| row.iter().collect::<String>())
            .join("\n")
    }

    fn set_pixel(&mut self, (x, y): (usize, usize), pixel: char) {
        self.pixels[y][x] = pixel;
    }

    fn draw(&mut self, commands: &[Command]) -> String {
        for (cycle, position) in positions(commands) {
            let draw_position = self.get_draw_position(cycle);
            if (draw_position.0 as i64).abs_diff(position) <= 1 {
                self.set_pixel(draw_position, '#');
            }
        }
        self.render()
    }
}

fn signal_strength(cycle: i64, x: i64) -> i64 {
    cycle * x
}

fn record_signal_strength(cycle: i64) -> bool {
    (cycle - 20) % 40 == 0
}

fn positions(commands: &[Command]) -> impl Iterator<Item = (i64, i64)> + '_ {
    chain(
        [(1, 1)],
        (2..).zip(as_single_cycle(commands).scan(1, |x, command| {
            command.apply(x);
            Some(*x)
        })),
    )
}

fn total_signal_strength(commands: &[Command]) -> i64 {
    positions(commands)
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
        parse_input(&data)
    }

    fn solve(commands: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = total_signal_strength(&commands).to_string();
        let part_two = Screen::<40, 6>::default().draw(&commands);
        (Some(part_one), Some(part_two))
    }
}
