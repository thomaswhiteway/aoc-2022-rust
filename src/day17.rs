use failure::{err_msg, Error};
use std::{
    cmp::{max, min},
    collections::HashMap,
    collections::HashSet,
    ops::Range,
    str::FromStr,
};

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
    height: i64,
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
        let height = offsets.iter().map(|pos| pos.y).max().unwrap() + 1;

        Ok(Rock {
            offsets,
            width,
            height,
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Collision {
    Wall,
    Floor,
    Rocks(usize),
}

struct Tower {
    filled: HashMap<Position, usize>,
    max_y: i64,
    width: i64,
}

impl Tower {
    fn new(width: i64) -> Self {
        Tower {
            filled: HashMap::new(),
            max_y: -1,
            width,
        }
    }

    fn height(&self) -> i64 {
        self.max_y + 1
    }

    fn add_rock(&mut self, rock: &Rock, position: Position, index: usize) {
        let positions = rock.positions_at(position).collect::<Vec<_>>();
        self.max_y = max(self.max_y, positions.iter().map(|pos| pos.y).max().unwrap());
        self.filled
            .extend(positions.into_iter().map(|position| (position, index)));
    }

    fn check_collision(&self, rock: &Rock, position: Position) -> Option<Collision> {
        if position.y < 0 {
            Some(Collision::Floor)
        } else if position.x < 0 || position.x + rock.width > self.width {
            Some(Collision::Wall)
        } else {
            let rocks = rock
                .positions_at(position)
                .filter_map(|pos| self.filled.get(&pos));

            rocks.max().map(|latest| Collision::Rocks(*latest))
        }
    }

    fn can_fit(&self, rock: &Rock, position: Position) -> bool {
        self.check_collision(rock, position).is_none()
    }

    #[allow(unused)]
    fn draw(&self, rock: Option<(&Rock, Position)>, rows: usize) {
        let rock_positions = if let Some((rock, position)) = rock {
            rock.positions_at(position).collect::<HashSet<_>>()
        } else {
            HashSet::new()
        };

        for y in (0..=self.max_y + 4).rev().take(rows) {
            print!("|");
            for x in 0..self.width {
                let position = (x, y).into();
                if rock_positions.contains(&position) {
                    print!("@");
                } else if self.filled.contains_key(&position) {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!("|");
        }
        if self.height() as usize > rows {
            println!("...\n\n");
        } else {
            print!("+");
            for _ in 0..self.width {
                print!("-");
            }
            print!("+\n\n")
        }
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Action<T> {
    Stop(T),
    Continue,
}

struct FallenRock<'a> {
    position: Position,
    rock: &'a Rock,
    collision: Collision,
}

trait Watcher {
    type Output;
    fn watch(
        &mut self,
        old_state: &State,
        new_state: &State,
        fallen_rock: &FallenRock,
    ) -> Action<Self::Output>;
}

fn move_sideways(position: &mut Position, rock: &Rock, direction: Direction, tower: &Tower) {
    let next_position = *position + direction.offset();

    if tower.can_fit(rock, next_position) {
        *position = next_position;
    }
}

fn move_down(position: &mut Position, rock: &Rock, tower: &Tower) -> Option<Collision> {
    let next_position = Position {
        x: position.x,
        y: position.y - 1,
    };
    if let Some(collision) = tower.check_collision(rock, next_position) {
        Some(collision)
    } else {
        *position = next_position;
        None
    }
}

fn drop_rock<'a>(
    rock: &'a Rock,
    jets: &mut impl Iterator<Item = Direction>,
    tower: &Tower,
    from: Position,
) -> (usize, FallenRock<'a>) {
    let mut position = from;

    for (index, jet) in jets.enumerate() {
        move_sideways(&mut position, rock, jet, tower);
        if let Some(collision) = move_down(&mut position, rock, tower) {
            return (
                index + 1,
                FallenRock {
                    position,
                    rock,
                    collision,
                },
            );
        }
    }

    panic!("Ran out of jets")
}

#[derive(Default, Clone)]
struct State {
    num_rocks: usize,
    num_steps: usize,
    height: i64,
}

impl State {
    fn update(&self, num_steps: usize, tower: &Tower) -> Self {
        State {
            num_rocks: self.num_rocks + 1,
            num_steps: self.num_steps + num_steps,
            height: tower.height(),
        }
    }
}

fn drop_rocks<'a, W: Watcher>(
    rocks: impl Iterator<Item = &'a Rock>,
    mut jets: impl Iterator<Item = Direction>,
    mut watcher: W,
    display: Draw,
) -> W::Output {
    let mut tower = Tower::new(TOWER_WIDTH);
    let mut state = State::default();

    for (dropped_rocks, rock) in rocks.enumerate() {
        let drop_position = Position {
            x: 2,
            y: tower.max_y + 4,
        };
        let (num_steps, fallen_rock) = drop_rock(rock, &mut jets, &tower, drop_position);

        display.draw_tower(dropped_rocks + 1, &tower, &fallen_rock);

        tower.add_rock(rock, fallen_rock.position, dropped_rocks + 1);

        let new_state = state.update(num_steps, &tower);

        if let Action::Stop(outcome) = watcher.watch(&state, &new_state, &fallen_rock) {
            return outcome;
        }

        state = new_state;
    }
    panic!("Ran out of rocks");
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct CycleIndex {
    rock_index: usize,
    jet_index: usize,
    x: i64,
}

impl CycleIndex {
    fn new(rock_index: usize, jet_index: usize, x: i64) -> Self {
        CycleIndex {
            rock_index,
            jet_index,
            x,
        }
    }
}

enum Draw {
    Never,
    #[allow(unused)]
    Ranges(Vec<Range<usize>>),
}

impl Draw {
    fn draw_tower(&self, dropped_rocks: usize, tower: &Tower, state: &FallenRock<'_>) {
        if let Draw::Ranges(ranges) = self {
            if ranges.iter().any(|range| range.contains(&dropped_rocks)) {
                println!("After {} rocks:", dropped_rocks + 1);
                tower.draw(Some((state.rock, state.position)), 20);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Segment {
    height_deltas: Vec<i64>,
}

impl Segment {
    fn total_height(&self) -> i64 {
        *self.height_deltas.last().unwrap()
    }

    fn height_after_rocks(&self, num_rocks: usize) -> i64 {
        if num_rocks > 0 {
            self.height_deltas[num_rocks - 1]
        } else {
            0
        }
    }

    fn cycle_height_after_rocks(&self, num_rocks: usize) -> i64 {
        (num_rocks / self.num_rocks()) as i64 * self.total_height()
            + self.height_after_rocks(num_rocks % self.num_rocks())
    }

    fn num_rocks(&self) -> usize {
        self.height_deltas.len()
    }
}

struct GetHeightAfter {
    num_rocks: usize,
}

impl GetHeightAfter {
    #[allow(unused)]
    fn new(num_rocks: usize) -> Self {
        GetHeightAfter { num_rocks }
    }
}

impl Watcher for GetHeightAfter {
    type Output = i64;
    fn watch(
        &mut self,
        _old_state: &State,
        new_state: &State,
        _fallen_rock: &FallenRock,
    ) -> Action<Self::Output> {
        if new_state.num_rocks == self.num_rocks {
            Action::Stop(new_state.height)
        } else {
            Action::Continue
        }
    }
}

struct CycleFinder {
    rock_cycle_len: usize,
    jet_cycle_len: usize,
    visited: HashMap<CycleIndex, usize>,
    heights: Vec<i64>,
}

impl CycleFinder {
    fn new(rock_cycle_len: usize, jet_cycle_len: usize) -> Self {
        CycleFinder {
            rock_cycle_len,
            jet_cycle_len,
            visited: HashMap::default(),
            heights: Vec::new(),
        }
    }

    fn cycle_index(&self, state: &State, fallen_rock: &FallenRock) -> CycleIndex {
        CycleIndex::new(
            state.num_rocks % self.rock_cycle_len,
            state.num_steps % self.jet_cycle_len,
            fallen_rock.position.x,
        )
    }

    fn segment(&self, range: Range<usize>) -> Segment {
        let initial_height = if range.start == 0 {
            0
        } else {
            self.heights[range.start - 1]
        };
        Segment {
            height_deltas: self.heights[range]
                .iter()
                .map(|height| height - initial_height)
                .collect(),
        }
    }
}

impl Watcher for CycleFinder {
    type Output = (Segment, Segment);
    fn watch(
        &mut self,
        old_state: &State,
        new_state: &State,
        fallen_rock: &FallenRock,
    ) -> Action<Self::Output> {
        self.heights.push(new_state.height);

        if let Collision::Rocks(latest) = fallen_rock.collision {
            self.visited.retain(|_, num_rocks| *num_rocks <= latest);
        } else if fallen_rock.collision == Collision::Floor {
            self.visited.clear();
        }

        // Only consider starting a cycle where a rock has fallen in a way where
        // there's a clean break between that rock and any previous rocks.
        if new_state.height - old_state.height == fallen_rock.rock.height {
            let cycle_index = self.cycle_index(new_state, fallen_rock);

            if let Some(prefix_len) = self.visited.insert(cycle_index, new_state.num_rocks) {
                let prefix = self.segment(0..prefix_len);
                let cycle = self.segment(prefix_len..new_state.num_rocks);
                Action::Stop((prefix, cycle))
            } else {
                Action::Continue
            }
        } else {
            Action::Continue
        }
    }
}

fn find_prefix_and_cycle_time(jets: &[Direction], rocks: &[Rock]) -> (Segment, Segment) {
    drop_rocks(
        rocks.iter().cycle(),
        jets.iter().cloned().cycle(),
        CycleFinder::new(rocks.len(), jets.len()),
        Draw::Never,
    )
}

fn find_height_after(rocks: &[Rock], jets: &[Direction], num_rocks: usize) -> i64 {
    let (prefix, cycle) = find_prefix_and_cycle_time(jets, rocks);

    let prefix_rocks = min(prefix.num_rocks(), num_rocks);
    let cycle_rocks = num_rocks - prefix_rocks;

    prefix.height_after_rocks(prefix_rocks) + cycle.cycle_height_after_rocks(cycle_rocks)
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
