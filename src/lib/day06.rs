use clap::{App, Arg, ArgMatches, SubCommand};
use std::num::ParseIntError;

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

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
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day06Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day06Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let count_of_lanternfish = simulate_lanternfish(&file_contents, 80)?.len();
    println!(
        "After 80 days there are {} lanternfish.",
        count_of_lanternfish
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
) -> Result<Vec<Lanternfish>, SimulateLanternfishError> {
    let mut lanternfish = ages_of_nearby_lanternfish
        .trim()
        .split(',')
        .map(|element| {
            element
                .parse::<u8>()
                .map(Into::into)
                .map_err(|error| SimulateLanternfishError::Parse(element.to_string(), error))
        })
        .collect::<Result<Vec<Lanternfish>, SimulateLanternfishError>>()?;
    for _ in 1..=simulation_days {
        lanternfish = lanternfish
            .into_iter()
            .flat_map(|mut lanternfish| {
                if lanternfish.is_soon_to_die() {
                    let offspring = lanternfish.produce_offspring();
                    lanternfish.reincarnate();
                    vec![lanternfish, offspring]
                } else {
                    lanternfish.grow_old();
                    vec![lanternfish]
                }
            })
            .collect();
    }
    Ok(lanternfish)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lanternfish(u8);

impl Lanternfish {
    // allowing dead code so the test case is nicer and maybe its needed in the second part
    #[allow(dead_code)]
    fn new(days_left: u8) -> Self {
        Self(days_left)
    }

    // allowing dead code so the test case is nicer and maybe its needed in the second part
    #[allow(dead_code)]
    fn days_left(&self) -> u8 {
        self.0
    }

    fn is_soon_to_die(&self) -> bool {
        self.0 == 0
    }

    fn produce_offspring(&self) -> Self {
        Lanternfish(8)
    }

    fn reincarnate(&mut self) {
        self.0 = 6;
    }

    fn grow_old(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl From<u8> for Lanternfish {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SimulateLanternfishError {
    #[error("Could not parse lantern fish age \"{0}\" ({1})")]
    Parse(String, ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lanternfish_new() {
        // when
        let lanternfish_a = Lanternfish::new(7);
        let lanternfish_b = Lanternfish::new(8);

        // then
        assert_eq!(lanternfish_a, Lanternfish(7));
        assert_eq!(lanternfish_b, Lanternfish(8));
    }

    #[test]
    fn lanternfish_days_left() {
        // given
        let young_lanternfish = Lanternfish::new(8);
        let middle_aged_lanternfish = Lanternfish::new(4);
        let old_lanternfish = Lanternfish::new(0);

        // when + then
        assert_eq!(young_lanternfish.days_left(), 8);
        assert_eq!(middle_aged_lanternfish.days_left(), 4);
        assert_eq!(old_lanternfish.days_left(), 0);
    }

    #[test]
    fn lanternfish_is_soon_to_die() {
        // given
        let young_lanternfish = Lanternfish::new(8);
        let middle_aged_lanternfish = Lanternfish::new(4);
        let old_lanternfish = Lanternfish::new(0);

        // when + then
        assert!(!young_lanternfish.is_soon_to_die());
        assert!(!middle_aged_lanternfish.is_soon_to_die());
        assert!(old_lanternfish.is_soon_to_die());
    }

    #[test]
    fn lanternfish_produce_offspring() {
        // given
        let lanternfish = Lanternfish::new(0);

        // when
        let offspring = lanternfish.produce_offspring();

        // then
        assert_eq!(offspring, Lanternfish::new(8));
    }

    #[test]
    fn lanternfish_reincarnate() {
        // given
        let mut lanternfish_a = Lanternfish::new(0);
        let mut lanternfish_b = Lanternfish::new(1);
        let mut lanternfish_c = Lanternfish::new(6);

        // when
        lanternfish_a.reincarnate();
        lanternfish_b.reincarnate();
        lanternfish_c.reincarnate();

        // then
        assert_eq!(lanternfish_a.days_left(), 6);
        assert_eq!(lanternfish_b.days_left(), 6);
        assert_eq!(lanternfish_c.days_left(), 6);
    }

    #[test]
    fn grow_old() {
        // given
        let mut lanternfish_a = Lanternfish::new(1);
        let mut lanternfish_b = Lanternfish::new(7);

        // when
        lanternfish_a.grow_old();
        lanternfish_b.grow_old();

        // then
        assert_eq!(lanternfish_a.days_left(), 0);
        assert_eq!(lanternfish_b.days_left(), 6);
    }

    #[test]
    fn simulate_lanternfish_should_return_5_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 1);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().len(), 5);
    }

    #[test]
    fn simulate_lanternfish_should_return_6_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 2);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().len(), 6);
    }

    #[test]
    fn simulate_lanternfish_should_return_5934_elements() {
        // given
        let input = "3,4,3,1,2\r\n";

        // when
        let lanternfish = simulate_lanternfish(input, 80);

        // then
        assert!(lanternfish.is_ok());
        assert_eq!(lanternfish.unwrap().len(), 5934);
    }
}
