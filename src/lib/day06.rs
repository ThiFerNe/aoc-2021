use std::num::ParseIntError;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day06";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 6: Lanternfish")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day06-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day06Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day06Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let simulation_days = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => 256,
        _ => 80,
    };
    let count_of_lanternfish = simulate_lanternfish(&file_contents, simulation_days)?
        .iter()
        .sum::<u128>();
    println!(
        "After {} days there are {} lanternfish.",
        simulation_days, count_of_lanternfish
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day06Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not simulate lanternfish ({0})")]
    SimulateLanternfish(#[from] SimulateLanternfishError),
}

pub fn simulate_lanternfish(
    ages_of_nearby_lanternfish: &str,
    simulation_days: u128,
) -> Result<[u128; 9], SimulateLanternfishError> {
    let mut lanternfish = ages_of_nearby_lanternfish
        .trim()
        .split(',')
        .map(|element| {
            element
                .parse::<u8>()
                .map_err(|error| SimulateLanternfishError::Parse(element.to_string(), error))
                .and_then(|days_left| {
                    if days_left > 8 {
                        Err(SimulateLanternfishError::TooYoung(days_left))
                    } else {
                        Ok(days_left)
                    }
                })
        })
        .collect::<Result<Vec<u8>, SimulateLanternfishError>>()?
        .into_iter()
        .fold([0u128; 9], |mut lanternfish, next| {
            lanternfish[next as usize] += 1;
            lanternfish
        });
    for _ in 1..=simulation_days {
        let reincarnating_lanternfish = lanternfish[0];
        for lanternfish_age in 1..=8 {
            let aging_lanternfish = lanternfish[lanternfish_age];
            lanternfish[lanternfish_age - 1] = aging_lanternfish;
        }
        lanternfish[6] += reincarnating_lanternfish;
        lanternfish[8] = reincarnating_lanternfish;
    }
    Ok(lanternfish)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SimulateLanternfishError {
    #[error("Could not parse lantern fish age \"{0}\" ({1})")]
    Parse(String, ParseIntError),
    #[error("Lanternfish with {0} days left is too young")]
    TooYoung(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulate_lanternfish_should_return_5_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 1);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().iter().sum::<u128>(), 5);
    }

    #[test]
    fn simulate_lanternfish_should_return_6_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 2);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().iter().sum::<u128>(), 6);
    }

    #[test]
    fn simulate_lanternfish_should_return_5934_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 80);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().iter().sum::<u128>(), 5934);
    }

    #[test]
    fn simulate_lanternfish_should_return_26984457539_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 256);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().iter().sum::<u128>(), 26984457539);
    }
}
