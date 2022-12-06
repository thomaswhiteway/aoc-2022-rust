use failure::Error;
use itertools::Itertools;

fn all_different<E: Eq>(values: &[E]) -> bool {
    (0..values.len()).all(|i| (i + 1..values.len()).all(|j| values[i] != values[j]))
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = String;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        Ok(data)
    }

    fn solve(data: Self::Problem) -> (Option<String>, Option<String>) {
        let part_one =
            data.chars()
                .tuple_windows()
                .enumerate()
                .find_map(|(index, (a, b, c, d))| {
                    if all_different(&[a, b, c, d]) {
                        Some((index + 4).to_string())
                    } else {
                        None
                    }
                });
        (part_one, None)
    }
}
