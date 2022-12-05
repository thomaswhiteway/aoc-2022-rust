use failure::Error;
pub struct Solver {}

use nom::{
    bytes::complete::{tag, take_while1},
    combinator::{map, map_res},
    sequence::tuple,
    IResult,
};

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn number(input: &str) -> IResult<&str, usize> {
    map_res(take_while1(is_digit), |val: &str| val.parse())(input)
}

fn crate_move(input: &str) -> IResult<&str, Move> {
    map(
        tuple((
            tag("move "),
            number,
            tag(" from "),
            number,
            tag(" to "),
            number,
        )),
        |(_, num_crates, _, from, _, to)| Move {
            num_crates,
            from,
            to,
        },
    )(input)
}

struct Move {
    num_crates: usize,
    from: usize,
    to: usize,
}

impl Move {
    fn apply(&self, stacks: &mut [Vec<char>]) {
        for _ in 0..self.num_crates {
            if let Some(crate_name) = stacks[self.from - 1].pop() {
                stacks[self.to - 1].push(crate_name)
            }
        }
    }
}

fn read_diagram<'a, T: Iterator<Item = &'a str>>(lines: T) -> Vec<Vec<char>> {
    let mut diagram_lines = vec![];

    for line in lines {
        if line.is_empty() {
            break;
        }
        diagram_lines.push(line.chars().collect::<Vec<_>>());
    }

    let num_stacks = (diagram_lines[0].len() + 1) / 4;
    let max_depth = diagram_lines.len() - 1;

    (0..num_stacks)
        .map(|index| 1 + index * 4)
        .map(|col| {
            (0..=max_depth)
                .rev()
                .map(|row| diagram_lines[row][col])
                .take_while(|c| *c != ' ')
                .collect()
        })
        .collect()
}

fn read_moves<'a, T: Iterator<Item = &'a str> + 'a>(lines: T) -> Vec<Move> {
    lines.map(|line| crate_move(line).unwrap().1).collect()
}

pub struct Problem {
    stacks: Vec<Vec<char>>,
    moves: Vec<Move>,
}

impl super::Solver for Solver {
    type Problem = Problem;

    fn parse_input(data: &str) -> Result<Self::Problem, Error> {
        let mut lines = data.lines();
        let stacks = read_diagram(&mut lines);
        let moves = read_moves(&mut lines);

        Ok(Problem { stacks, moves })
    }

    fn solve(problem: Self::Problem) -> (Option<String>, Option<String>) {
        let mut stacks = problem.stacks.clone();
        for crate_move in problem.moves {
            crate_move.apply(&mut stacks);
        }

        let part_one = stacks
            .iter()
            .map(|stack| stack.last().cloned().unwrap_or(' '))
            .collect();

        (Some(part_one), None)
    }
}
