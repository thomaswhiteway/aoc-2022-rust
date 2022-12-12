use std::{cmp::max, collections::HashMap, fmt::Debug, hash::Hash, str::FromStr};

use crate::a_star;
use crate::common::{Direction, Position};
use failure::{err_msg, Error};

pub struct HeightMap {
    heights: HashMap<Position, u8>,
    start: Position,
    end: Position,
    top_left: Position,
    bottom_right: Position,
}

fn read_height_chars(input: &str) -> impl Iterator<Item = (Position, char)> + '_ {
    input.lines().enumerate().flat_map(|(y, row)| {
        row.chars().enumerate().map(move |(x, h)| {
            (
                Position {
                    x: x as i64,
                    y: y as i64,
                },
                h,
            )
        })
    })
}

fn get_height(h: char) -> Result<u8, Error> {
    let actual_h = match h {
        'S' => 'a',
        'E' => 'z',
        'a'..='z' => h,
        _ => return Err(err_msg(format!("Invalid height {}", h))),
    };

    Ok(actual_h as u8 - 'a' as u8)
}

fn is_start(h: char) -> bool {
    h == 'S'
}

fn is_end(h: char) -> bool {
    h == 'E'
}

impl FromStr for HeightMap {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut heights = HashMap::new();
        let mut start = None;
        let mut end = None;
        let mut max_x = 0;
        let mut max_y = 0;

        for (position, h) in read_height_chars(s) {
            let height = get_height(h)?;
            if is_start(h) {
                start = Some(position);
            } else if is_end(h) {
                end = Some(position);
            }

            if position.x > max_x {
                max_x = position.x;
            }
            if position.y > max_y {
                max_y = position.y;
            }

            heights.insert(position, height);
        }

        Ok(HeightMap {
            heights,
            start: start.ok_or_else(|| err_msg("Start position not specified"))?,
            end: end.ok_or_else(|| err_msg("End position not specified"))?,
            top_left: Position { x: 0, y: 0 },
            bottom_right: Position { x: max_x, y: max_y },
        })
    }
}

#[derive(Clone)]
struct State<'a> {
    height_map: &'a HeightMap,
    position: Position,
}

impl<'a> Debug for State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.position.x, self.position.y)
    }
}

impl<'a> State<'a> {
    fn new(height_map: &'a HeightMap, position: Position) -> Self {
        State {
            height_map,
            position,
        }
    }
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl<'a> Eq for State<'a> {}

impl<'a> Hash for State<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state)
    }
}

impl<'a> a_star::State for State<'a> {
    fn heuristic(&self) -> u64 {
        return (self.height_map.heights.get(&self.height_map.end).unwrap()
            - self.height_map.heights.get(&self.position).unwrap()) as u64;
        // TODO: Figure out why this doesn't work
        max(
            self.position.manhattan_distance_to(&self.height_map.end),
            (self.height_map.heights.get(&self.height_map.end).unwrap()
                - self.height_map.heights.get(&self.position).unwrap()) as u64,
        )
    }

    fn successors(&self) -> Vec<(u64, Self)> {
        let current_height = *self.height_map.heights.get(&self.position).unwrap();
        self.position
            .adjacent()
            .filter_map(|position| {
                self.height_map.heights.get(&position).and_then(|&height| {
                    if height <= current_height + 1 {
                        Some((
                            1_u64,
                            State {
                                height_map: self.height_map,
                                position,
                            },
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

fn height_char(height: u8) -> char {
    ('a' as u8 + height) as char
}

fn display_route<'a>(height_map: &HeightMap, route: Vec<State<'a>>) {
    let directions: HashMap<Position, Direction> = route
        .iter()
        .zip(route.iter().skip(1))
        .map(|(state, next_state)| {
            (
                state.position.clone(),
                state.position.direction_to(&next_state.position).unwrap(),
            )
        })
        .collect();
    for y in height_map.top_left.y..=height_map.bottom_right.y {
        let row: String = (height_map.top_left.x..=height_map.bottom_right.x)
            .map(|x| Position { x, y })
            .map(|position| {
                directions
                    .get(&position)
                    .map(|dir| dir.as_char())
                    .or_else(|| height_map.heights.get(&position).cloned().map(height_char))
                    .unwrap_or(' ')
            })
            .collect();
        println!("{}", row);
    }
}

fn find_shortest_route_from(height_map: &HeightMap, start: Position) -> Option<u64> {
    let start = State::new(height_map, start);
    let end = State::new(height_map, height_map.end.into());
    let (distance, _route) = a_star::solve(start, end)?;
    Some(distance)
}

fn all_start_points(height_map: &HeightMap) -> Vec<Position> {
    height_map
        .heights
        .iter()
        .filter_map(|(position, height)| if *height == 0 { Some(*position) } else { None })
        .collect()
}

fn find_shortest_route(height_map: &HeightMap, starts: Vec<Position>) -> Option<u64> {
    starts
        .into_iter()
        .filter_map(|start| find_shortest_route_from(height_map, start))
        .min()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = HeightMap;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.parse()
    }

    fn solve(height_map: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_shortest_route(&height_map, vec![height_map.start])
            .expect("Failed to solve part one")
            .to_string();

        let part_two = find_shortest_route(&height_map, all_start_points(&height_map))
            .expect("Failed to solve part one")
            .to_string();
        (Some(part_one), Some(part_two))
    }
}
