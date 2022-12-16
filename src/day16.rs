use crate::parsers::unsigned;
use failure::{err_msg, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline},
    combinator::{all_consuming, map},
    multi::{many1, separated_list1},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use std::collections::{HashMap, HashSet};

fn parse_input(input: &str) -> Result<Vec<Valve>, Error> {
    fn valve_name(input: &str) -> IResult<&str, String> {
        map(alpha1, |val: &str| val.to_string())(input)
    }

    let tunnels = separated_list1(tag(", "), valve_name);
    let valve = map(
        tuple((
            preceded(tag("Valve "), valve_name),
            preceded(tag(" has flow rate="), unsigned),
            preceded(
                alt((
                    tag("; tunnel leads to valve "),
                    tag("; tunnels lead to valves "),
                )),
                tunnels,
            ),
        )),
        |(name, flow_rate, tunnels)| Valve {
            name: name.to_string(),
            flow_rate,
            tunnels: tunnels.into_boxed_slice(),
        },
    );

    let valves = many1(terminated(valve, newline));

    all_consuming(valves)(input)
        .map(|(_, valves)| valves)
        .map_err(|err| err_msg(format!("Failed to parse valves: {}", err)))
}

struct State {
    location: String,
    time_left: u64,
    valves_opened: HashSet<String>,
    pressure_released: u64,
}

impl State {
    fn successors(
        &self,
        valves: &HashMap<String, Valve>,
        distances: &HashMap<String, HashMap<String, u64>>,
    ) -> Vec<State> {
        valves
            .values()
            .filter(|valve| !self.valves_opened.contains(&valve.name))
            .filter_map(|next| {
                let distance = distances
                    .get(&self.location)
                    .unwrap()
                    .get(&next.name)
                    .unwrap();
                if self.time_left > distance + 1 && next.flow_rate > 0 {
                    let time_left = self.time_left - distance - 1;
                    let mut valves_opened = self.valves_opened.clone();
                    valves_opened.insert(next.name.clone());
                    Some(State {
                        location: next.name.to_string(),
                        time_left,
                        valves_opened,
                        pressure_released: self.pressure_released + time_left * next.flow_rate,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct Valve {
    name: String,
    flow_rate: u64,
    tunnels: Box<[String]>,
}

fn calculate_distances(valves: &HashMap<String, Valve>) -> HashMap<String, HashMap<String, u64>> {
    let mut distances = HashMap::new();
    for valve in valves.values() {
        let mut distance = 1;
        let mut next: Vec<_> = valve.tunnels.iter().collect();
        let mut ds: HashMap<String, u64> = Default::default();

        while !next.is_empty() {
            ds.extend(next.iter().map(|name| ((*name).clone(), distance)));
            distance += 1;

            next = next
                .iter()
                .flat_map(|name| valves.get(*name).unwrap().tunnels.iter())
                .filter(|name| !ds.contains_key(*name))
                .collect();
        }

        distances.insert(valve.name.clone(), ds);
    }

    distances
}

fn max_remaining_pressure(state: &State, valves: &[&Valve]) -> u64 {
    if state.time_left <= 2 {
        return 0;
    }
    let rem_valves = valves
        .iter()
        .filter(|valve| !state.valves_opened.contains(&valve.name));
    (0..=state.time_left - 2)
        .rev()
        .step_by(2)
        .zip(rem_valves)
        .map(|(time, valve)| time * valve.flow_rate)
        .sum::<u64>()
        + if !state.valves_opened.contains(&state.location) {
            valves
                .iter()
                .find(|valve| valve.name == state.location)
                .unwrap()
                .flow_rate
                * state.time_left
        } else {
            0
        }
}

fn find_most_pressure(valves: &HashMap<String, Valve>) -> u64 {
    let mut stack = vec![State {
        location: "AA".to_string(),
        time_left: 30,
        valves_opened: Default::default(),
        pressure_released: 0,
    }];

    let mut sorted_valves: Vec<_> = valves.values().collect();
    sorted_valves.sort_by(|a, b| a.flow_rate.cmp(&b.flow_rate).reverse());

    let distances = calculate_distances(valves);

    let mut best = 0;
    while let Some(state) = stack.pop() {
        if state.pressure_released + max_remaining_pressure(&state, &sorted_valves) <= best {
            continue;
        }

        if state.time_left <= 2 {
            if state.pressure_released > best {
                best = state.pressure_released;
            }
            continue;
        }

        let mut successors = state.successors(valves, &distances);
        successors.sort_by_key(|state| (state.pressure_released, state.time_left));

        stack.extend(successors);
    }

    best
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = HashMap<String, Valve>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data).map(|valves| {
            valves
                .into_iter()
                .map(|valve| (valve.name.clone(), valve))
                .collect()
        })
    }

    fn solve(valves: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = find_most_pressure(&valves).to_string();
        (Some(part_one), None)
    }
}
