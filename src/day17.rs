use std::{collections::HashSet, str::FromStr, cmp::max};

use failure::{Error, err_msg};

use crate::common::Position;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right
}

impl Direction {
    fn offset(&self) -> Position {
        match self {
            Direction::Left => Position{ x: -1, y: 0 },
            Direction::Right => Position{ x: 1, y: 0 },
        }
    }
}

impl TryFrom<char> for Direction {
    type Error =  Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '<' => Ok(Direction::Left),
            '>' => Ok(Direction::Right),
            _ => Err(err_msg(format!("Unknown jet direction {}", value))),
        }
    }
}

#[derive(Debug)]
struct Rock {
    offsets: Vec<Position>,
    width: i64,
}

impl Rock {
    fn positions_at(&self, position: Position) -> impl Iterator<Item=Position> + '_ {
        self.offsets.iter().map(move |offset| position + *offset)
    }
}

impl FromStr for Rock {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let offsets: Vec<Position> = s
            .lines()
            .rev()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars().enumerate().filter_map(move |(x, c)| {
                    if c == '#' {
                        Some(Position {
                            x: x as i64,
                            y: y as i64,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();
        let width = offsets.iter().map(|pos| pos.x).max().unwrap() + 1;
        Ok(Rock { offsets, width })
    }
}

struct Tower {
    filled: HashSet<Position>,
    max_y: i64,
    width: i64,
}

#[allow(unused)]
fn draw(tower: &Tower, rock: Option<(&Rock, Position)>) {
    let rock_positions = if let Some((rock, position)) = rock {
        rock.positions_at(position).collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };

    for y in (0..=tower.max_y+4).rev() {
        print!("|");
        for x in 0..tower.width {
            let position = (x, y).into();
            if rock_positions.contains(&position) {
                print!("@");
            } else if tower.filled.contains(&position) {
                print!("#");
            } else {
                print!(".");
            }
        }
        print!("|\n");
    }
    print!("+");
    for _ in 0..tower.width {
        print!("-");
    }
    print!("+\n\n")
}

impl Tower {
    fn new(width: i64) -> Self {
        Tower {
            filled: HashSet::new(),
            max_y: -1,
            width,
        }
    }

    fn add_rock(&mut self, positions: &[Position]) {
        assert!(positions.iter().all(|pos| pos.x >= 0 && pos.x < self.width));
        self.max_y = max(self.max_y, positions.iter().map(|pos| pos.y).max().unwrap());
        self.filled.extend(positions);
    }

    fn can_fit(&self, rock: &Rock, position: Position) -> bool {
        position.y >= 0 && position.x >= 0 && position.x + rock.width <= self.width && self.filled.is_disjoint(&rock.positions_at(position).collect::<HashSet<_>>())
    }
}


fn get_rocks() -> Box<[Rock]> {
    ["####",
     ".#.\n###\n.#.",
     "..#\n..#\n###",
     "#\n#\n#\n#",
     "##\n##"].into_iter().map(|rock| rock.parse().unwrap()).collect::<Vec<_>>().into_boxed_slice()
}

fn move_sideways(position: &mut Position, rock: &Rock, direction: Direction, tower: &Tower) {
    let next_position = *position + direction.offset();

    if tower.can_fit(rock, next_position) {
        *position = next_position;
    }
}

fn move_down(position: &mut Position, rock: &Rock, tower: &Tower) -> bool {
    let next_position = Position { x: position.x, y: position.y - 1};
    if tower.can_fit(rock, next_position) {
        *position = next_position;
        true
    } else {
        false
    }
}

fn drop_rock(rock: &Rock, jets: &mut impl Iterator<Item=Direction>, tower: &Tower, from: Position) -> Vec<Position> {
    let mut position = from;

    for jet in jets {
        move_sideways(&mut position, rock, jet, tower);
        if !move_down(&mut position, rock, tower) {
            break;
        }
    }

    rock.positions_at(position).collect()
}

fn find_height_after(mut jets: impl Iterator<Item=Direction>, num_rocks: usize) -> i64 {
    let rocks = get_rocks();
    let mut tower = Tower::new(7);
    for rock in rocks.iter().cycle().take(num_rocks) {
        let end_position = drop_rock(rock, &mut jets, &tower, Position { x: 2, y: tower.max_y + 4});
        tower.add_rock(&end_position);
    }
    tower.max_y + 1
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Direction]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.trim().chars().map(Direction::try_from).collect::<Result<Vec<_>,_>>().map(Vec::into_boxed_slice)
    }

    fn solve(jets: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_height_after(jets.iter().cloned().cycle(), 2022).to_string();
        (Some(part_one), None)
    }
}
