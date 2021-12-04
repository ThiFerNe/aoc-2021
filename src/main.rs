use clap::{crate_authors, crate_description, crate_version, App, ArgMatches};

use thiserror::Error;

mod lib;

use crate::lib::{day01, day02, day03, day04};

fn main() {
    let matches = App::new("Advent of Code 2021")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(day01::subcommand())
        .subcommand(day02::subcommand())
        .subcommand(day03::subcommand())
        .subcommand(day04::subcommand())
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
}
