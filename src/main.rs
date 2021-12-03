use clap::{crate_authors, crate_description, crate_version, App, ArgMatches};

use thiserror::Error;

mod lib;

use crate::lib::day01;

fn main() {
    let matches = App::new("Advent of Code 2021")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(day01::subcommand())
        .get_matches();
    if let Err(error) = handle_matches(matches) {
        eprintln!("Error: {}", error);
    }
}

fn handle_matches(matches: ArgMatches) -> Result<(), HandleMatchesError> {
    match matches.subcommand() {
        ("day01", optional_subcommand_matches) => match optional_subcommand_matches {
            Some(subcommand_matches) => day01::handle(subcommand_matches).map_err(Into::into),
            None => Err(HandleMatchesError::SubCommandArgumentsAreMissing),
        },
        (subcommand_name, _) => Err(HandleMatchesError::SubCommandDoesNotExist(
            subcommand_name.to_string(),
        )),
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
}
