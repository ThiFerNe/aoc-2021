use clap::{crate_authors, crate_description, crate_version, App, ArgMatches};

use thiserror::Error;

fn main() {
    let matches = App::new("Advent of Code 2021")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();
    if let Err(error) = handle_matches(matches) {
        eprintln!("Error: {}", error);
    }
}

fn handle_matches(matches: ArgMatches) -> Result<(), HandleMatchesError> {
    match matches.subcommand() {
        (subcommand_name, _) => Err(HandleMatchesError::SubCommandDoesNotExist(
            subcommand_name.to_string(),
        )),
    }
}

#[derive(Debug, Error)]
enum HandleMatchesError {
    #[error("SubCommand \"{0}\" does not exist")]
    SubCommandDoesNotExist(String),
}
