use priority_queue::PriorityQueue;
use std::{collections::HashSet, fmt::Debug, hash::Hash};

pub trait State: Sized + Eq + PartialEq + Hash {
    fn heuristic(&self) -> u64;
    fn successors(&self) -> Vec<(u64, Self)>;
    fn is_end(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Priority(u64);

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0).reverse())
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct Entry<S: State> {
    cost: u64,
    state: S,
    route: Vec<S>,
}

impl<S: State> Entry<S> {
    fn priority(&self) -> Priority {
        Priority(self.cost + self.state.heuristic())
    }
}

impl<S: State> PartialEq for Entry<S> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<S: State> Eq for Entry<S> {}

impl<S: State> Hash for Entry<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state)
    }
}

pub fn solve<S: State + Clone + Debug>(start: S) -> Result<(u64, Vec<S>), HashSet<S>> {
    let mut queue = PriorityQueue::new();
    let entry = Entry {
        cost: 0,
        state: start.clone(),
        route: vec![start],
    };
    let priority = entry.priority();
    queue.push(entry, priority);

    let mut visited = HashSet::new();

    while let Some((Entry { cost, state, route }, _)) = queue.pop() {
        if state.is_end() {
            return Ok((cost, route));
        }

        visited.insert(state.clone());

        for (delta, next_state) in state.successors() {
            if visited.contains(&next_state) {
                continue;
            }

            let mut route = route.clone();
            route.push(next_state.clone());
            let next_entry = Entry {
                cost: cost + delta,
                state: next_state,
                route,
            };
            let priority = next_entry.priority();

            queue.push_increase(next_entry, priority);
        }
    }

    Err(visited)
}
