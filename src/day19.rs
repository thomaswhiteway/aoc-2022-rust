mod parse {
    use crate::parsers::unsigned;
    use failure::{err_msg, Error};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::newline,
        combinator::{all_consuming, map, value},
        multi::{many1, separated_list1},
        sequence::{delimited, separated_pair, terminated, tuple},
        IResult,
    };

    use super::{Blueprint, Resource, ResourceCosts};

    fn resource(input: &str) -> IResult<&str, Resource> {
        alt((
            value(Resource::Ore, tag("ore")),
            value(Resource::Clay, tag("clay")),
            value(Resource::Obsidian, tag("obsidian")),
            value(Resource::Geode, tag("geode")),
        ))(input)
    }

    fn cost(input: &str) -> IResult<&str, (u64, Resource)> {
        separated_pair(unsigned, tag(" "), resource)(input)
    }

    fn costs(input: &str) -> IResult<&str, ResourceCosts> {
        map(separated_list1(tag(" and "), cost), Vec::into_boxed_slice)(input)
    }

    fn costs_for_robot(input: &str) -> IResult<&str, (Resource, ResourceCosts)> {
        tuple((
            delimited(tag("Each "), resource, tag(" robot ")),
            delimited(tag("costs "), costs, tag(".")),
        ))(input)
    }

    fn blueprint(input: &str) -> IResult<&str, Blueprint> {
        map(
            tuple((
                delimited(tag("Blueprint "), unsigned, tag(": ")),
                separated_list1(tag(" "), costs_for_robot),
            )),
            |(index, resource_costs)| Blueprint::new(index, &resource_costs),
        )(input)
    }

    fn blueprints(input: &str) -> IResult<&str, Box<[Blueprint]>> {
        map(many1(terminated(blueprint, newline)), Vec::into_boxed_slice)(input)
    }

    pub(super) fn parse_input(data: &str) -> Result<Box<[Blueprint]>, Error> {
        all_consuming(blueprints)(data)
            .map(|(_, blueprints)| blueprints)
            .map_err(|err| err_msg(format!("Failed to parse blueprints: {}", err)))
    }
}

use self::parse::parse_input;
use crate::common::div_ceil;
use failure::{err_msg, Error};
use std::{
    array,
    cmp::{max, Ordering},
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

impl Resource {
    const NUM: usize = 4;

    fn all() -> impl DoubleEndedIterator<Item = Self> {
        [
            Resource::Ore,
            Resource::Clay,
            Resource::Obsidian,
            Resource::Geode,
        ]
        .into_iter()
    }
}

impl TryFrom<usize> for Resource {
    type Error = Error;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Resource::Ore),
            1 => Ok(Resource::Clay),
            2 => Ok(Resource::Obsidian),
            3 => Ok(Resource::Geode),
            _ => Err(err_msg(format!("Unknown resource {}", value))),
        }
    }
}

type ResourceCosts = Box<[(u64, Resource)]>;

#[derive(Debug)]
pub struct Blueprint {
    index: u64,
    costs_for_robot: ResourceArray<ResourceArray<u64>>,
    max_robot_costs: ResourceArray<u64>,
}

impl Blueprint {
    fn new(index: u64, robot_costs: &[(Resource, ResourceCosts)]) -> Self {
        let mut costs_for_robot: ResourceArray<ResourceArray<u64>> = ResourceArray::default();
        for (robot_type, resource_costs) in robot_costs.iter() {
            for (cost, resource) in resource_costs.iter() {
                costs_for_robot[*robot_type][*resource] += cost;
            }
        }
        let max_robot_costs = ResourceArray::from_fn(|resource| {
            costs_for_robot
                .values()
                .map(|robot_costs| robot_costs[resource])
                .max()
                .unwrap()
        });
        Blueprint {
            index,
            costs_for_robot,
            max_robot_costs,
        }
    }

    fn need_resource_for_robot(&self, resource: Resource, robot_type: Resource) -> bool {
        self.resource_amount_for_robot(robot_type, resource) > 0
    }

    fn resource_amount_for_robot(&self, robot_type: Resource, resource: Resource) -> u64 {
        self.costs_for_robot[robot_type][resource]
    }

    fn resources_for_robot(&self, robot_type: Resource) -> ResourceArray<u64> {
        self.costs_for_robot[robot_type]
    }

    fn max_cost_for_robot(&self, resource: Resource) -> u64 {
        self.max_robot_costs[resource]
    }
}

