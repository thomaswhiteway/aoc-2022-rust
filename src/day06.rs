use failure::Error;

fn find_non_repeating<E: Eq>(values: &[E], len: usize) -> Option<usize> {
    let mut current_len = 0;
    for (i, next) in values.iter().enumerate() {
        let mut found_dup = false;
        for j in (i - current_len..i).rev() {
            if values[j] == *next {
                current_len = i - j;
                found_dup = true;
                break;
            }
        }
        if !found_dup {
            current_len += 1;
        }

        if current_len == len {
            return Some(i + 1);
        }
    }

    None
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
