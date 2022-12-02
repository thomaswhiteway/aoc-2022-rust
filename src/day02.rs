use failure::{err_msg, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    multi::many1,
    sequence::{separated_pair, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpponentKey {
    A,
    B,
    C,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PlayerKey {
    X,
    Y,
    Z,
}

pub struct Rule {
    opponent: OpponentKey,
    player: PlayerKey,
}

fn opponent_key(input: &str) -> IResult<&str, OpponentKey> {
    alt((
        value(OpponentKey::A, tag("A")),
        value(OpponentKey::B, tag("B")),
        value(OpponentKey::C, tag("C")),
    ))(input)
}

fn player_key(input: &str) -> IResult<&str, PlayerKey> {
    alt((
        value(PlayerKey::X, tag("X")),
        value(PlayerKey::Y, tag("Y")),
        value(PlayerKey::Z, tag("Z")),
    ))(input)
}

fn rule(input: &str) -> IResult<&str, Rule> {
    map(
        separated_pair(opponent_key, tag(" "), player_key),
        |(opponent, player)| Rule { opponent, player },
    )(input)
}

fn rules(input: &str) -> IResult<&str, Box<[Rule]>> {
    map(many1(terminated(rule, tag("\n"))), |rules| {
        rules.into_boxed_slice()
    })(input)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    fn score(self) -> u64 {
        use Hand::*;
        match self {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        }
    }
}

impl From<PlayerKey> for Hand {
    fn from(player: PlayerKey) -> Self {
        match player {
            PlayerKey::X => Hand::Rock,
            PlayerKey::Y => Hand::Paper,
            PlayerKey::Z => Hand::Scissors,
        }
    }
}

impl From<OpponentKey> for Hand {
    fn from(player: OpponentKey) -> Self {
        match player {
            OpponentKey::A => Hand::Rock,
            OpponentKey::B => Hand::Paper,
            OpponentKey::C => Hand::Scissors,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Outcome {
    Win,
    Draw,
    Lose,
}

impl Outcome {
    fn score(self) -> u64 {
        use Outcome::*;
        match self {
            Win => 6,
            Draw => 3,
            Lose => 0,
        }
    }
}

impl From<PlayerKey> for Outcome {
    fn from(player: PlayerKey) -> Self {
        match player {
            PlayerKey::X => Outcome::Lose,
            PlayerKey::Y => Outcome::Draw,
            PlayerKey::Z => Outcome::Win,
        }
    }
}

fn play_game(player: Hand, opponent: Hand) -> Outcome {
    use Hand::*;
    use Outcome::*;
    match (player, opponent) {
        (Rock, Rock) => Draw,
        (Rock, Paper) => Lose,
        (Rock, Scissors) => Win,
        (Paper, Rock) => Win,
        (Paper, Paper) => Draw,
        (Paper, Scissors) => Lose,
        (Scissors, Rock) => Lose,
        (Scissors, Paper) => Win,
        (Scissors, Scissors) => Draw,
    }
}

fn pick_hand(opponent: Hand, outcome: Outcome) -> Hand {
    use Hand::*;
    use Outcome::*;
    match (opponent, outcome) {
        (Rock, Lose) => Scissors,
        (Rock, Draw) => Rock,
        (Rock, Win) => Paper,
        (Paper, Lose) => Rock,
        (Paper, Draw) => Paper,
        (Paper, Win) => Scissors,
        (Scissors, Lose) => Paper,
        (Scissors, Draw) => Scissors,
        (Scissors, Win) => Rock,
    }
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Rule]>;

    fn parse_input(data: &str) -> Result<Self::Problem, Error> {
        rules(data)
            .map(|(_, rules)| rules)
            .map_err(|err| err_msg(format!("Failed to parse rules: {}", err)))
    }

    fn solve(problem: &Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = problem
            .iter()
            .map(|rule| {
                let player: Hand = rule.player.into();
                let opponent: Hand = rule.opponent.into();
                let outcome: Outcome = play_game(player, opponent);
                player.score() + outcome.score()
            })
            .sum::<u64>()
            .to_string();

        let part_two = problem
            .iter()
            .map(|rule| {
                let opponent: Hand = rule.opponent.into();
                let outcome = rule.player.into();
                let player = pick_hand(opponent, outcome);
                player.score() + outcome.score()
            })
            .sum::<u64>()
            .to_string();

        (Some(part_one), Some(part_two))
    }
}
