use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub mod part1;
pub mod part2;

pub const SUBCOMMAND_NAME: &str = "day02";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 2: Dive!")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day02-input"),
        )
        .arg(
            Arg::with_name("puzzle_part")
                .short("p")
                .long("part")
                .value_name("PUZZLE_PART")
                .help("selects the part of the puzzle solution")
                .possible_values(&["one", "two", "1", "2"])
                .default_value("two"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day02Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day02Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let mut submarine = part2::Submarine::default();
            submarine.drive(&file_contents)?;
            println!("Drove submarine to {:?}.", submarine.position);
        }
        _ => {
            let mut submarine = part1::Submarine::default();
            submarine.drive(&file_contents)?;
            println!("Drove submarine to {:?}.", submarine.position);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day02Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not drive submarine ({0})")]
    Part1SubmarineDrive(#[from] part1::SubmarineDriveError),
    #[error("Could not drive submarine ({0})")]
    Part2SubmarineDrive(#[from] part2::SubmarineDriveError),
}
