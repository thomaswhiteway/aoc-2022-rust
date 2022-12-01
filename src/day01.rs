use failure::Error;

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = Box<[Box<[u32]>]>;

    fn parse_input(data: &str) -> Result<Self::Problem, Error> {
        let (mut elves, last) = data.lines().map(|line| line.parse::<u32>().ok()).fold(
            (vec![], vec![]),
            |(mut elves, mut current), value| {
                if let Some(calories) = value {
                    current.push(calories);
                    (elves, current)
                } else {
                    elves.push(current.into_boxed_slice());
                    (elves, vec![])
                }
            },
        );

        if !last.is_empty() {
            elves.push(last.into_boxed_slice());
        }

        Ok(elves.into_boxed_slice())
    }

    fn solve(elves: &Self::Problem) -> (Option<String>, Option<String>) {
        let mut elf_calories = elves
            .iter()
            .map(|elf| elf.iter().sum::<u32>())
            .collect::<Vec<_>>();
        elf_calories.sort_unstable_by(|a, b| a.cmp(b).reverse());

        let part_one = elf_calories[0].to_string();
        let part_two = elf_calories.iter().take(3).sum::<u32>().to_string();

        (Some(part_one), Some(part_two))
    }
}
