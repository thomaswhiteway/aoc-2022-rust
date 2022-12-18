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
use std::{array, cell::Cell, collections::HashMap};

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
            name,
            flow_rate,
            tunnels: tunnels.into_boxed_slice(),
        },
    );

    let valves = many1(terminated(valve, newline));

    all_consuming(valves)(input)
        .map(|(_, valves)| valves)
        .map_err(|err| err_msg(format!("Failed to parse valves: {}", err)))
}

#[derive(Clone, Debug)]
enum Location<'a> {
    At(&'a Valve),
    EnRoute(&'a Valve, u64),
}

impl<'a> Location<'a> {
    fn valve(&self) -> &'a Valve {
        match self {
            Location::At(valve) => valve,
            Self::EnRoute(valve, _) => valve,
        }
    }

    fn time_remaining(&self) -> u64 {
        match self {
            Location::At(_) => 0,
            Location::EnRoute(_, t) => *t,
        }
    }
}

struct Distances(HashMap<String, HashMap<String, u64>>);

impl Distances {
    fn distance_between(&self, from: &Valve, to: &Valve) -> u64 {
        *self.0.get(&from.name).unwrap().get(&to.name).unwrap()
    }

    fn min_distance(&self) -> u64 {
        *self
            .0
            .iter()
            .map(|(_, ds)| ds.values().filter(|dist| **dist > 0).min().unwrap())
            .min()
            .unwrap()
    }
}

#[derive(Debug)]
struct State<'a, const N: usize> {
    locations: [Location<'a>; N],
    time_left: u64,
    valves_remaining: Vec<&'a Valve>,
    pressure_released: u64,
    max_pressure: Cell<Option<u64>>,
}

fn all_location_combos<'a>(locations: &[Vec<Location<'a>>]) -> Vec<Vec<Location<'a>>> {
    if locations.len() == 1 {
        locations[0].iter().map(|loc| vec![loc.clone()]).collect()
    } else {
        locations[0]
            .iter()
            .flat_map(|loc| {
                all_location_combos(&locations[1..])
                    .into_iter()
                    .filter_map(|mut locs| {
                        if locs
                            .iter()
                            .all(|loc2| loc2.valve().name != loc.valve().name)
                        {
                            locs.insert(0, loc.clone());
                            Some(locs)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}

impl<'a, const N: usize> State<'a, N> {
    fn successors<'b>(
        &'b self,
        valves: &'b HashMap<String, Valve>,
        distances: &'b Distances,
    ) -> impl Iterator<Item = State<'a, N>> + 'b {
        let next_locations_per_actor = self
            .locations
            .iter()
            .map(|location| {
                if let Location::At(loc) = location {
                    self.valves_remaining
                        .iter()
                        .map(|valve| {
                            Location::EnRoute(valve, distances.distance_between(loc, valve) + 1)
                        })
                        .collect()
                } else {
                    vec![location.clone()]
                }
            })
            .collect::<Vec<_>>();

        all_location_combos(&next_locations_per_actor)
            .into_iter()
            .filter_map(|next_locs| {
                let time_needed = next_locs
                    .iter()
                    .map(|loc| loc.time_remaining())
                    .min()
                    .unwrap();
                if self.time_left > time_needed {
                    let time_left = self.time_left - time_needed;
                    let locations = array::from_fn(|i| match next_locs[i] {
                        Location::At(_) => panic!("Should be en-route"),
                        Location::EnRoute(to, t) => {
                            if t <= time_needed {
                                Location::At(to)
                            } else {
                                Location::EnRoute(to, t - time_needed)
                            }
                        }
                    });
                    let opened_valves = locations
                        .iter()
                        .filter_map(|loc| {
                            if let Location::At(valve) = loc {
                                Some(valve.name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let valves_remaining = self
                        .valves_remaining
                        .iter()
                        .filter(|valve| !opened_valves.contains(&valve.name.as_str()))
                        .cloned()
                        .collect();
                    let flow_rate = opened_valves
                        .iter()
                        .map(|name| valves.get(*name).unwrap().flow_rate)
                        .sum::<u64>();
                    Some(State {
                        locations,
                        time_left,
                        valves_remaining,
                        pressure_released: self.pressure_released + time_left * flow_rate,
                        max_pressure: Cell::new(None),
                    })
                } else {
                    None
                }
            })
    }

    fn max_total_pressure(&self, min_distance: u64) -> u64 {
        if let Some(pressure) = self.max_pressure.get() {
            return pressure;
        }

        let mut pressure_released = 0;
        let mut time = self.time_left;

        let mut rem_valves: &[&Valve] = &self.valves_remaining;

        while time > min_distance + 1 {
            for _ in self.locations.iter() {
                if rem_valves.is_empty() {
                    break;
                }
                pressure_released += rem_valves[0].flow_rate * (time - 2);
                rem_valves = &rem_valves[1..];
            }

            time -= min_distance + 1;
        }

        self.max_pressure
            .set(Some(self.pressure_released + pressure_released));

        self.pressure_released + pressure_released
    }
}

#[derive(Debug)]
pub struct Valve {
    name: String,
    flow_rate: u64,
    tunnels: Box<[String]>,
}

fn calculate_distances<F>(valves: &HashMap<String, Valve>, include_valve: F) -> Distances
where
    F: Fn(&Valve) -> bool,
{
    let mut distances = HashMap::new();
    for valve in valves.values() {
        let mut distance = 1;
        let mut next: Vec<_> = valve.tunnels.iter().collect();
        let mut ds: HashMap<String, u64> = Default::default();
        ds.insert(valve.name.clone(), 0);

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

    Distances(
        distances
            .into_iter()
            .filter_map(|(name, ds)| {
                if include_valve(valves.get(&name).unwrap()) {
                    Some((
                        name,
                        ds.into_iter()
                            .filter(|(name, _)| include_valve(valves.get(name).unwrap()))
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect(),
    )
}

fn find_most_pressure<const N: usize>(valves: &HashMap<String, Valve>, time_left: u64) -> u64 {
    fn include_valve(valve: &Valve) -> bool {
        valve.name == "AA" || valve.flow_rate > 0
    }

    assert!(valves.get("AA").unwrap().flow_rate == 0);

    let distances = calculate_distances(valves, include_valve);
    let min_distance = distances.min_distance();

    let mut valves_by_flow_rate: Vec<_> = valves
        .values()
        .filter(|valve| include_valve(valve))
        .collect();
    valves_by_flow_rate.sort_by(|a, b| a.flow_rate.cmp(&b.flow_rate).reverse());

    let mut stack: Vec<State<N>> = vec![State {
        locations: array::from_fn(|_| Location::At(valves.get("AA").unwrap())),
        time_left,
        valves_remaining: valves_by_flow_rate
            .iter()
            .filter(|valve| valve.flow_rate > 0)
            .cloned()
            .collect(),
        pressure_released: 0,
        max_pressure: Cell::new(None),
    }];

    let mut best = 0;
    while let Some(state) = stack.pop() {
        if state.max_total_pressure(min_distance) <= best {
            continue;
        }

        if state.pressure_released > best {
            best = state.pressure_released;
        }

        let mut successors = state
            .successors(valves, &distances)
            .filter(|state| state.max_total_pressure(min_distance) > best)
            .collect::<Vec<_>>();
        successors.sort_by_key(|state| state.max_total_pressure(min_distance));

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
        let part_one = find_most_pressure::<1>(&valves, 30).to_string();
        let part_two = find_most_pressure::<2>(&valves, 26).to_string();
        (Some(part_one), Some(part_two))
    }
}
