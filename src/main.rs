use aocf::Aoc;
use failure::{err_msg, Error};

use std::{path::PathBuf, time::Instant};
use structopt::StructOpt;

use aoc2022::{read_input, solve_day, Part};

#[derive(StructOpt, Debug)]
struct Opt {
    day: Option<u32>,
    input: Option<PathBuf>,

    #[structopt(long)]
    submit: Option<Part>,
}

fn run_day(day: u32, input: Option<PathBuf>, submit: Option<Part>) -> Result<(), Error> {
    let mut aoc = Aoc::new()
        .parse_cli(false)
        .year(Some(2022))
        .day(Some(day))
        .init()?;

    let data = read_input(input, &mut aoc)
        .map_err(|err| failure::err_msg(format!("Failed to read input: {}", err)))?;

    solve_day(day, data, &mut aoc, submit)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    if let Some(day) = opt.day {
        run_day(day, opt.input, opt.submit)?;
    } else {
        if opt.input.is_some() {
            return Err(err_msg("Can't provide input for all days"));
        }
        if opt.submit.is_some() {
            return Err(err_msg("Can't submit solution for all days"));
        }
        for day in 1..=25 {
            println!("Day {}", day);
            let start = Instant::now();
            run_day(day, None, None)?;
            let elapsed = start.elapsed();
            if elapsed.as_secs() > 0 {
                println!("Took {}.{:03}s", elapsed.as_secs(), elapsed.subsec_millis());
            } else if elapsed.as_millis() > 0 {
                println!("Took {}ms", elapsed.as_millis());
            } else {
                println!("Took {}µs", elapsed.as_micros());
            }
            println!();
        }
    }

    Ok(())
}
