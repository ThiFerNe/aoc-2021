use std::num::ParseIntError;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name("day01")
        .about("My solution for day 1")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day01-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day01Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file).map_err(|error| {
        Day01Error::ReadFileContentsError(input_file.map(str::to_string), error)
    })?;
    let increases_count = count_depth_measurement_increases(&file_contents)?;
    println!("Depth measurement increases count is: {}", increases_count);
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day01Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContentsError(Option<String>, #[source] ReadFileContentsError),
    #[error(transparent)]
    CountDepthMeasurementIncreasesError(#[from] CountDepthMeasurementIncreasesError),
}

pub fn count_depth_measurement_increases(
    depth_measurements: &str,
) -> Result<u128, CountDepthMeasurementIncreasesError> {
    Ok(depth_measurements
        .split(|c| c == '\r' || c == '\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.parse::<u128>().map_err(|error| {
                CountDepthMeasurementIncreasesError::Parse(line.to_string(), error)
            })
        })
        .collect::<Result<Vec<u128>, CountDepthMeasurementIncreasesError>>()?
        .into_iter()
        .fold((0u128, None), |(mut increase_count, previous), value| {
            if let Some(prev) = previous {
                if value > prev {
                    increase_count += 1;
                }
            }
            (increase_count, Some(value))
        })
        .0)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountDepthMeasurementIncreasesError {
    #[error("Could not parse number \"{0}\" ({1})")]
    Parse(String, #[source] ParseIntError),
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
}
