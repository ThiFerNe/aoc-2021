use std::num::ParseIntError;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day09";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 9: Smoke Basin")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day09-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day09Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day09Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let summed_risk_levels = sum_risk_levels_of_lowest_points(&file_contents)?;
    println!(
        "The sum of risk levels of lowest points is {}.",
        summed_risk_levels
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day09Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not sum risk levels of lowest points ({0})")]
    SumRiskLevelsOfLowestPoints(#[from] SumRiskLevelsOfLowestPointsError),
}

pub fn sum_risk_levels_of_lowest_points(
    height_map: &str,
) -> Result<u128, SumRiskLevelsOfLowestPointsError> {
    let height_map = height_map
        .lines()
        .map(|line| {
            line.chars()
                .map(|char| {
                    char.to_string()
                        .parse::<u8>()
                        .map_err(|error| SumRiskLevelsOfLowestPointsError::ParseInt(char, error))
                })
                .collect::<Result<Vec<u8>, SumRiskLevelsOfLowestPointsError>>()
        })
        .collect::<Result<Vec<Vec<u8>>, SumRiskLevelsOfLowestPointsError>>()?;
    Ok((0..height_map.len())
        .into_iter()
        .flat_map(|y| (0..height_map[y].len()).into_iter().map(move |x| (x, y)))
        .filter_map(|(x, y)| {
            let lower_than_west = x == 0 || height_map[y][x] < height_map[y][x - 1];
            let lower_than_north = y == 0 || height_map[y][x] < height_map[y - 1][x];
            let lower_than_east =
                (x + 1) >= height_map[y].len() || height_map[y][x] < height_map[y][x + 1];
            let lower_than_south =
                (y + 1) >= height_map.len() || height_map[y][x] < height_map[y + 1][x];
            if lower_than_west && lower_than_north && lower_than_east && lower_than_south {
                Some(height_map[y][x])
            } else {
                None
            }
        })
        .map(|low_point| low_point + 1)
        .map(|risk_level| risk_level as u128)
        .sum::<u128>())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SumRiskLevelsOfLowestPointsError {
    #[error("Could not parse '{0}' ({1})")]
    ParseInt(char, ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_risk_levels_should_return_15() {
        // given
        let input = "2199943210\r\n3987894921\r\n9856789892\r\n8767896789\r\n9899965678";

        // when
        let summed_risk_levels = sum_risk_levels_of_lowest_points(input);

        // then
        assert_eq!(summed_risk_levels, Ok(15));
    }
}
