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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Location {
    At(String),
    EnRoute(String, u64),
}

impl Location {
    fn valve(&self) -> &str {
        match self {
            Location::At(loc) => loc.as_str(),
            Self::EnRoute(loc, _) => loc.as_str()
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
}

#[derive(Debug)]
struct State {
    locations: Vec<Location>,
    time_left: u64,
    valves_opened: HashSet<String>,
    pressure_released: u64,
}

fn all_location_combos(locations: &[Vec<Location>]) -> Vec<Vec<Location>> {
    if locations.len() == 1 {
        locations[0].iter().map(|loc| vec![loc.clone()]).collect()
    } else {
        locations[0].iter().flat_map(|loc| all_location_combos(&locations[1..]).into_iter().filter_map(|mut locs| {
            if locs.iter().all(|loc2| loc2.valve() != loc.valve()) {
                locs.insert(0, loc.clone());
                Some(locs)
            } else {
                None
            }
        })).collect()
    }
}

impl State {
    fn successors(
        &self,
        valves: &HashMap<String, Valve>,
        distances: &Distances,
    ) -> Vec<State> {

        let next_locations_per_actor = self.locations.iter().map(|location| {
            if let Location::At(loc) = location {
                valves
                .values()
                .filter(|valve| valve.flow_rate > 0 && !self.valves_opened.contains(&valve.name))
                .map(|valve| Location::EnRoute(valve.name.clone(), distances.distance_between(loc, &valve.name) + 1))
                .collect()
            } else {
                vec![location.clone()]
            }
        }).collect::<Vec<_>>();

        all_location_combos(&next_locations_per_actor)
            .into_iter()
            .filter_map(|next_locs| {
                let time_needed = next_locs.iter().map(|loc| loc.time_remaining()).min().unwrap();
                if self.time_left > time_needed {
                    let time_left = self.time_left - time_needed;
                    let locations = next_locs.into_iter().map(|loc| match loc {
                        Location::At(_) => panic!("Should be en-route"),
                        Location::EnRoute(to, t) => if t <= time_needed  {
                            Location::At(to)
                        } else {
                            Location::EnRoute(to, t - time_needed)
                        }
                    }).collect::<Vec<_>>();
                    let mut valves_opened = self.valves_opened.clone();
                    for loc in locations.iter() {
                        if let Location::At(name) = loc {
                            valves_opened.insert(name.clone());
                        }
                    }
                    let flow_rate = locations.iter().filter_map(|loc| if let Location::At(name) = loc {
                        Some(valves.get(name).unwrap().flow_rate)
                    } else {
                        None
                    }).sum::<u64>();
                    Some(State {
                        locations,
                        time_left,
                        valves_opened,
                        pressure_released: self.pressure_released + time_left * flow_rate,
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

fn calculate_distances(valves: &HashMap<String, Valve>) -> Distances {
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

    Distances(distances)
}

fn max_remaining_pressure(state: &State, valves: &[&Valve]) -> u64 {
    let mut rem_valves = valves
        .iter()
        .filter(|valve| !state.valves_opened.contains(&valve.name)).collect::<Vec<_>>();

    let mut pressure_released = 0;
    let mut time = state.time_left;

    while time > 2 {
        for _ in state.locations.iter() {
            if rem_valves.is_empty() {
                break;
            }
            pressure_released += rem_valves.remove(0).flow_rate * (time - 2);
        }

        time -= 2;
    }

    for location in state.locations.iter() {
        if let Location::At(name) = location {
            if !state.valves_opened.contains(name) {
                pressure_released += valves
                .iter()
                .find(|valve| valve.name.as_str() == name)
                .unwrap()
                .flow_rate
                * (state.time_left - 1)
            }
        }
    }

    pressure_released
}

fn find_most_pressure(valves: &HashMap<String, Valve>, num_actors: usize, time_left: u64) -> u64 {
    let mut stack = vec![State {
        locations: (0..num_actors).map(|_| Location::At("AA".to_string())).collect(),
        time_left,
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

        if state.pressure_released > best {
            best = state.pressure_released;
        }

        if state.time_left <= 2 {
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
        let part_one = find_most_pressure(&valves, 1, 30).to_string();
        let part_two = find_most_pressure(&valves, 2, 26).to_string();
        (Some(part_one), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use super::{Location, all_location_combos};

    #[test]
    fn test_location_combos() {
        let loc_a = Location::EnRoute("A".to_string(), 1);
        let loc_b1 = Location::EnRoute("B".to_string(), 2);
        let loc_b2 = Location::EnRoute("B".to_string(), 3);
        let loc_c = Location::EnRoute("C".to_string(), 4);
        let locations = vec![
            vec![loc_a.clone(), loc_b1.clone()],
            vec![loc_b2.clone(), loc_c.clone()]
        ];

        assert_eq!(all_location_combos(&locations), vec![
            vec![loc_a.clone(), loc_b2.clone()],
            vec![loc_a.clone(), loc_c.clone()],
            vec![loc_b1.clone(), loc_c.clone()],
        ]);
    }
}