#[derive(Debug, Default, Clone)]
struct ResourceArray<T> {
    values: [T; Resource::NUM],
}

impl<T: PartialEq> PartialEq for ResourceArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values().zip(other.values()).all(|(x, y)| x == y)
    }
}

impl<T: PartialOrd> PartialOrd for ResourceArray<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(x, y)| x.partial_cmp(y))
            .reduce(|x, y| match (x, y) {
                (Some(x), Some(y)) => match (x, y) {
                    (Ordering::Equal, y) => Some(y),
                    (x, Ordering::Equal) => Some(x),
                    (x, y) => {
                        if x == y {
                            Some(x)
                        } else {
                            None
                        }
                    }
                },
                _ => None,
            })
            .flatten()
    }
}

impl<T: Copy> Copy for ResourceArray<T> {}

impl<T> ResourceArray<T> {
    fn from_fn<F>(func: F) -> Self
    where
        F: Fn(Resource) -> T,
    {
        ResourceArray {
            values: array::from_fn(|i| func(Resource::try_from(i).unwrap())),
        }
    }

    #[allow(unused)]
    fn iter(&self) -> impl Iterator<Item = (Resource, &T)> + '_ {
        Resource::all().zip(self.values.iter())
    }

    fn values(&self) -> impl Iterator<Item = &T> + '_ {
        self.values.iter()
    }

    #[allow(unused)]
    fn into_values(self) -> impl Iterator<Item = T> {
        self.values.into_iter()
    }
}

impl<T> From<[T; Resource::NUM]> for ResourceArray<T> {
    fn from(values: [T; Resource::NUM]) -> Self {
        ResourceArray { values }
    }
}

impl ResourceArray<u64> {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        let mut values = [0; Resource::NUM];
        for resource in Resource::all() {
            values[resource as usize] = self[resource].checked_sub(other[resource])?;
        }
        Some(ResourceArray { values })
    }
}

impl<T> Index<Resource> for ResourceArray<T> {
    type Output = T;

    fn index(&self, resource: Resource) -> &Self::Output {
        &self.values[resource as usize]
    }
}

impl<T> IndexMut<Resource> for ResourceArray<T> {
    fn index_mut(&mut self, resource: Resource) -> &mut Self::Output {
        &mut self.values[resource as usize]
    }
}

#[derive(Clone)]
struct State<'a> {
    blueprint: &'a Blueprint,
    minutes_passed: u64,
    minutes_remaining: u64,
    resources: ResourceArray<u64>,
    num_robots: ResourceArray<u64>,
    history: Vec<(u64, Resource)>
}

impl<'a> Debug for State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "State {{ blueprint: {}, time: ({},{}), resources: {:?}, robots: {:?} }}",
            self.blueprint.index,
            self.minutes_passed,
            self.minutes_remaining,
            self.resources,
            self.num_robots
        )
    }
}

impl<'a> State<'a> {
    fn new(blueprint: &'a Blueprint, minutes_remaining: u64) -> Self {
        let mut num_robots = ResourceArray::default();
        num_robots[Resource::Ore] += 1;
        State {
            blueprint,
            minutes_passed: 0,
            minutes_remaining,
            resources: ResourceArray::default(),
            num_robots,
            history: vec![],
        }
    }

    fn is_producing_resource(&self, resource: Resource) -> bool {
        self.num_robots[resource] > 0
    }

    fn have_prerequisites_for_robot(&self, robot_type: Resource) -> bool {
        Resource::all().all(|resource| {
            !self.blueprint.need_resource_for_robot(resource, robot_type)
                || self.is_producing_resource(resource)
        })
    }

    fn time_until_ready_to_produce(&self, robot_type: Resource) -> Option<u64> {
        Resource::all()
            .map(|resource| {
                let amount_needed = self
                    .blueprint
                    .resource_amount_for_robot(robot_type, resource);
                let extra_amount_needed = amount_needed.saturating_sub(self.resources[resource]);
                let num_robots = self.num_robots[resource];
                if extra_amount_needed == 0 {
                    Some(0)
                } else if num_robots > 0 {
                    Some(div_ceil(extra_amount_needed, num_robots))
                } else {
                    None
                }
            })
            .collect::<Option<Vec<u64>>>()
            .map(|vals| *vals.iter().max().unwrap())
    }

