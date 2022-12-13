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

    fn pairs(input: &str) -> IResult<&str, Vec<(Packet, Packet)>> {
        separated_list1(newline, pair)(input)
    }

    pub fn parse_input(input: &str) -> Result<Vec<(Packet, Packet)>, Error> {
        all_consuming(pairs)(input)
            .map(|(_, pairs)| pairs)
            .map_err(|err| err_msg(format!("Failed to parse packets: {}", err)))
    }
}
use failure::{err_msg, Error};

use itertools::Itertools;
use parse::parse_input;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Packet {
    List(Box<[Packet]>),
    Number(u64),
}

impl Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Packet::*;
        match self {
            Number(x) => write!(f, "{}", x),
            List(xs) => {
                write!(f, "[")?;
                for (index, x) in xs.iter().enumerate() {
                    if index > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "]")
            }
        }
    }
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

fn build_divider(num: u64) -> Packet {
    Packet::List(
        vec![Packet::List(vec![Packet::Number(num)].into_boxed_slice())].into_boxed_slice(),
    )
}

fn find_packet(packets: &[Packet], packet: Packet) -> Option<usize> {
    packets
        .iter()
        .find_position(|&p| *p == packet)
        .map(|(index, _)| index + 1)
}

fn get_decoder_key(pairs: Vec<(Packet, Packet)>) -> Result<usize, Error> {
    let divider_one = build_divider(2);
    let divider_two = build_divider(6);

    let mut all_packets: Vec<Packet> = pairs.into_iter().flat_map(|(x, y)| [x, y]).collect();
    all_packets.extend([divider_one.clone(), divider_two.clone()]);
    all_packets.sort();

    let position_one = find_packet(&all_packets, divider_one)
        .ok_or_else(|| err_msg("Failed to find first divider"))?;
    let position_two = find_packet(&all_packets, divider_two)
        .ok_or_else(|| err_msg("Failed to find second divider"))?;

    Ok(position_one * position_two)
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Vec<(Packet, Packet)>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(pairs: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = indices_of_ordered_pairs(&pairs).sum::<usize>().to_string();
        let part_two = get_decoder_key(pairs)
            .expect("Failed to solve part two")
            .to_string();
        (Some(part_one), Some(part_two))
    }
}
