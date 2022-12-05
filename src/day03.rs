use failure::Error;
use itertools::Itertools;
use std::{collections::HashSet, hash::Hash};

fn find_duplicate(contents: &[char]) -> Option<char> {
    let first_compartment = contents[..contents.len() / 2]
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let second_compartment = contents[contents.len() / 2..]
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    first_compartment
        .intersection(&second_compartment)
        .next()
        .cloned()
}

fn score(item: char) -> u64 {
    match item {
        'a'..='z' => 1 + (item as u64 - 'a' as u64),
        'A'..='Z' => 27 + (item as u64 - 'A' as u64),
        _ => panic!("Unknown item {}", item),
    }
}

fn intersect_3<T: Eq + Hash + Clone>(a: HashSet<T>, b: HashSet<T>, c: HashSet<T>) -> HashSet<T> {
    [a, b, c]
        .into_iter()
        .reduce(|x, y| x.intersection(&y).cloned().collect::<HashSet<_>>())
        .unwrap()
}

fn pick_one<T>(set: HashSet<T>) -> Option<T> {
    set.into_iter().next()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Box<[char]>]>;

    fn parse_input(data: &str) -> Result<Self::Problem, Error> {
        Ok(data
            .lines()
            .map(|line| line.trim().chars().collect::<Vec<_>>().into_boxed_slice())
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    fn solve(problem: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = problem
            .iter()
            .map(|contents| find_duplicate(contents).unwrap())
            .map(score)
            .sum::<u64>()
            .to_string();

        let part_two = problem
            .iter()
            .map(|contents| contents.iter().cloned().collect::<HashSet<_>>())
            .tuples()
            .map(|(a, b, c)| intersect_3(a, b, c))
            .map(|set| pick_one(set).unwrap())
            .map(score)
            .sum::<u64>()
            .to_string();
        (Some(part_one), Some(part_two))
    }
}
