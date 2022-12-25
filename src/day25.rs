use failure::{err_msg, Error};
use std::{
    fmt::Display,
    iter::{self, Sum},
    ops::AddAssign,
    str::FromStr,
};

fn from_snafu_digit(c: char) -> Result<i64, Error> {
    match c {
        '0'..='2' => Ok(c.to_digit(5).unwrap() as i64),
        '-' => Ok(-1),
        '=' => Ok(-2),
        _ => Err(err_msg(format!("Invalid digit {}", c))),
    }
}

fn to_snafu_digit(val: i64) -> Result<char, Error> {
    match val {
        0..=2 => Ok(char::from_digit(val as u32, 5).unwrap()),
        -1 => Ok('-'),
        -2 => Ok('='),
        _ => Err(err_msg(format!("Invalid digit {}", val))),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Snafu(i64);

impl From<i64> for Snafu {
    fn from(val: i64) -> Self {
        Snafu(val)
    }
}

impl FromStr for Snafu {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .rev()
            .zip(0..)
            .map(|(c, pow)| from_snafu_digit(c).map(|d| d * 5_i64.pow(pow)))
            .collect::<Result<Vec<_>, _>>()
            .map(|ds| ds.iter().sum::<i64>().into())
    }
}

impl Display for Snafu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = self.0;
        let digits = iter::from_fn(|| {
            if value == 0 {
                None
            } else {
                let mut d = value % 5;
                if d > 2 {
                    d -= 5;
                }
                value -= d;
                value /= 5;
                Some(to_snafu_digit(d).unwrap())
            }
        })
        .collect::<Vec<_>>();
        let string: String = digits.iter().rev().collect();
        write!(f, "{}", string)
    }
}

impl AddAssign<Snafu> for Snafu {
    fn add_assign(&mut self, rhs: Snafu) {
        self.0 += rhs.0;
    }
}

impl<'a> Sum<&'a Snafu> for Snafu {
    fn sum<I: Iterator<Item = &'a Snafu>>(iter: I) -> Self {
        let mut total = 0.into();
        for num in iter {
            total += *num
        }
        total
    }
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Snafu]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        data.lines()
            .map(|line| line.parse())
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn solve(fuel: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = fuel.iter().sum::<Snafu>().to_string();
        (Some(part_one), None)
    }
}

#[cfg(test)]
mod test {
    use super::Snafu;

    #[test]
    fn test_parse() {
        assert_eq!("1=-0-2".parse::<Snafu>().unwrap(), Snafu(1747))
    }
}
