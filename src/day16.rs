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
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Location {
    At(String),
    EnRoute(String, u64),
}

impl Location {
    fn valve(&self) -> &str {
        match self {
            Location::At(loc) => loc.as_str(),
            Self::EnRoute(loc, _) => loc.as_str(),
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
    fn distance_between(&self, from: &str, to: &str) -> u64 {
        *self.0.get(from).unwrap().get(to).unwrap()
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
struct State {
    locations: Vec<Location>,
    time_left: u64,
    valves_opened: HashSet<String>,
    pressure_released: u64,
    max_pressure: Cell<Option<u64>>,
}

fn all_location_combos(locations: &[Vec<Location>]) -> Vec<Vec<Location>> {
    if locations.len() == 1 {
        locations[0].iter().map(|loc| vec![loc.clone()]).collect()
    } else {
        locations[0]
            .iter()
            .flat_map(|loc| {
                all_location_combos(&locations[1..])
                    .into_iter()
                    .filter_map(|mut locs| {
                        if locs.iter().all(|loc2| loc2.valve() != loc.valve()) {
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

impl State {
    fn successors(
        &self,
        valves: &HashMap<String, Valve>,
        useful_valves: &[&Valve],
        distances: &Distances,
    ) -> Vec<State> {
        let next_locations_per_actor = self
            .locations
            .iter()
            .map(|location| {
                if let Location::At(loc) = location {
                    useful_valves
                        .iter()
                        .filter(|valve| {
                            valve.flow_rate > 0 && !self.valves_opened.contains(&valve.name)
                        })
                        .map(|valve| {
                            Location::EnRoute(
                                valve.name.clone(),
                                distances.distance_between(loc, &valve.name) + 1,
                            )
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
                    let locations = next_locs
                        .into_iter()
                        .map(|loc| match loc {
                            Location::At(_) => panic!("Should be en-route"),
                            Location::EnRoute(to, t) => {
                                if t <= time_needed {
                                    Location::At(to)
                                } else {
                                    Location::EnRoute(to, t - time_needed)
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                    let mut valves_opened = self.valves_opened.clone();
                    for loc in locations.iter() {
                        if let Location::At(name) = loc {
                            valves_opened.insert(name.clone());
                        }
                    }
                    let flow_rate = locations
                        .iter()
                        .filter_map(|loc| {
                            if let Location::At(name) = loc {
                                Some(valves.get(name).unwrap().flow_rate)
                            } else {
                                None
                            }
                        })
                        .sum::<u64>();
                    Some(State {
                        locations,
                        time_left,
                        valves_opened,
                        pressure_released: self.pressure_released + time_left * flow_rate,
                        max_pressure: Cell::new(None),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn max_total_pressure(&self, valves: &[&Valve], min_distance: u64) -> u64 {
        if let Some(pressure) = self.max_pressure.get() {
            return pressure;
        }

        let mut rem_valves = valves
            .iter()
            .filter(|valve| !self.valves_opened.contains(&valve.name))
            .collect::<Vec<_>>();

        let mut pressure_released = 0;
        let mut time = self.time_left;

        while time > min_distance + 1 {
            for _ in self.locations.iter() {
                if rem_valves.is_empty() {
                    break;
                }
                pressure_released += rem_valves.remove(0).flow_rate * (time - 2);
            }

            time -= min_distance + 1;
        }

        self.max_pressure
            .set(Some(self.pressure_released + pressure_released));

        self.pressure_released + pressure_released
    }
}

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

fn find_most_pressure(valves: &HashMap<String, Valve>, num_actors: usize, time_left: u64) -> u64 {
    let mut stack = vec![State {
        locations: (0..num_actors)
            .map(|_| Location::At("AA".to_string()))
            .collect(),
        time_left,
        valves_opened: Default::default(),
        pressure_released: 0,
        max_pressure: Cell::new(None),
    }];

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

    let mut count = 0;
    let mut best = 0;
    while let Some(state) = stack.pop() {
        count += 1;
        if state.max_total_pressure(&valves_by_flow_rate, min_distance) <= best {
            continue;
        }

        if state.pressure_released > best {
            best = state.pressure_released;
        }

        let mut successors = state.successors(valves, &valves_by_flow_rate, &distances);
        successors.retain(|state| state.max_total_pressure(&valves_by_flow_rate, min_distance) > best);
        successors.sort_by_key(|state| state.max_total_pressure(&valves_by_flow_rate, min_distance));

        stack.extend(successors);
    }
    println!("visited {} states", count);
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
        let part_one = find_most_pressure(&valves, 1, 30).to_string();
        let part_two = find_most_pressure(&valves, 2, 26).to_string();
        (Some(part_one), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use super::{all_location_combos, Location};

    #[test]
    fn test_location_combos() {
        let loc_a = Location::EnRoute("A".to_string(), 1);
        let loc_b1 = Location::EnRoute("B".to_string(), 2);
        let loc_b2 = Location::EnRoute("B".to_string(), 3);
        let loc_c = Location::EnRoute("C".to_string(), 4);
        let locations = vec![
            vec![loc_a.clone(), loc_b1.clone()],
            vec![loc_b2.clone(), loc_c.clone()],
        ];

        assert_eq!(
            all_location_combos(&locations),
            vec![
                vec![loc_a.clone(), loc_b2.clone()],
                vec![loc_a.clone(), loc_c.clone()],
                vec![loc_b1.clone(), loc_c.clone()],
            ]
        );
    }
}
