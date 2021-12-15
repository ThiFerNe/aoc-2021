use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day15";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 15: Chiton")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day15-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day15Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day15Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let lowest_total_risk_of_any_path = calculate_lowest_total_risk_of_any_path(&file_contents)?;
    println!(
        "The lowest total risk of any path is {}.",
        lowest_total_risk_of_any_path
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day15Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate lowest total risk of any path ({0})")]
    CalculateLowestTotalRiskOfAnyPath(#[from] CalculateLowestTotalRiskOfAnyPathError),
}

pub fn calculate_lowest_total_risk_of_any_path(
    risk_level_map: &str,
) -> Result<u128, CalculateLowestTotalRiskOfAnyPathError> {
    let risk_level_map = RiskLevelMap::from_str(risk_level_map)?;

    let start: (usize, usize) = (0, 0);
    let end: (usize, usize) = (
        risk_level_map.map[0].len() - 1,
        risk_level_map.map.len() - 1,
    );

    let mut distance = risk_level_map
        .map
        .iter()
        .map(|line| vec![u128::MAX; line.len()])
        .collect::<Vec<Vec<u128>>>();
    let mut predecessor = risk_level_map
        .map
        .iter()
        .map(|line| vec![None; line.len()])
        .collect::<Vec<Vec<Option<(usize, usize)>>>>();
    distance[start.1][start.0] = 0;
    let mut open = (0..risk_level_map.map.len())
        .flat_map(|y| (0..risk_level_map.map[y].len()).map(move |x| (x, y)))
        .collect::<Vec<(usize, usize)>>();
    while !open.is_empty() {
        let current = open.remove(
            open.iter()
                .enumerate()
                .reduce(|a, b| {
                    if distance[a.1 .1][a.1 .0] < distance[b.1 .1][b.1 .0] {
                        a
                    } else {
                        b
                    }
                })
                .unwrap()
                .0,
        );
        let mut distance_update = |neighbour: (usize, usize)| {
            if open.contains(&neighbour) {
                let alternative = distance[current.1][current.0]
                    + risk_level_map.map[neighbour.1][neighbour.0] as u128;
                if alternative < distance[neighbour.1][neighbour.0] {
                    distance[neighbour.1][neighbour.0] = alternative;
                    predecessor[neighbour.1][neighbour.0] = Some(current);
                }
            }
        };
        if current.0 > 0 {
            distance_update((current.0 - 1, current.1));
        }
        if current.1 > 0 {
            distance_update((current.0, current.1 - 1));
        }
        distance_update((current.0, current.1 + 1));
        distance_update((current.0 + 1, current.1));
    }

    Ok(distance[end.1][end.0])
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateLowestTotalRiskOfAnyPathError {
    #[error("Could not parse risk level map from string ({0})")]
    RiskLevelMapFromStr(#[from] RiskLevelMapFromStrError),
}

#[derive(Debug)]
struct RiskLevelMap {
    map: Vec<Vec<u8>>,
}

impl FromStr for RiskLevelMap {
    type Err = RiskLevelMapFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            map: s
                .lines()
                .map(|line| {
                    line.chars()
                        .map(|character| {
                            character.to_string().parse::<u8>().map_err(|error| {
                                RiskLevelMapFromStrError::ParseInt(character, error)
                            })
                        })
                        .collect::<Result<Vec<u8>, RiskLevelMapFromStrError>>()
                })
                .collect::<Result<Vec<Vec<u8>>, RiskLevelMapFromStrError>>()?,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RiskLevelMapFromStrError {
    #[error("Could not parse '{0}' ({1})")]
    ParseInt(char, #[source] ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_lowest_total_risk_of_any_path_should_return_40() {
        // given
        let input = "1163751742\r\n1381373672\r\n2136511328\r\n3694931569\r\n7463417111\r\n\
                            1319128137\r\n1359912421\r\n3125421639\r\n1293138521\r\n2311944581";

        // when
        let lowest_total_risk = calculate_lowest_total_risk_of_any_path(input);

        // then
        assert_eq!(lowest_total_risk, Ok(40));
    }
}
