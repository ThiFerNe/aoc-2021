use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;

use clap::Arg;

use thiserror::Error;

pub mod day01;
pub mod day02;
pub mod day03;
pub mod day04;
pub mod day05;
pub mod day06;
pub mod day07;
pub mod day08;
pub mod day09;
pub mod day10;
pub mod day11;
pub mod day12;
pub mod day13;
pub mod day14;
pub mod day15;
pub mod day16;
pub mod day17;
pub mod day18;
pub mod day19;
pub mod day20;
pub mod day21;

fn read_file_contents(file_path: Option<&str>) -> Result<String, ReadFileContentsError> {
    let mut content = String::new();
    File::open(file_path.ok_or(ReadFileContentsError::MissingFilePath)?)
        .map_err(ReadFileContentsError::OpeningFile)?
        .read_to_string(&mut content)
        .map_err(ReadFileContentsError::ReadingFile)?;
    Ok(content)
}

#[derive(Debug, Error)]
pub enum ReadFileContentsError {
    #[error("Missing file path")]
    MissingFilePath,
    #[error("Failed opening file ({0})")]
    OpeningFile(#[source] IoError),
    #[error("Failed reading file ({0})")]
    ReadingFile(#[source] IoError),
}

fn clap_arg_puzzle_part_time_two() -> Arg<'static, 'static> {
    Arg::with_name("puzzle_part")
        .short("p")
        .long("part")
        .value_name("PUZZLE_PART")
        .help("selects the part of the puzzle solution")
        .possible_values(&["one", "two", "1", "2"])
        .default_value("two")
}
