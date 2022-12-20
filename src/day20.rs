use std::{
    cmp::Ordering,
    fmt::Display,
    iter::repeat,
    num::ParseIntError,
    ops::{Index, IndexMut},
};

use failure::Error;
use itertools::Itertools;

fn modulo(x: isize, m: usize) -> usize {
    ((x % m as isize + if x < 0 { m as isize } else { 0 }) as usize) % m
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircularBuffer<T> {
    values: Vec<T>,
}

impl Display for CircularBuffer<isize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, val) in self.values.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            write!(f, "{:2}", val)?;
        }
        Ok(())
    }
}

impl<T> CircularBuffer<T> {
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<T> From<Vec<T>> for CircularBuffer<T> {
    fn from(values: Vec<T>) -> Self {
        CircularBuffer { values }
    }
}

impl<T> FromIterator<T> for CircularBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        CircularBuffer {
            values: iter.into_iter().collect(),
        }
    }
}

impl<T> Index<isize> for CircularBuffer<T> {
    type Output = T;
    fn index(&self, index: isize) -> &Self::Output {
        &self.values[modulo(index, self.values.len())]
    }
}

impl<T> IndexMut<isize> for CircularBuffer<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let len = self.values.len();
        &mut self.values[modulo(index, len)]
    }
}

#[derive(Debug, Clone)]
struct ModRange {
    start: usize,
    end: usize,
    len: usize,
}

impl ModRange {
    fn new(start: isize, end: isize, len: usize) -> Self {
        ModRange {
            start: modulo(start, len),
            end: modulo(end, len),
            len,
        }
    }
    fn contains(&self, value: isize) -> bool {
        let value = modulo(value, self.len);
        match self.end.cmp(&self.start) {
            Ordering::Greater => value >= self.start && value < self.end,
            Ordering::Less => value >= self.start || value < self.end,
            Ordering::Equal => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Permutation {
    indices: Vec<usize>,
}

impl From<Vec<usize>> for Permutation {
    fn from(indices: Vec<usize>) -> Self {
        Permutation { indices }
    }
}

impl FromIterator<usize> for Permutation {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Permutation {
            indices: iter.into_iter().collect(),
        }
    }
}

impl Permutation {
    fn new(len: usize) -> Self {
        Permutation {
            indices: (0..len).collect(),
        }
    }

    fn shift(&mut self, start_index: usize, diff: isize) {
        let len = self.indices.len();
        let current_index = self.indices[start_index] as isize;
        let wraps = diff / (len as isize - 1);

        let new_index = current_index + diff % (len as isize - 1);
        let (shift_range, shift_offset) = if new_index < current_index {
            (ModRange::new(new_index, current_index, len), 1)
        } else {
            (ModRange::new(current_index + 1, new_index + 1, len), -1)
        };

        for index in self.indices.iter_mut() {
            if *index as isize == current_index {
                *index = modulo(current_index + diff, len);
            } else {
                let shift = if shift_range.contains(*index as isize) {
                    shift_offset
                } else {
                    0
                };
                *index = modulo(*index as isize - wraps + shift, len);
            }
        }
    }

    #[allow(unused)]
    fn is_valid(&self) -> bool {
        let mut indices = self.indices.clone();
        indices.sort();
        indices == (0..indices.len()).collect::<Vec<_>>()
    }

    fn apply<T: Default + Clone>(&self, initial: &CircularBuffer<T>) -> CircularBuffer<T> {
        let mut end = repeat(T::default())
            .take(self.indices.len())
            .collect::<Vec<_>>();
        for (start_index, end_index) in self.indices.iter().enumerate() {
            end[*end_index] = initial[start_index as isize].clone()
        }
        end.into()
    }
}

fn mix(initial: &CircularBuffer<isize>, num_times: usize) -> CircularBuffer<isize> {
    let mut permutation = Permutation::new(initial.len());

    for _ in 0..num_times {
        for (start_index, value) in initial.iter().enumerate() {
            permutation.shift(start_index, *value);
        }
    }

    permutation.apply(initial)
}

fn get_grove_coordinates(
    start: &CircularBuffer<isize>,
    decryption_key: Option<isize>,
    num_times: usize,
) -> (isize, isize, isize) {
    let values = start
        .iter()
        .map(|val| val * decryption_key.unwrap_or(1))
        .collect();
    let end_values = mix(&values, num_times);
    let start_pos = end_values.iter().find_position(|x| **x == 0).unwrap().0 as isize;
    (
        end_values[start_pos + 1000],
        end_values[start_pos + 2000],
        end_values[start_pos + 3000],
    )
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = CircularBuffer<isize>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.lines()
            .map(|line| line.parse().map_err(|err: ParseIntError| err.into()))
            .collect::<Result<CircularBuffer<_>, _>>()
    }

    fn solve(values: Self::Problem) -> (Option<String>, Option<String>) {
        let (x, y, z) = get_grove_coordinates(&values, None, 1);
        let part_one = (x + y + z).to_string();
        let (x, y, z) = get_grove_coordinates(&values, Some(811589153), 10);
        let part_two = (x + y + z).to_string();
        (Some(part_one), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shift_1() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, 1);
        assert_eq!(permutation, vec![1, 0, 2].into());
    }

    #[test]
    fn test_shift_2() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, 2);
        assert_eq!(permutation, vec![2, 0, 1].into());
    }

    #[test]
    fn test_shift_3() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, 3);
        assert_eq!(permutation, vec![0, 2, 1].into());
    }

    #[test]
    fn test_shift_4() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, 4);
        assert_eq!(permutation, vec![1, 2, 0].into());
    }

    #[test]
    fn test_shift_minus_1() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, -1);
        assert_eq!(permutation, vec![2, 1, 0].into());
    }

    #[test]
    fn test_shift_minus_2() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, -2);
        assert_eq!(permutation, vec![1, 2, 0].into());
    }

    #[test]
    fn test_shift_minus_3() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, -3);
        assert_eq!(permutation, vec![0, 2, 1].into());
    }

    #[test]
    fn test_shift_minus_4() {
        let mut permutation = Permutation::new(3);
        permutation.shift(0, -4);
        assert_eq!(permutation, vec![2, 0, 1].into());
    }
}
