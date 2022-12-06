use aocf::Aoc;
use failure::Error;

use std::path::PathBuf;
use structopt::StructOpt;

use aoc2022::{read_input, solve_day, Part};

#[derive(StructOpt, Debug)]
struct Opt {
    day: u32,
    input: Option<PathBuf>,

    #[structopt(long)]
    submit: Option<Part>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let mut aoc = Aoc::new()
        .parse_cli(false)
        .year(Some(2022))
        .day(Some(opt.day))
        .init()?;

    let data = read_input(opt.input, &mut aoc)
        .map_err(|err| failure::err_msg(format!("Failed to read input: {}", err)))?;

    solve_day(opt.day, data, &mut aoc, opt.submit)?;

    Ok(())
}
