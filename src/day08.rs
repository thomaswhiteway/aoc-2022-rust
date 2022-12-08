use failure::{err_msg, Error};
use itertools::iproduct;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn all() -> impl Iterator<Item = Self> {
        use Direction::*;
        [North, East, South, West].into_iter()
    }
}

pub struct HeightMap {
    heights: Box<[Box<[u32]>]>,
    width: usize,
    height: usize,
}

impl HeightMap {
    fn new(heights: Box<[Box<[u32]>]>) -> Self {
        let width = heights[0].len();
        let height = heights.len();
        HeightMap {
            heights,
            width,
            height,
        }
    }

    fn all_positions(&self) -> impl Iterator<Item = (usize, usize)> {
        iproduct!(0..self.width, 0..self.height)
    }

    fn positions_in_direction(
        &self,
        (x, y): (usize, usize),
        direction: Direction,
    ) -> Vec<(usize, usize)> {
        match direction {
            Direction::North => (0..y).map(|y2| (x, y2)).collect(),
            Direction::East => (x + 1..self.width).map(|x2| (x2, y)).collect(),
            Direction::South => (y + 1..self.height).map(|y2| (x, y2)).collect(),
            Direction::West => (0..x).map(|x2| (x2, y)).collect(),
        }
    }

    fn is_tree_visible_from_direction(&self, (x, y): (usize, usize), direction: Direction) -> bool {
        let tree_height = self.heights[y][x];
        !self
            .positions_in_direction((x, y), direction)
            .into_iter()
            .any(|(x2, y2)| self.heights[y2][x2] >= tree_height)
    }

    fn is_tree_visible(&self, position: (usize, usize)) -> bool {
        Direction::all().any(|direction| self.is_tree_visible_from_direction(position, direction))
    }
}

pub struct Solver {}

fn parse_height(c: char) -> Result<u32, Error> {
    c.to_digit(10)
        .ok_or_else(|| err_msg(format!("Invalid height {}", c)))
}

fn parse_line(line: &str) -> Result<Box<[u32]>, Error> {
    line.chars()
        .map(parse_height)
        .collect::<Result<Vec<_>, _>>()
        .map(|row| row.into_boxed_slice())
}

impl super::Solver for Solver {
    type Problem = HeightMap;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.lines()
            .map(parse_line)
            .collect::<Result<Vec<_>, _>>()
            .map(|rows| rows.into_boxed_slice())
            .map(HeightMap::new)
    }

    fn solve(map: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = map
            .all_positions()
            .filter(|&position| map.is_tree_visible(position))
            .count()
            .to_string();
        (Some(part_one), None)
    }
}
