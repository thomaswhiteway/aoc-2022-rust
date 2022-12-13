use failure::Error;

mod parse {
    use super::Packet;
    use crate::parsers::unsigned;
    use failure::{err_msg, Error};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::newline,
        combinator::{all_consuming, map},
        multi::{separated_list0, separated_list1},
        sequence::{delimited, terminated, tuple},
        IResult,
    };

    fn number(input: &str) -> IResult<&str, Packet> {
        map(unsigned, Packet::Number)(input)
    }

    fn list(input: &str) -> IResult<&str, Packet> {
        map(
            map(
                delimited(tag("["), separated_list0(tag(","), packet), tag("]")),
                Vec::into_boxed_slice,
            ),
            Packet::List,
        )(input)
    }

    fn packet(input: &str) -> IResult<&str, Packet> {
        alt((number, list))(input)
    }

    fn pair(input: &str) -> IResult<&str, (Packet, Packet)> {
        tuple((terminated(packet, newline), terminated(packet, newline)))(input)
    }

    fn pairs(input: &str) -> IResult<&str, Box<[(Packet, Packet)]>> {
        map(separated_list1(newline, pair), Vec::into_boxed_slice)(input)
    }

    pub fn parse_input(input: &str) -> Result<Box<[(Packet, Packet)]>, Error> {
        all_consuming(pairs)(input)
            .map(|(_, pairs)| pairs)
            .map_err(|err| err_msg(format!("Failed to parse packets: {}", err)))
    }
}

use parse::parse_input;
use std::cmp::Ordering;

#[derive(Eq, PartialEq, Debug)]
pub enum Packet {
    List(Box<[Packet]>),
    Number(u64),
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Packet::*;
        Some(match (self, other) {
            (Number(x), Number(y)) => x.cmp(y),
            (List(x), List(y)) => x
                .iter()
                .zip(y.iter())
                .fold(Ordering::Equal, |ord, (a, b)| ord.then_with(|| a.cmp(b)))
                .then_with(|| x.len().cmp(&y.len())),
            (List(_), Number(y)) => self.cmp(&List(Box::new([Number(*y)]))),
            (Number(x), List(_)) => List(Box::new([Number(*x)])).cmp(other),
        })
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn indices_of_ordered_pairs(pairs: &[(Packet, Packet)]) -> impl Iterator<Item = usize> + '_ {
    (1..)
        .zip(pairs.iter())
        .filter_map(|(index, (x, y))| if x < y { Some(index) } else { None })
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[(Packet, Packet)]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(pairs: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = indices_of_ordered_pairs(&pairs).sum::<usize>().to_string();
        (Some(part_one), None)
    }
}
