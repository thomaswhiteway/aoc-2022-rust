use failure::Error;

pub struct Solver {}

impl super::Solver for Solver {
    type Problem = ();

    fn parse_input(_data: &str) -> Result<Self::Problem, Error> {
        Ok(())
    }

    fn solve(_problem: &Self::Problem) -> (Option<String>, Option<String>) {
        (None, None)
    }
}
