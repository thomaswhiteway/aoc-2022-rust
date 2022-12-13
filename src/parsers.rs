use nom::{bytes::complete::take_while1, combinator::map_res, IResult};
use std::str::FromStr;

pub fn unsigned<T: FromStr>(input: &str) -> IResult<&str, T> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |size: &str| {
        size.parse()
    })(input)
}
