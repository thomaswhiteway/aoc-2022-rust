use failure::{err_msg, Error};
use std::collections::HashMap;
use std::{cmp::max, collections::HashSet, str::FromStr};

use crate::common::Position;

const TOWER_WIDTH: i64 = 7;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn offset(&self) -> Position {
        match self {
            Direction::Left => Position { x: -1, y: 0 },
            Direction::Right => Position { x: 1, y: 0 },
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = Error;
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
    fn positions_at(&self, position: Position) -> impl Iterator<Item = Position> + '_ {
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
fn draw(tower: &Tower, rock: Option<(&Rock, Position)>, rows: usize) {
    let rock_positions = if let Some((rock, position)) = rock {
        rock.positions_at(position).collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };

    for y in (0..=tower.max_y + 4).rev().take(rows) {
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
    if tower.height() as usize > rows {
        println!("...\n\n");
    } else {
        print!("+");
        for _ in 0..tower.width {
            print!("-");
        }
        print!("+\n\n")
    }
}

impl Tower {
    fn new(width: i64) -> Self {
        Tower {
            filled: HashSet::new(),
            max_y: -1,
            width,
        }
    }

    fn height(&self) -> i64 {
        self.max_y + 1
    }

    fn add_rock(&mut self, positions: &[Position]) {
        assert!(positions.iter().all(|pos| pos.x >= 0 && pos.x < self.width));
        self.max_y = max(self.max_y, positions.iter().map(|pos| pos.y).max().unwrap());
        self.filled.extend(positions);
    }

    fn can_fit(&self, rock: &Rock, position: Position) -> bool {
        position.y >= 0
            && position.x >= 0
            && position.x + rock.width <= self.width
            && self
                .filled
                .is_disjoint(&rock.positions_at(position).collect::<HashSet<_>>())
    }
}

struct SimulationResult {
    num_rocks: usize,
    num_steps: usize,
    tower: Tower,
}

fn get_rocks() -> Box<[Rock]> {
    [
        "####",
        ".#.\n###\n.#.",
        "..#\n..#\n###",
        "#\n#\n#\n#",
        "##\n##",
    ]
    .into_iter()
    .map(|rock| rock.parse().unwrap())
    .collect::<Vec<_>>()
    .into_boxed_slice()
}

fn move_sideways(position: &mut Position, rock: &Rock, direction: Direction, tower: &Tower) {
    let next_position = *position + direction.offset();

    if tower.can_fit(rock, next_position) {
        *position = next_position;
    }
}

fn move_down(position: &mut Position, rock: &Rock, tower: &Tower) -> bool {
    let next_position = Position {
        x: position.x,
        y: position.y - 1,
    };
    if tower.can_fit(rock, next_position) {
        *position = next_position;
        true
    } else {
        false
    }
}

fn drop_rock(
    rock: &Rock,
    jets: &mut impl Iterator<Item = Direction>,
    tower: &Tower,
    from: Position,
) -> (usize, Position) {
    let mut position = from;

    for (step, jet) in jets.enumerate() {
        move_sideways(&mut position, rock, jet, tower);
        if !move_down(&mut position, rock, tower) {
            return (step + 1, position);
        }
    }

    panic!("Ran out of jets")
}

fn simulate_dropping_rocks<'a, F, G>(
    rocks: impl Iterator<Item = &'a Rock>,
    mut jets: impl Iterator<Item = Direction>,
    mut stop: F,
    display: G,
) -> SimulationResult
where
    F: FnMut(usize, usize, &Tower, Position) -> bool,
    G: Fn(usize) -> bool
{
    let mut tower = Tower::new(TOWER_WIDTH);
    let mut total_steps = 0;
    for (dropped_rocks, rock) in rocks.enumerate() {
        let (steps, end_position) = drop_rock(
            rock,
            &mut jets,
            &tower,
            Position {
                x: 2,
                y: tower.max_y + 4,
            },
        );
        total_steps += steps;

        if display(dropped_rocks + 1) {
            println!("After {} rocks:", dropped_rocks + 1);
            draw(&tower, Some((rock, end_position)), 20);
        }

        tower.add_rock(&rock.positions_at(end_position).collect::<Vec<_>>());
        //println!("{},{},{},{},{}", dropped_rocks + 1, total_steps, tower.height(), end_position.x, end_position.y);
        if stop(dropped_rocks + 1, total_steps, &tower, end_position) {
            return SimulationResult {
                num_rocks: dropped_rocks + 1,
                num_steps: total_steps,
                tower,
            };
        }
    }
    panic!("Ran out of rocks");
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct CycleIndex {
    rock_index: usize,
    jet_index: usize,
    x: i64
}

impl CycleIndex {
    fn new(rock_index: usize, jet_index: usize, x: i64) -> Self {
        CycleIndex {
            rock_index,
            jet_index,
            x
        }
    }
}

struct CycleState {
    num_dropped_rocks: usize,
    num_steps: usize,
    tower_height: i64,
    last_rock_position: Position,
}

#[derive(Debug, PartialEq, Eq)]
struct Segment {
    num_rocks: usize,
    num_steps: usize,
    height_delta: i64,
}

impl From<SimulationResult> for Segment {
    fn from(res: SimulationResult) -> Self {
        Segment {
            num_rocks: res.num_rocks,
            num_steps: res.num_steps,
            height_delta: res.tower.height(),
        }
    }
}

fn find_prefix_and_cycle_time(jets: &[Direction], rocks: &[Rock]) -> (Segment, Segment) {
    let mut visited: HashMap<CycleIndex, CycleState> = HashMap::new();
    let mut prefix = None;

    let can_start_cycle =
        |num_dropped_rocks: usize, last_rock_position: Position, tower: &Tower| {
            num_dropped_rocks % rocks.len() == 1
                && last_rock_position.x > 0
                && last_rock_position.x < 3
                && (0..tower.width)
                    .filter(|x| {
                        tower.filled.contains(&Position {
                            x: *x,
                            y: last_rock_position.y,
                        })
                    })
                    .count()
                    == 4
        };

    let outcome = simulate_dropping_rocks(
        rocks.iter().cycle(),
        jets.iter().cloned().cycle(),
        |num_dropped_rocks, num_steps, tower, last_rock_position| {
            visited.retain(|_, state| state.last_rock_position.y < last_rock_position.y);
            if can_start_cycle(num_dropped_rocks, last_rock_position, tower) {
                let key = CycleIndex::new(num_dropped_rocks % rocks.len(), num_steps % jets.len(), last_rock_position.x);
                let state = CycleState {
                    num_dropped_rocks,
                    num_steps,
                    tower_height: tower.height(),
                    last_rock_position,
                };
                if let Some(prefix_state) = visited.insert(key, state) {
                    prefix = Some(Segment {
                        num_rocks: prefix_state.num_dropped_rocks,
                        num_steps: prefix_state.num_steps,
                        height_delta: prefix_state.tower_height,
                    });
                    true
                } else {
                    false
                }
            } else {
                false
            }
        },
        |_| false
    );

    let prefix = prefix.unwrap();
    let cycle = Segment {
        num_rocks: outcome.num_rocks - prefix.num_rocks,
        num_steps: outcome.num_steps - prefix.num_steps,
        height_delta: outcome.tower.height() - prefix.height_delta,
    };
    (prefix, cycle)
}

fn find_height_after_by_simulating<'a>(
    rocks: impl Iterator<Item = &'a Rock>,
    jets: impl Iterator<Item = Direction>,
    num_rocks: usize,
) -> i64 {
    simulate_dropping_rocks(rocks, jets, |dropped_rocks, _, _, _| {
        dropped_rocks == num_rocks
    }, |_| false)
    .tower
    .height()
}

fn find_height_after(rocks: &[Rock], jets: &[Direction], mut num_rocks: usize) -> i64 {
    let (prefix, cycle) = find_prefix_and_cycle_time(jets, &rocks);

    let mut total_height = 0;
    if num_rocks > prefix.num_rocks {
        total_height += prefix.height_delta;
        num_rocks -= prefix.num_rocks;

        total_height += (num_rocks / cycle.num_rocks) as i64 * cycle.height_delta;
        num_rocks %= cycle.num_rocks;
    }

    total_height += find_height_after_by_simulating(
        rocks.iter().cycle().skip(1),
        jets.iter()
            .cloned()
            .cycle()
            .skip(prefix.num_steps % jets.len()),
        num_rocks,
    );

    total_height
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Direction]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.trim()
            .chars()
            .map(Direction::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn solve(jets: Self::Problem) -> (Option<String>, Option<String>) {
        let rocks = get_rocks();
        let part_one = find_height_after(&rocks, &jets, 2022).to_string();
        let part_two = find_height_after(&rocks, &jets, 1000000000000).to_string();
        (Some(part_one), Some(part_two))
    }
}
