use crate::common::Direction;
use failure::{err_msg, Error};
use itertools::iproduct;

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

    fn get_height(&self, (x, y): (usize, usize)) -> u32 {
        self.heights[y][x]
    }

    fn positions_in_direction(
        &self,
        (x, y): (usize, usize),
        direction: Direction,
    ) -> Vec<(usize, usize)> {
        match direction {
            Direction::North => (0..y).rev().map(|y2| (x, y2)).collect(),
            Direction::East => (x + 1..self.width).map(|x2| (x2, y)).collect(),
            Direction::South => (y + 1..self.height).map(|y2| (x, y2)).collect(),
            Direction::West => (0..x).rev().map(|x2| (x2, y)).collect(),
        }
    }

    fn is_tree_visible_from_direction(
        &self,
        position: (usize, usize),
        direction: Direction,
    ) -> bool {
        let tree_height = self.get_height(position);
        !self
            .positions_in_direction(position, direction)
            .into_iter()
            .any(|(x2, y2)| self.heights[y2][x2] >= tree_height)
    }

    fn is_tree_visible(&self, position: (usize, usize)) -> bool {
        Direction::all().any(|direction| self.is_tree_visible_from_direction(position, direction))
    }

    fn num_trees_visible_in_direction(
        &self,
        position: (usize, usize),
        direction: Direction,
    ) -> usize {
        let treehouse_height = self.get_height(position);
        let mut num_visible = 0;
        for position2 in self.positions_in_direction(position, direction) {
            num_visible += 1;
            if self.get_height(position2) >= treehouse_height {
                break;
            }
        }
        num_visible
    }

    fn scenic_score(&self, position: (usize, usize)) -> usize {
        Direction::all()
            .map(|direction| self.num_trees_visible_in_direction(position, direction))
            .product()
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

        let part_two = map
            .all_positions()
            .map(|position| map.scenic_score(position))
            .max()
            .unwrap()
            .to_string();

        (Some(part_one), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use crate::Solver;

    #[test]
    fn test_score() {
        let data = r"30373
25512
65332
33549
35390
"
        .to_string();
        let map = super::Solver::parse_input(data).unwrap();
        assert_eq!(map.scenic_score((2, 1)), 4);
    }

    #[test]
    fn test_visible() {
        let data = r"30373
25512
65332
33549
35390
"
        .to_string();
        let map = super::Solver::parse_input(data).unwrap();

        assert_eq!(
            map.num_trees_visible_in_direction((2, 1), super::Direction::North),
            1
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 1), super::Direction::East),
            2
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 1), super::Direction::South),
            2
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 1), super::Direction::West),
            1
        );
    }

    #[test]
    fn test_score2() {
        let data = r"30373
25512
65332
33549
35390
"
        .to_string();
        let map = super::Solver::parse_input(data).unwrap();
        assert_eq!(map.scenic_score((2, 3)), 8);
    }

    #[test]
    fn test_visible2() {
        let data = r"30373
25512
65332
33549
35390
"
        .to_string();
        let map = super::Solver::parse_input(data).unwrap();

        assert_eq!(
            map.num_trees_visible_in_direction((2, 3), super::Direction::North),
            2
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 3), super::Direction::East),
            2
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 3), super::Direction::South),
            1
        );
        assert_eq!(
            map.num_trees_visible_in_direction((2, 3), super::Direction::West),
            2
        );
    }
}
