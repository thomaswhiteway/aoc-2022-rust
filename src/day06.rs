use failure::Error;

fn all_different<E: Eq>(values: &[E]) -> bool {
    (0..values.len()).all(|i| (i + 1..values.len()).all(|j| values[i] != values[j]))
}

fn find_non_repeating<E: Eq>(values: &[E], len: usize) -> Option<usize> {
    values.windows(len).enumerate().find_map(|(index, values)| {
        if all_different(values) {
            Some(index + len)
        } else {
            None
        }
    })
}

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = String;

    fn parse_input(data: String) -> Result<Self::Problem, Error> {
        Ok(data)
    }

    fn solve(data: Self::Problem) -> (Option<String>, Option<String>) {
        let chars = data.chars().collect::<Vec<_>>();
        let part_one = find_non_repeating(&chars, 4).unwrap().to_string();
        let part_two = find_non_repeating(&chars, 14).unwrap().to_string();

        (Some(part_one), Some(part_two))
    }
}
