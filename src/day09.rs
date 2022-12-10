mod parse {
    use super::{Direction, Move};
    use failure::{err_msg, Error};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, newline},
        combinator::{all_consuming, map, map_res, value},
        multi::many1,
        sequence::{separated_pair, terminated},
        IResult,
    };

    fn direction(input: &str) -> IResult<&str, Direction> {
        alt((
            value(Direction::Up, tag("U")),
            value(Direction::Down, tag("D")),
            value(Direction::Left, tag("L")),
            value(Direction::Right, tag("R")),
        ))(input)
    }

    fn distance(input: &str) -> IResult<&str, usize> {
        map_res(digit1, |num: &str| num.parse())(input)
    }

    fn head_move(input: &str) -> IResult<&str, Move> {
        map(
            terminated(separated_pair(direction, tag(" "), distance), newline),
            |(direction, distance)| Move {
                direction,
                distance,
            },
        )(input)
    }

    fn moves(input: &str) -> IResult<&str, Box<[Move]>> {
        map(many1(head_move), Vec::into_boxed_slice)(input)
    }

    pub fn parse_input(input: &str) -> Result<Box<[Move]>, Error> {
        all_consuming(moves)(&input)
            .map_err(|err| err_msg(format!("Failed to parse moves: {}", err)))
            .map(|(_, moves)| moves)
    }
}

use std::{cmp::Ordering, collections::HashSet};

use failure::Error;
use itertools::{chain, repeat_n};
use parse::parse_input;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    direction: Direction,
    distance: usize,
}

impl Move {
    fn expand(&self) -> impl Iterator<Item = Direction> {
        repeat_n(self.direction, self.distance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Clone, Debug)]
struct Rope<const L: usize> {
    positions: [Position; L],
}

impl<const L: usize> Default for Rope<L> {
    fn default() -> Self {
        Rope {
            positions: [Position::default(); L],
        }
    }
}

impl<const L: usize> Rope<L> {
    fn move_head(&mut self, direction: Direction) {
        let head_position = &mut self.positions[0];
        match direction {
            Direction::Up => head_position.y += 1,
            Direction::Down => head_position.y -= 1,
            Direction::Left => head_position.x -= 1,
            Direction::Right => head_position.x += 1,
        }
    }

    fn move_tail(&mut self, index: usize) {
        let last_position = self.positions[index - 1];
        let tail_position = &mut self.positions[index];

        if (tail_position.x - last_position.x).abs() < 2
            && (tail_position.y - last_position.y).abs() < 2
        {
            return;
        }

        tail_position.x += match last_position.x.cmp(&tail_position.x) {
            Ordering::Greater => 1,
            Ordering::Equal => 0,
            Ordering::Less => -1,
        };

        tail_position.y += match last_position.y.cmp(&tail_position.y) {
            Ordering::Greater => 1,
            Ordering::Equal => 0,
            Ordering::Less => -1,
        };
    }

    fn move_rope(&mut self, direction: Direction) {
        self.move_head(direction);
        for index in 1..L {
            self.move_tail(index);
        }
    }

    fn tail_position(&self) -> Position {
        *self.positions.last().unwrap()
    }
}

fn expand(moves: &[Move]) -> impl Iterator<Item = Direction> + '_ {
    moves.iter().flat_map(|move_| move_.expand())
}

fn all_tail_positions<const L: usize>(moves: &[Move]) -> impl Iterator<Item = Position> + '_ {
    let rope = Rope::<L>::default();
    chain(
        [rope.tail_position()],
        expand(moves).scan(rope, |rope, direction| {
            rope.move_rope(direction);
            Some(rope.tail_position())
        }),
    )
}

fn num_tail_positions<const L: usize>(moves: &[Move]) -> usize {
    all_tail_positions::<L>(moves).collect::<HashSet<_>>().len()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Move]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(moves: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = num_tail_positions::<2>(&moves).to_string();
        let part_two = num_tail_positions::<10>(&moves).to_string();

        (Some(part_one), Some(part_two))
    }
}
