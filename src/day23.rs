use failure::Error;
use std::collections::{HashMap, HashSet};

use crate::common::{Bounds, Direction, Position};

const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

fn find_next_position(elves: &HashSet<Position>, position: Position, round: usize) -> Position {
    let surrounding = position
        .surrounding()
        .filter(|pos| elves.contains(pos))
        .collect::<Vec<_>>();
    if surrounding.is_empty() {
        position
    } else {
        for dir_index in 0..DIRECTIONS.len() {
            let direction = DIRECTIONS[(dir_index + round) % DIRECTIONS.len()];
            if !surrounding
                .iter()
                .any(|&pos| position.is_in_direction(pos, direction))
            {
                return position.step(direction);
            }
        }
        position
    }
}

fn execute_round(elves: &mut HashSet<Position>, round: usize) {
    let moves = elves
        .iter()
        .map(|&pos| (pos, find_next_position(elves, pos, round)));

    let mut moving_to: HashMap<Position, Vec<Position>> = HashMap::new();
    for (current, next) in moves {
        moving_to.entry(next).or_default().push(current);
    }

    for (next_position, current_positions) in moving_to {
        if let &[position] = current_positions.as_slice() {
            elves.remove(&position);
            elves.insert(next_position);
        }
    }
}

fn execute_rounds(elves: &HashSet<Position>, num_rounds: usize) -> HashSet<Position> {
    let mut elves = elves.clone();

    for round in 0..num_rounds {
        execute_round(&mut elves, round);
    }

    elves
}

fn find_empty_space(elves: &HashSet<Position>) -> usize {
    let end_state = execute_rounds(elves, 10);
    let bounds: Bounds = end_state.iter().cloned().into();
    (bounds.width() * bounds.height()) as usize - elves.len()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = HashSet<Position>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        Ok(data
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars().enumerate().filter_map(move |(x, c)| {
                    if c == '#' {
                        Some((x as i64, y as i64).into())
                    } else {
                        None
                    }
                })
            })
            .collect())
    }

    fn solve(elves: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_empty_space(&elves).to_string();
        (Some(part_one), None)
    }
}
