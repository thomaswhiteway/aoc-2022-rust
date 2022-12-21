use std::{collections::HashMap, fmt::Display};

use failure::{err_msg, Error};

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
            |(left, _, op, _, right)| Operation {
                op,
                left: Box::new(Expression::Variable(left)),
                right: Box::new(Expression::Variable(right)),
            },
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
    Equals,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::Equals => write!(f, "="),
        }
    }
}

impl Operator {
    fn apply(self, left: i64, right: i64) -> i64 {
        match self {
            Operator::Add => left + right,
            Operator::Sub => left - right,
            Operator::Multiply => left * right,
            Operator::Divide => left / right,
            Operator::Equals => i64::from(left == right),
        }
    }

    fn inverse(self) -> Self {
        match self {
            Operator::Add => Operator::Sub,
            Operator::Sub => Operator::Add,
            Operator::Multiply => Operator::Divide,
            Operator::Divide => Operator::Multiply,
            Operator::Equals => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    op: Operator,
    left: Box<Expression>,
    right: Box<Expression>,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl Operation {
    fn expand(&self, expressions: &HashMap<Monkey, Expression>) -> Operation {
        let left = Box::new(self.left.expand(expressions));
        let right = Box::new(self.right.expand(expressions));
        Operation {
            op: self.op,
            left,
            right,
        }
    }

    fn reduce(&self) -> Expression {
        let left = self.left.reduce();
        let right = self.right.reduce();

        if let (Some(left), Some(right)) = (left.value(), right.value()) {
            Expression::Value(self.op.apply(left, right))
        } else {
            Expression::Operation(Operation {
                op: self.op,
                left: Box::new(left),
                right: Box::new(right),
            })
        }
    }

    fn normalize(&self) -> Expression {
        let mut op = self.op;
        let mut left = Box::new(self.left.normalize());
        let mut right = Box::new(self.right.normalize());

        match self.op {
            Operator::Equals => {
                while left
                    .operation()
                    .map(|operation| operation.right.is_value())
                    .unwrap_or(false)
                {
                    let left_op = left.operation().unwrap().clone();
                    left = left_op.left.clone();
                    right = Box::new(
                        Expression::Operation(Operation {
                            op: left_op.op.inverse(),
                            left: right,
                            right: left_op.right,
                        })
                        .reduce(),
                    );
                }
            }
            Operator::Add => {
                if self.left.is_value() {
                    std::mem::swap(&mut left, &mut right);
                }
            }
            Operator::Sub => {
                op = Operator::Add;
                right = Box::new(
                    Expression::Operation(Operation {
                        op: Operator::Multiply,
                        left: right,
                        right: Box::new(Expression::Value(-1)),
                    })
                    .reduce()
                    .normalize(),
                );
            }
            Operator::Multiply => {
                if self.right.is_value() && self.left.is_operation() {
                    let left_op = self.left.operation().unwrap();
                    match left_op.op {
                        Operator::Add | Operator::Sub => {
                            op = left_op.op;
                            left = Box::new(
                                Operation {
                                    op: Operator::Multiply,
                                    left: left_op.left.clone(),
                                    right: right.clone(),
                                }
                                .reduce()
                                .normalize(),
                            );
                            right = Box::new(
                                Operation {
                                    op: Operator::Multiply,
                                    left: left_op.right.clone(),
                                    right: right.clone(),
                                }
                                .reduce()
                                .normalize(),
                            );
                        }
                        _ => {}
                    }
                } else if self.left.is_value() {
                    std::mem::swap(&mut left, &mut right);
                }
            }
            _ => {}
        }

        let expression = Expression::Operation(Operation { op, left, right });

        if op != self.op {
            expression.normalize()
        } else {
            expression
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Value(i64),
    Operation(Operation),
    Variable(Monkey),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(x) => write!(f, "{}", x),
            Expression::Variable(monkey) => write!(f, "{}", monkey),
            Expression::Operation(operation) => write!(f, "{}", operation),
        }
    }
}

impl Expression {
    fn expand(&self, expressions: &HashMap<Monkey, Expression>) -> Self {
        match self {
            Expression::Value(_) => self.clone(),
            Expression::Operation(operation) => {
                Expression::Operation(operation.expand(expressions))
            }
            Expression::Variable(monkey) => {
                if let Some(expression) = expressions.get(monkey) {
                    expression.expand(expressions)
                } else {
                    self.clone()
                }
            }
        }
    }

    fn reduce(&self) -> Self {
        if let Expression::Operation(operation) = self {
            operation.reduce()
        } else {
            self.clone()
        }
    }

    fn normalize(&self) -> Self {
        match self {
            Expression::Operation(operation) => operation.normalize(),
            _ => self.clone(),
        }
    }

    fn is_value(&self) -> bool {
        matches!(self, Expression::Value(_))
    }

    fn value(&self) -> Option<i64> {
        if let Expression::Value(x) = self {
            Some(*x)
        } else {
            None
        }
    }

    fn is_operation(&self) -> bool {
        matches!(self, Expression::Operation(_))
    }

    fn operation(&self) -> Option<&Operation> {
        if let Expression::Operation(operation) = self {
            Some(operation)
        } else {
            None
        }
    }

    fn operation_mut(&mut self) -> Option<&mut Operation> {
        if let Expression::Operation(operation) = self {
            Some(operation)
        } else {
            None
        }
    }
}

fn what_does_the_monkey_shout(instructions: &[Instruction], target: Monkey) -> Result<i64, Error> {
    let instructions = instructions.iter().cloned().collect::<HashMap<_, _>>();
    let outcome = instructions
        .get(&target)
        .ok_or_else(|| err_msg("Failed to find target"))?
        .expand(&instructions)
        .reduce();

    if let Some(x) = outcome.value() {
        Ok(x)
    } else {
        Err(err_msg(format!("{} is not fully reduced", outcome)))
    }
}

fn what_should_i_shout(
    instructions: &[Instruction],
    target: Monkey,
    me: Monkey,
) -> Result<i64, Error> {
    let mut instructions = instructions.iter().cloned().collect::<HashMap<_, _>>();
    instructions.remove(&me);
    instructions
        .get_mut(&target)
        .ok_or_else(|| err_msg("Failed to find target"))?
        .operation_mut()
        .ok_or_else(|| err_msg("Target does not have an operation"))?
        .op = Operator::Equals;

    let reduced = instructions
        .get(&target)
        .ok_or_else(|| err_msg("Failed to find target"))?
        .expand(&instructions)
        .reduce();
    let normalized = reduced.normalize();

    let operation = normalized
        .operation()
        .ok_or_else(|| err_msg(format!("Not and operation: {}", normalized)))?;

    if operation.op != Operator::Equals {
        return Err(err_msg(format!("Not an equality: {}", operation)));
    }

    if *operation.left != Expression::Variable(me) {
        return Err(err_msg(format!(
            "Failed to normalize expression: {}",
            operation
        )));
    }

    operation
        .right
        .value()
        .ok_or_else(|| err_msg(format!("Failed to normalize expression: {}", operation)))
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
        let part_two = what_should_i_shout(&instructions, "root".to_string(), "humn".to_string())
            .expect("Failed to solve part two")
            .to_string();
        (Some(part_one), Some(part_two))
    }
}