    fn build_robot(&self, robot_type: Resource) -> Option<Self> {
        let resources_needed = self.blueprint.resources_for_robot(robot_type);
        if !(resources_needed <= self.resources) {
            return None;
        }

        let mut state = self.clone();
        state.resources = state.resources.checked_sub(&resources_needed)?;

        state = state.advance(1)?;
        state.num_robots[robot_type] += 1;

        state.history.push((state.minutes_passed, robot_type));

        Some(state)
    }

    fn advance_to(&self, minutes: u64) -> Option<Self> {
        self.advance(minutes.checked_sub(self.minutes_passed)?)
    }

    fn advance(&self, minutes: u64) -> Option<Self> {
        if minutes <= self.minutes_remaining {
            let resources = ResourceArray::from_fn(|resource| {
                self.projected_resource_amount(resource, minutes)
            });

            Some(State {
                blueprint: self.blueprint,
                minutes_passed: self.minutes_passed + minutes,
                minutes_remaining: self.minutes_remaining - minutes,
                resources,
                num_robots: self.num_robots,
                history: self.history.clone(),
            })
        } else {
            None
        }
    }

    fn projected_resource_amount(&self, resource: Resource, minutes: u64) -> u64 {
        self.resources[resource] + minutes * self.num_robots[resource]
    }

    fn better_than(&self, other: &Self) -> bool {
        self.minutes_passed <= other.minutes_passed
            && self
                .advance_to(other.minutes_passed)
                .map(|state| state.resources > other.resources)
                .unwrap_or_default()
    }
}

fn find_max_geodes(blueprint: &Blueprint, minutes: u64) -> u64 {
    println!("Checking blueprint {}", blueprint.index);
    let mut stack = vec![State::new(blueprint, minutes)];

    let mut max_geodes = 0;

    while let Some(state) = stack.pop() {
        let possible_robot_types = Resource::all()
            // We can only build one robot per minute, so if the most a single robot can cost
            // of a resource is X, then there's no point building more than X of that robot.
            // We always want to build geode robots.
            .filter(|&robot_type| {
                robot_type == Resource::Geode || state.num_robots[robot_type] < blueprint.max_cost_for_robot(robot_type)
            })
            // Can't build a robot if we can't produce all the resources for it.
            .filter(|&robot_type| state.have_prerequisites_for_robot(robot_type))
            .collect::<Vec<_>>();

        let possible_states = possible_robot_types
            .iter()
            .cloned()
            .filter_map(|robot_type| {
                state
                    .time_until_ready_to_produce(robot_type)
                    .and_then(|minutes| state.advance(minutes))
                    .and_then(|before| {
                        before
                            .build_robot(robot_type)
                            .map(|after| (robot_type, before, after))
                    })
            })
            .collect::<Vec<_>>();

        let next_states = possible_states
            .iter()
            .filter(|(robot_type, before, _)| {
                possible_states.iter().all(|(robot_type2, _, after2)| {
                    robot_type == robot_type2 || !(after2.better_than(before))
                })
            })
            .map(|(_, _, after)| after)
            .cloned()
            .collect::<Vec<_>>();

        if next_states.is_empty() {
            max_geodes = max(
                state.projected_resource_amount(Resource::Geode, state.minutes_remaining),
                max_geodes,
            );
        } else {
            stack.extend(next_states);
        }
    }

    max_geodes
}

fn get_quality(blueprint: &Blueprint, minutes: u64) -> u64 {
    blueprint.index * find_max_geodes(blueprint, minutes)
}

fn total_quality(blueprints: &[Blueprint], minutes: u64) -> u64 {
    blueprints.iter().map(|blueprint| get_quality(blueprint, minutes)).sum()
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Blueprint]>;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        parse_input(&data)
    }

    fn solve(blueprints: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one = total_quality(&blueprints, 24).to_string();
        let part_two = blueprints[..3].iter().map(|blueprint| find_max_geodes(blueprint, 32)).product::<u64>().to_string();
        (Some(part_one), Some(part_two))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order_resource() {
        let a = ResourceArray::from([1, 2, 3, 4]);
        let b = ResourceArray::from([1, 2, 3, 5]);
        let c = ResourceArray::from([1, 1, 3, 5]);

        assert!(a < b);
        assert!(!(b < a));
        assert!(!(a < c));
        assert!(!(c < a));
        assert!(c < b);
        assert!(!(b < c));
    }
}
