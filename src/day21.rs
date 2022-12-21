use std::collections::HashMap;

use failure::Error;

use self::parse::parse_input;

mod parse {
    use super::{Expression, Instruction, Monkey, Operation, Operator};
    use crate::parsers::signed;
    use failure::{err_msg, Error};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, newline},
        combinator::{all_consuming, map, value},
        multi::many1,
        sequence::{separated_pair, terminated, tuple},
        IResult,
    };

    fn operator(input: &str) -> IResult<&str, Operator> {
        alt((
            value(Operator::Add, tag("+")),
            value(Operator::Multiply, tag("*")),
            value(Operator::Sub, tag("-")),
            value(Operator::Divide, tag("/")),
        ))(input)
    }

    fn monkey(input: &str) -> IResult<&str, Monkey> {
        map(alpha1, |val: &str| val.to_string())(input)
    }

    fn operation(input: &str) -> IResult<&str, Operation> {
        map(
            tuple((monkey, tag(" "), operator, tag(" "), monkey)),
            |(left, _, op, _, right)| Operation { op, left, right },
        )(input)
    }

    fn expression(input: &str) -> IResult<&str, Expression> {
        alt((
            map(signed, Expression::Value),
            map(operation, Expression::Operation),
        ))(input)
    }

    fn instruction(input: &str) -> IResult<&str, Instruction> {
        separated_pair(monkey, tag(": "), expression)(input)
    }

    fn instructions(input: &str) -> IResult<&str, Box<[Instruction]>> {
        map(
            many1(terminated(instruction, newline)),
            Vec::into_boxed_slice,
        )(input)
    }

    pub(super) fn parse_input(input: &str) -> Result<Box<[Instruction]>, Error> {
        all_consuming(instructions)(input)
            .map(|(_, instructions)| instructions)
            .map_err(|err| err_msg(format!("Failed to parse instructions: {}", err)))
    }
}

type Monkey = String;
type Instruction = (Monkey, Expression);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Multiply,
    Divide,
}

impl Operator {
    fn apply(self, left: i64, right: i64) -> i64 {
        match self {
            Operator::Add => left + right,
            Operator::Sub => left - right,
            Operator::Multiply => left * right,
            Operator::Divide => left / right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    op: Operator,
    left: Monkey,
    right: Monkey,
}

impl Operation {
    fn resolve(&self, values: &HashMap<Monkey, i64>) -> Option<i64> {
        let left = *values.get(self.left.as_str())?;
        let right = *values.get(self.right.as_str())?;
        Some(self.op.apply(left, right))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Value(i64),
    Operation(Operation),
}

impl Expression {
    fn resolve(&self, values: &HashMap<Monkey, i64>) -> Option<i64> {
        match self {
            Expression::Value(value) => Some(*value),
            Expression::Operation(operation) => operation.resolve(values),
        }
    }
}

fn what_does_the_monkey_shout(instructions: &[Instruction], target: Monkey) -> Option<i64> {
    let mut values = HashMap::new();
    let mut remaining = instructions.to_vec();

    while !values.contains_key(&target) && !remaining.is_empty() {
        remaining.retain(|(monkey, operation)| {
            if let Some(value) = operation.resolve(&values) {
                values.insert(monkey.clone(), value);
                false
            } else {
                true
            }
        });
    }

    values.get(&target).cloned()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Instruction]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(instructions: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = what_does_the_monkey_shout(&instructions, "root".to_string())
            .expect("Failed to solve part one")
            .to_string();
        (Some(part_one), None)
    }
}
