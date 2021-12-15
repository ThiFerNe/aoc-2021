use clap::{crate_authors, crate_description, crate_version, App, ArgMatches};

use thiserror::Error;

mod lib;

use crate::lib::{
    day01, day02, day03, day04, day05, day06, day07, day08, day09, day10, day11, day12, day13,
    day14, day15,
};

fn main() {
    let matches = App::new("Advent of Code 2021")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(day01::subcommand())
        .subcommand(day02::subcommand())
        .subcommand(day03::subcommand())
        .subcommand(day04::subcommand())
        .subcommand(day05::subcommand())
        .subcommand(day06::subcommand())
        .subcommand(day07::subcommand())
        .subcommand(day08::subcommand())
        .subcommand(day09::subcommand())
        .subcommand(day10::subcommand())
        .subcommand(day11::subcommand())
        .subcommand(day12::subcommand())
        .subcommand(day13::subcommand())
        .subcommand(day14::subcommand())
        .subcommand(day15::subcommand())
        .get_matches();
    if let Err(error) = handle_matches(matches) {
        eprintln!("Error: {}", error);
    }
}

fn handle_matches(matches: ArgMatches) -> Result<(), HandleMatchesError> {
    let (subcommand_name, optional_subcommand_matches) = matches.subcommand();
    match optional_subcommand_matches {
        Some(subcommand_matches) => match subcommand_name {
            day01::SUBCOMMAND_NAME => day01::handle(subcommand_matches).map_err(Into::into),
            day02::SUBCOMMAND_NAME => day02::handle(subcommand_matches).map_err(Into::into),
            day03::SUBCOMMAND_NAME => day03::handle(subcommand_matches).map_err(Into::into),
            day04::SUBCOMMAND_NAME => day04::handle(subcommand_matches).map_err(Into::into),
            day05::SUBCOMMAND_NAME => day05::handle(subcommand_matches).map_err(Into::into),
            day06::SUBCOMMAND_NAME => day06::handle(subcommand_matches).map_err(Into::into),
            day07::SUBCOMMAND_NAME => day07::handle(subcommand_matches).map_err(Into::into),
            day08::SUBCOMMAND_NAME => day08::handle(subcommand_matches).map_err(Into::into),
            day09::SUBCOMMAND_NAME => day09::handle(subcommand_matches).map_err(Into::into),
            day10::SUBCOMMAND_NAME => day10::handle(subcommand_matches).map_err(Into::into),
            day11::SUBCOMMAND_NAME => day11::handle(subcommand_matches).map_err(Into::into),
            day12::SUBCOMMAND_NAME => day12::handle(subcommand_matches).map_err(Into::into),
            day13::SUBCOMMAND_NAME => day13::handle(subcommand_matches).map_err(Into::into),
            day14::SUBCOMMAND_NAME => day14::handle(subcommand_matches).map_err(Into::into),
            day15::SUBCOMMAND_NAME => day15::handle(subcommand_matches).map_err(Into::into),
            subcommand_name => Err(HandleMatchesError::SubCommandDoesNotExist(
                subcommand_name.to_string(),
            )),
        },
        None => Err(HandleMatchesError::SubCommandArgumentsAreMissing),
    }
}

#[derive(Debug, Error)]
enum HandleMatchesError {
    #[error("SubCommand \"{0}\" does not exist")]
    SubCommandDoesNotExist(String),
    #[error("Missing arguments for subcommand")]
    SubCommandArgumentsAreMissing,
    #[error(transparent)]
    Day01Error(#[from] day01::Day01Error),
    #[error(transparent)]
    Day02Error(#[from] day02::Day02Error),
    #[error(transparent)]
    Day03Error(#[from] day03::Day03Error),
    #[error(transparent)]
    Day04Error(#[from] day04::Day04Error),
    #[error(transparent)]
    Day05Error(#[from] day05::Day05Error),
    #[error(transparent)]
    Day06Error(#[from] day06::Day06Error),
    #[error(transparent)]
    Day07Error(#[from] day07::Day07Error),
    #[error(transparent)]
    Day08Error(#[from] day08::Day08Error),
    #[error(transparent)]
    Day09Error(#[from] day09::Day09Error),
    #[error(transparent)]
    Day10Error(#[from] day10::Day10Error),
    #[error(transparent)]
    Day11Error(#[from] day11::Day11Error),
    #[error(transparent)]
    Day12Error(#[from] day12::Day12Error),
    #[error(transparent)]
    Day13Error(#[from] day13::Day13Error),
    #[error(transparent)]
    Day14Error(#[from] day14::Day14Error),
    #[error(transparent)]
    Day15Error(#[from] day15::Day15Error),
}
