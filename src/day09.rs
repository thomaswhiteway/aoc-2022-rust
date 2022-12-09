use std::{cmp::Ordering, collections::HashSet};

use failure::{err_msg, Error};
use itertools::repeat_n;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, newline},
    combinator::{all_consuming, map, map_res, value},
    multi::many1,
    sequence::{separated_pair, terminated},
    IResult,
};

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

fn expand(moves: &[Move]) -> impl Iterator<Item = Direction> + '_ {
    moves.iter().flat_map(|move_| move_.expand())
}

fn move_head(head_position: &mut Position, direction: Direction) {
    match direction {
        Direction::Up => head_position.y += 1,
        Direction::Down => head_position.y -= 1,
        Direction::Left => head_position.x -= 1,
        Direction::Right => head_position.x += 1,
    }
}

fn move_tail(tail_position: &mut Position, head_position: Position) {
    if (tail_position.x - head_position.x).abs() < 2
        && (tail_position.y - head_position.y).abs() < 2
    {
        return;
    }

    tail_position.x += match head_position.x.cmp(&tail_position.x) {
        Ordering::Greater => 1,
        Ordering::Equal => 0,
        Ordering::Less => -1,
    };

    tail_position.y += match head_position.y.cmp(&tail_position.y) {
        Ordering::Greater => 1,
        Ordering::Equal => 0,
        Ordering::Less => -1,
    };
}

fn all_tail_positions(moves: &[Move]) -> HashSet<Position> {
    let mut tail_positions = HashSet::new();
    let mut head_position = Position::default();
    let mut tail_position = Position::default();

    tail_positions.insert(tail_position);

    for direction in expand(moves) {
        move_head(&mut head_position, direction);
        move_tail(&mut tail_position, head_position);
        tail_positions.insert(tail_position);
    }

    tail_positions
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Move]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        all_consuming(moves)(&data)
            .map_err(|err| err_msg(format!("Failed to parse moves: {}", err)))
            .map(|(_, moves)| moves)
    }

    fn solve(moves: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = all_tail_positions(&moves).len().to_string();

        (Some(part_one), None)
    }
}
