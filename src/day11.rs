use failure::{err_msg, Error};

mod parse {
    use std::str::FromStr;

    use super::{Monkey, Operation, Value};
    use failure::{err_msg, Error};
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_while1},
        combinator::{all_consuming, map, map_res, value},
        multi::separated_list1,
        sequence::{delimited, preceded, tuple},
        IResult,
    };

    fn unsigned<T: FromStr>(input: &str) -> IResult<&str, T> {
        map_res(take_while1(|c: char| c.is_ascii_digit()), |size: &str| {
            size.parse()
        })(input)
    }

    fn items(input: &str) -> IResult<&str, Vec<u64>> {
        separated_list1(tag(", "), unsigned)(input)
    }

    fn op_value(input: &str) -> IResult<&str, Value> {
        alt((value(Value::Old, tag("old")), map(unsigned, Value::Literal)))(input)
    }

    fn operator(input: &str) -> IResult<&str, impl Fn(Value, Value) -> Operation> {
        map(alt((tag("+"), tag("*"))), |op: &str| {
            let op = op.to_string();
            move |x, y| match op.as_str() {
                "+" => Operation::Add(x, y),
                "*" => Operation::Multiply(x, y),
                _ => panic!("Unexpected operator: {}", op),
            }
        })(input)
    }

    fn test_divisible(input: &str) -> IResult<&str, u64> {
        preceded(tag("divisible by "), unsigned)(input)
    }

    fn throw(input: &str) -> IResult<&str, usize> {
        preceded(tag("throw to monkey "), unsigned)(input)
    }

    fn operation(input: &str) -> IResult<&str, Operation> {
        map(
            preceded(
                tag("new = "),
                tuple((op_value, delimited(tag(" "), operator, tag(" ")), op_value)),
            ),
            |(x, op, y)| op(x, y),
        )(input)
    }

    fn monkey(input: &str) -> IResult<&str, Monkey> {
        map(
            tuple((
                delimited(tag("Monkey "), unsigned, tag(":\n")),
                delimited(tag("  Starting items: "), items, tag("\n")),
                delimited(tag("  Operation: "), operation, tag("\n")),
                delimited(tag("  Test: "), test_divisible, tag("\n")),
                delimited(tag("    If true: "), throw, tag("\n")),
                delimited(tag("    If false: "), throw, tag("\n")),
            )),
            |(index, items, operation, test_divisible, test_pass_throw, test_fail_throw)| Monkey {
                index,
                items,
                operation,
                test_divisible,
                test_pass_throw,
                test_fail_throw,
                inspections: 0,
            },
        )(input)
    }

    fn monkeys(input: &str) -> IResult<&str, Box<[Monkey]>> {
        map(separated_list1(tag("\n"), monkey), Vec::into_boxed_slice)(input)
    }

    pub fn parse_input(input: &str) -> Result<Box<[Monkey]>, Error> {
        all_consuming(monkeys)(input)
            .map(|(_, ms)| ms)
            .map_err(|err| err_msg(format!("Failed to parse monkeys: {}", err)))
    }
}

use parse::parse_input;

#[derive(Debug, Clone)]
pub struct Monkey {
    index: usize,
    items: Vec<u64>,
    operation: Operation,
    test_divisible: u64,
    test_pass_throw: usize,
    test_fail_throw: usize,
    inspections: usize,
}

impl Monkey {
    fn take_turn(&mut self, reduce_worry: bool, modulo: u64) -> Vec<Throw> {
        self.inspections += self.items.len();
        self.items
            .drain(..)
            .map(|mut worry_level| {
                worry_level = self.operation.apply(worry_level);

                if reduce_worry {
                    worry_level /= 3;
                }

                worry_level %= modulo;

                let monkey = if worry_level % self.test_divisible == 0 {
                    self.test_pass_throw
                } else {
                    self.test_fail_throw
                };

                Throw {
                    monkey,
                    item: worry_level,
                }
            })
            .collect()
    }

    fn catch(&mut self, item: u64) {
        self.items.push(item);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    Old,
    Literal(u64),
}

impl Value {
    fn value(&self, old: u64) -> u64 {
        match self {
            Value::Old => old,
            Value::Literal(val) => *val,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Add(Value, Value),
    Multiply(Value, Value),
}

impl Operation {
    fn apply(&self, old: u64) -> u64 {
        match self {
            Operation::Add(x, y) => x.value(old) + y.value(old),
            Operation::Multiply(x, y) => x.value(old) * y.value(old),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Throw {
    monkey: usize,
    item: u64,
}

struct Executor {
    monkeys: Box<[Monkey]>,
    reduce_worry: bool,
    modulo: u64,
}

impl Executor {
    fn new(monkeys: Box<[Monkey]>, reduce_worry: bool) -> Self {
        let modulo = monkeys.iter().map(|monkey| monkey.test_divisible).product();
        Executor {
            monkeys,
            reduce_worry,
            modulo,
        }
    }

    fn execute_round(&mut self) {
        for index in 0..self.monkeys.len() {
            for throw in self.monkeys[index].take_turn(self.reduce_worry, self.modulo) {
                self.monkeys[throw.monkey].catch(throw.item);
            }
        }
    }

    fn execute(&mut self, rounds: usize) {
        for _ in 0..rounds {
            self.execute_round()
        }
    }

    fn count_inspections(&self) -> Box<[usize]> {
        self.monkeys
            .iter()
            .map(|monkey| monkey.inspections)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn get_monkey_business(&self) -> usize {
        let mut num_inspections = self.count_inspections();
        num_inspections.sort_unstable_by(|a, b| a.cmp(b).reverse());
        num_inspections[0] * num_inspections[1]
    }
}

fn get_monkey_business(monkeys: Box<[Monkey]>, reduce_worry: bool, rounds: usize) -> usize {
    let mut executor = Executor::new(monkeys, reduce_worry);
    executor.execute(rounds);
    executor.get_monkey_business()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Monkey]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        let mut monkeys = parse_input(&data)?;
        monkeys.sort_by_key(|monkey| monkey.index);

        for (index, monkey) in monkeys.iter().enumerate() {
            if monkey.index != index {
                return Err(err_msg(format!("Missing monkey {}", index)));
            }
        }

        Ok(monkeys)
    }

    fn solve(monkeys: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = get_monkey_business(monkeys.clone(), true, 20).to_string();
        let part_two = get_monkey_business(monkeys, false, 10000).to_string();
        (Some(part_one), Some(part_two))
    }
}
