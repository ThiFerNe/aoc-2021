use std::num::ParseIntError;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day01";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 1: Sonar Sweep")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day01-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day01Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day01Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let increases_count =
                count_depth_measurement_increases_three_sliding_window(&file_contents)?;
            println!(
                "Depth measurement increases (with sliding window of three) count is: {}",
                increases_count
            );
        }
        _ => {
            let increases_count = count_depth_measurement_increases(&file_contents)?;
            println!("Depth measurement increases count is: {}", increases_count);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day01Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error(transparent)]
    CountDepthMeasurementIncreases(#[from] CountDepthMeasurementIncreasesError),
    #[error(transparent)]
    CountDepthMeasurementIncreasesThreeSlidingWindow(
        #[from] CountDepthMeasurementIncreasesThreeSlidingWindowError,
    ),
}

pub fn count_depth_measurement_increases(
    depth_measurements: &str,
) -> Result<u128, CountDepthMeasurementIncreasesError> {
    count_depth_measurement_increases_with_sliding_window(depth_measurements, 1).map_err(Into::into)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountDepthMeasurementIncreasesError {
    #[error(transparent)]
    StrToNumVec(#[from] StrToNumVecError),
}

pub fn count_depth_measurement_increases_three_sliding_window(
    depth_measurement: &str,
) -> Result<u128, CountDepthMeasurementIncreasesThreeSlidingWindowError> {
    count_depth_measurement_increases_with_sliding_window(depth_measurement, 3).map_err(Into::into)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountDepthMeasurementIncreasesThreeSlidingWindowError {
    #[error(transparent)]
    StrToNumVec(#[from] StrToNumVecError),
}

fn count_depth_measurement_increases_with_sliding_window(
    depth_measurement: &str,
    window_size: usize,
) -> Result<u128, StrToNumVecError> {
    Ok(count_increases(&sliding_window(
        &str_to_num_vec(depth_measurement)?,
        window_size,
        |window| window.iter().sum(),
    )))
}

fn str_to_num_vec(content: &str) -> Result<Vec<u128>, StrToNumVecError> {
    content
        .split(|c| c == '\r' || c == '\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.parse::<u128>()
                .map_err(|error| StrToNumVecError::Parse(line.to_string(), error))
        })
        .collect::<Result<Vec<u128>, StrToNumVecError>>()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StrToNumVecError {
    #[error("Could not parse number \"{0}\" ({1})")]
    Parse(String, #[source] ParseIntError),
}

fn sliding_window<F: Fn(&[u128]) -> u128>(
    values: &[u128],
    window_size: usize,
    aggregator: F,
) -> Vec<u128> {
    values
        .iter()
        .fold(
            (Vec::new(), Vec::new()),
            |(mut output, mut window), value| {
                window.push(*value);
                if window.len() == window_size {
                    output.push(aggregator(&window[..]));
                    window.remove(0);
                }
                (output, window)
            },
        )
        .0
}

fn count_increases(values: &[u128]) -> u128 {
    values
        .iter()
        .fold((0u128, None), |(mut increase_count, previous), value| {
            if let Some(prev) = previous {
                if value > prev {
                    increase_count += 1;
                }
            }
            (increase_count, Some(value))
        })
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_7() {
        // given
        let input: &str = "199\r\n200\r\n208\r\n210\r\n200\r\n207\r\n240\r\n269\r\n260\r\n263";

        // when
        let result = count_depth_measurement_increases(&input);

        // then
        assert_eq!(result, Ok(7));
    }

    #[test]
    fn it_should_return_6() {
        // given
        let input: &str = "1\r\n2\r\n5\r\n8\r\n10\r\n33\r\n400";

        // when
        let result = count_depth_measurement_increases(&input);

        // then
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn it_should_return_0() {
        // given
        let input: &str = "100\r\n23\r\n18\r\n1";

        // when
        let result = count_depth_measurement_increases(&input);

        // then
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn it_should_return_5() {
        // given
        let input: &str = "199\r\n200\r\n208\r\n210\r\n200\r\n207\r\n240\r\n269\r\n260\r\n263";

        // when
        let result = count_depth_measurement_increases_three_sliding_window(&input);

        // then
        assert_eq!(result, Ok(5));
    }
}
