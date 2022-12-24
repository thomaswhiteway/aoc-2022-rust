use std::{array, hash::Hash, str::FromStr};

use failure::Error;
use itertools::chain;

use crate::{
    a_star,
    common::{Direction, Position},
};

#[derive(Debug)]
pub struct Map {
    blizzards: [Box<[Box<[i64]>]>; 4],
    height: i64,
    width: i64,
    start: Position,
    end: Position,
}

impl Map {
    fn blizzards_in_direction(&self, direction: Direction, row_or_col: i64) -> &[i64] {
        &self.blizzards[direction as usize][row_or_col as usize]
    }

    fn blizzards_in_direction_at_time(
        &self,
        direction: Direction,
        row_or_col: i64,
        time: i64,
    ) -> impl Iterator<Item = i64> + '_ {
        let (modulo, offset) = match direction {
            Direction::North => (self.height, self.height - 1),
            Direction::East => (self.width, 1),
            Direction::South => (self.height, 1),
            Direction::West => (self.width, self.width - 1),
        };

        self.blizzards_in_direction(direction, row_or_col)
            .iter()
            .map(move |pos| (pos + time * offset) % modulo)
    }

    fn is_free_at_time(&self, position: Position, time: i64) -> bool {
        if position == self.start || position == self.end {
            return true;
        }
        if position.x < 0 || position.y < 0 || position.x >= self.width || position.y >= self.height {
            return false;
        }
        Direction::all().all(|direction| {
            let (row_or_col, check) = match direction {
                Direction::North | Direction::South => (position.x, position.y),
                Direction::East | Direction::West => (position.y, position.x),
            };

            self.blizzards_in_direction_at_time(direction, row_or_col, time)
                .all(|pos| pos != check)
        })
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s
            .lines()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let height = grid.len() as i64 - 2;
        let width = grid[0].len() as i64 - 2;

        assert!(grid[0][1] == '.');
        let start = Position { x: 0, y: -1 };
        assert!(grid[height as usize + 1][width as usize] == '.');
        let end = Position {
            x: width - 1,
            y: height,
        };

        let blizzards = array::from_fn(|d| {
            let direction = Direction::try_from(d).unwrap();
            let (outer_len, inner_len, outer_is_x) = match direction {
                Direction::North | Direction::South => (width, height, true),
                Direction::East | Direction::West => (height, width, false),
            };

            (0..outer_len)
                .map(|outer| {
                    (0..inner_len)
                        .filter(|inner| {
                            let (x, y) = if outer_is_x {
                                (outer, *inner)
                            } else {
                                (*inner, outer)
                            };
                            grid[y as usize + 1][x as usize + 1] == direction.as_char()
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        });

        Ok(Map {
            blizzards,
            height,
            width,
            start,
            end,
        })
    }
}

#[derive(Debug, Clone)]
struct State<'a> {
    map: &'a Map,
    position: Position,
    time: i64,
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.time == other.time
    }
}

impl<'a> Eq for State<'a> {}

impl<'a> Hash for State<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.position, self.time).hash(state)
    }
}

impl<'a> a_star::State for State<'a> {
    fn heuristic(&self) -> u64 {
        self.position.manhattan_distance_to(&self.map.end)
    }

    fn is_end(&self) -> bool {
        self.position == self.map.end
    }

    fn successors(&self) -> Vec<(u64, Self)> {
        let time = self.time + 1;
        chain!([self.position], self.position.adjacent())
            .filter(|position| self.map.is_free_at_time(*position, time))
            .map(|position| {
                (
                    1,
                    State {
                        map: self.map,
                        position,
                        time,
                    },
                )
            })
            .collect()
    }
}

fn find_quickest_route(map: &Map) -> Option<u64> {
    let start = State {
        map,
        position: map.start,
        time: 0,
    };
    a_star::solve(start).map(|(min_time, _)| min_time).ok()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Map;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.parse()
    }

    fn solve(map: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_quickest_route(&map)
            .expect("Failed to solve part one")
            .to_string();
        (Some(part_one), None)
    }
}


#[cfg(test)]
mod test {
    use super::Map;
    use crate::common::{Position, Direction};
    use std::collections::HashSet;

    #[test]
    fn test_parse() {
        let map_string = r#"#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#
"#;
        let map: Map = map_string.parse().unwrap();
        assert_eq!(map.start, Position { x: 0, y: -1});
        assert_eq!(map.end, Position { x: 5, y: 4});
        assert_eq!(map.width, 6);
        assert_eq!(map.height, 4);
        assert_eq!(map.blizzards, [
            vec![
                vec![].into_boxed_slice(),
                vec![3].into_boxed_slice(),
                vec![].into_boxed_slice(),
                vec![3].into_boxed_slice(),
                vec![0, 3].into_boxed_slice(),
                vec![].into_boxed_slice(),
            ].into_boxed_slice(),
            vec![
                vec![0, 1].into_boxed_slice(),
                vec![].into_boxed_slice(),
                vec![0, 3, 5].into_boxed_slice(),
                vec![5].into_boxed_slice(),
            ].into_boxed_slice(),
            vec![
                vec![].into_boxed_slice(),
                vec![2].into_boxed_slice(),
                vec![3].into_boxed_slice(),
                vec![].into_boxed_slice(),
                vec![].into_boxed_slice(),
                vec![].into_boxed_slice(),
            ].into_boxed_slice(),
            vec![
                vec![3, 5].into_boxed_slice(),
                vec![1, 4,5].into_boxed_slice(),
                vec![4].into_boxed_slice(),
                vec![0].into_boxed_slice(),
            ].into_boxed_slice(),
        ])
    }

    #[test]
    fn test_free_initial() {
        let map_string = r#"#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#
"#;
        let map: Map = map_string.parse().unwrap();
        let free: HashSet<Position> = (-1..).zip(map_string.lines()).flat_map(
            |(y, line)| (-1..).zip(line.chars()).filter_map(move |(x, c)| if c == '.' { Some(Position { x, y})} else { None } )
        ).collect();

        for y in -1..5 {
            for x in -1..7 {
                let position = Position {x, y};
                let is_free = map.is_free_at_time(position, 0);
                let should_be_free = free.contains(&position);
                if is_free != should_be_free {
                    eprintln!("Position {:?} incorrect, should be free: {}, is free: {}", position, should_be_free, is_free);
                    for direction in Direction::all() {
                        eprintln!("{:?}: {:?}", direction, map.blizzards[direction as usize]);
                    }
                    panic!();
                }
            }
        }
    }
}
