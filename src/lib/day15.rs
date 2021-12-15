use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

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
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day15Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day15Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let multiply_map = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => (5, 5),
        _ => (1, 1),
    };
    let lowest_total_risk_of_any_path =
        calculate_lowest_total_risk_of_any_path(&file_contents, multiply_map)?;
    println!(
        "The lowest total risk of any path is {} with a map multiplied {:?}.",
        lowest_total_risk_of_any_path, multiply_map
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
    multiply_map: (usize, usize),
) -> Result<u128, CalculateLowestTotalRiskOfAnyPathError> {
    let risk_level_map = RiskLevelMap::from_str(risk_level_map)?.multiply(multiply_map);

    let start: (usize, usize) = (0, 0);
    let end: (usize, usize) = (
        risk_level_map.map[0].len() - 1,
        risk_level_map.map.len() - 1,
    );

    // Dijkstra takes around 3 minutes for the second part on my machine, but that's good enough for me

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

impl RiskLevelMap {
    fn multiply(self, multiply_map: (usize, usize)) -> Self {
        if multiply_map.0 == 0 || multiply_map.1 == 0 {
            Self { map: Vec::new() }
        } else {
            let mut output = self.map;
            if multiply_map.0 != 1 {
                output = output
                    .into_iter()
                    .map(|mut line| {
                        let line_len = line.len();
                        for _ in 0..(multiply_map.0 - 1) {
                            let offset = line.len() - line_len;
                            for index in offset..(offset + line_len) {
                                line.push(if line[index] == 9 { 1 } else { line[index] + 1 });
                            }
                        }
                        line
                    })
                    .collect::<Vec<Vec<u8>>>();
            }
            if multiply_map.1 != 1 {
                let height = output.len();
                for _ in 0..(multiply_map.1 - 1) {
                    let offset = output.len() - height;
                    for index in offset..(offset + height) {
                        output.push(
                            output[index]
                                .iter()
                                .map(|v| if *v == 9 { 1 } else { *v + 1 })
                                .collect(),
                        );
                    }
                }
            }
            Self { map: output }
        }
    }
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
        let lowest_total_risk = calculate_lowest_total_risk_of_any_path(input, (1, 1));

        // then
        assert_eq!(lowest_total_risk, Ok(40));
    }

    #[test]
    fn calculate_lowest_total_risk_of_any_path_should_return_315() {
        // given
        let input = "1163751742\r\n1381373672\r\n2136511328\r\n3694931569\r\n7463417111\r\n\
                            1319128137\r\n1359912421\r\n3125421639\r\n1293138521\r\n2311944581";

        // when
        let lowest_total_risk = calculate_lowest_total_risk_of_any_path(input, (5, 5));

        // then
        assert_eq!(lowest_total_risk, Ok(315));
    }
}
