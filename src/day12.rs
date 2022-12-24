use std::collections::HashSet;
use std::{cmp::max, collections::HashMap, fmt::Debug, hash::Hash, str::FromStr};

use crate::a_star;
use crate::common::{Direction, Position};
use failure::{err_msg, Error};

pub struct HeightMap {
    heights: HashMap<Position, u8>,
    start: Position,
    end: Position,
    #[allow(unused)]
    top_left: Position,
    #[allow(unused)]
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

    Ok(actual_h as u8 - b'a')
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
        #[allow(unreachable_code)]
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

    fn is_end(&self) -> bool {
        self.position == self.height_map.end
    }
}

#[allow(unused)]
fn height_char(height: u8) -> char {
    (b'a' + height) as char
}

#[allow(unused)]
fn display_route(height_map: &HeightMap, route: Vec<State<'_>>) {
    let directions: HashMap<Position, Direction> = route
        .iter()
        .zip(route.iter().skip(1))
        .map(|(state, next_state)| {
            (
                state.position,
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

fn find_shortest_route_from(
    height_map: &HeightMap,
    start: Position,
) -> Result<u64, HashSet<Position>> {
    let start = State::new(height_map, start);

    a_star::solve(start)
        .map(|(distance, _route)| distance)
        .map_err(|visited| visited.into_iter().map(|state| state.position).collect())
}

fn all_start_points(height_map: &HeightMap) -> Vec<Position> {
    height_map
        .heights
        .iter()
        .filter_map(|(position, height)| if *height == 0 { Some(*position) } else { None })
        .collect()
}

fn find_shortest_route(height_map: &HeightMap, mut starts: Vec<Position>) -> Option<u64> {
    let mut best = None;

    while let Some(start) = starts.pop() {
        match find_shortest_route_from(height_map, start) {
            Ok(distance) => {
                if best.map(|best| distance < best).unwrap_or(true) {
                    best = Some(distance)
                }
            }
            Err(visited) => {
                starts.retain(|start| !visited.contains(start));
            }
        }
    }

    best
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
