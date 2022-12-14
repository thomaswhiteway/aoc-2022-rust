use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::digit1,
    combinator::{map_res, opt, recognize},
    sequence::pair,
    IResult,
};
use std::str::FromStr;

pub fn unsigned<T: FromStr>(input: &str) -> IResult<&str, T> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |size: &str| {
        size.parse()
    })(input)
}

pub fn signed(input: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(tag("-")), digit1)), |val: &str| {
        val.parse()
    })(input)
}
