use std::collections::HashMap;
use std::num::ParseIntError;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

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
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day09Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day09Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let product_of_sizes_of_three_largest_basins =
                calculate_product_of_sizes_of_three_largest_basins(&file_contents)?;
            println!(
                "The product of the sizes of the three largest basins is {}.",
                product_of_sizes_of_three_largest_basins
            );
        }
        _ => {
            let summed_risk_levels = sum_risk_levels_of_lowest_points(&file_contents)?;
            println!(
                "The sum of risk levels of lowest points is {}.",
                summed_risk_levels
            );
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day09Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not sum risk levels of lowest points ({0})")]
    SumRiskLevelsOfLowestPoints(#[from] SumRiskLevelsOfLowestPointsError),
    #[error("Could not calculate product of sizes of three largest basins ({0})")]
    CalculateProductOfSizesOfThreeLargestBasins(
        #[from] CalculateProductOfSizesOfThreeLargestBasinsError,
    ),
}

pub fn sum_risk_levels_of_lowest_points(
    height_map: &str,
) -> Result<u128, SumRiskLevelsOfLowestPointsError> {
    Ok(find_low_points(&parse_height_map(height_map)?)
        .into_iter()
        .map(|low_point| low_point.value + 1)
        .map(|risk_level| risk_level as u128)
        .sum::<u128>())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SumRiskLevelsOfLowestPointsError {
    #[error("Could not parse height map ({0})")]
    ParseHeightMap(#[from] ParseHeightMapError),
}

pub fn calculate_product_of_sizes_of_three_largest_basins(
    height_map: &str,
) -> Result<u128, CalculateProductOfSizesOfThreeLargestBasinsError> {
    let height_map = parse_height_map(height_map)?;
    let low_points = find_low_points(&height_map);
    let mut basins = calculate_basins(&low_points, &height_map);
    basins.sort_by(|a, b| a.size.cmp(&b.size).reverse());
    if basins.len() >= 3 {
        Ok(basins[0].size as u128 * basins[1].size as u128 * basins[2].size as u128)
    } else {
        Err(CalculateProductOfSizesOfThreeLargestBasinsError::MissingBasins(basins.len()))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateProductOfSizesOfThreeLargestBasinsError {
    #[error("Could not parse height map ({0})")]
    ParseHeightMap(#[from] ParseHeightMapError),
    #[error("Not enough basins found (only {0})")]
    MissingBasins(usize),
}

fn parse_height_map(height_map: &str) -> Result<Vec<Vec<u8>>, ParseHeightMapError> {
    height_map
        .lines()
        .map(|line| {
            line.chars()
                .map(|char| {
                    char.to_string()
                        .parse::<u8>()
                        .map_err(|error| ParseHeightMapError::ParseInt(char, error))
                })
                .collect::<Result<Vec<u8>, ParseHeightMapError>>()
        })
        .collect::<Result<Vec<Vec<u8>>, ParseHeightMapError>>()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseHeightMapError {
    #[error("Could not parse '{0}' ({1})")]
    ParseInt(char, ParseIntError),
}

fn find_low_points(height_map: &[Vec<u8>]) -> Vec<LowPoint> {
    (0..height_map.len())
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
                Some(LowPoint {
                    position: Position { x, y },
                    value: height_map[y][x],
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct LowPoint {
    position: Position,
    value: u8,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn north(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.saturating_sub(1),
        }
    }

    fn south(&self, max: usize) -> Self {
        let y = self.y.saturating_add(1);
        Self {
            x: self.x,
            y: if y > max { self.y } else { y },
        }
    }

    fn east(&self, max: usize) -> Self {
        let x = self.x.saturating_add(1);
        Self {
            x: if x > max { self.x } else { x },
            y: self.y,
        }
    }

    fn west(&self) -> Self {
        Self {
            x: self.x.saturating_sub(1),
            y: self.y,
        }
    }
}

fn calculate_basins(low_points: &[LowPoint], height_map: &[Vec<u8>]) -> Vec<Basin> {
    low_points
        .iter()
        .map(|low_point| -> Basin {
            let mut positions_to_visit = vec![low_point.position];
            let mut positions_visited = Vec::new();

            let mut directions = HashMap::new();

            while !positions_to_visit.is_empty() {
                let current_position = positions_to_visit.remove(0);
                positions_visited.push(current_position);

                let mut add_next_positions =
                    |current_position: Position, next_position: Position| {
                        if next_position != current_position
                            && !(positions_to_visit.contains(&next_position)
                                || positions_visited.contains(&next_position))
                            && height_map[next_position.y][next_position.x] < 9
                        {
                            positions_to_visit.push(next_position);
                            directions
                                .entry(next_position)
                                .and_modify(|v| *v = current_position)
                                .or_insert(current_position);
                        }
                    };

                add_next_positions(current_position, current_position.north());
                add_next_positions(
                    current_position,
                    current_position.east(height_map[current_position.y].len().saturating_sub(1)),
                );
                add_next_positions(
                    current_position,
                    current_position.south(height_map.len().saturating_sub(1)),
                );
                add_next_positions(current_position, current_position.west());
            }

            Basin {
                low_point: *low_point,
                size: positions_visited.len(),
            }
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Basin {
    low_point: LowPoint,
    size: usize,
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

    #[test]
    fn calculate_product_of_sizes_of_three_largest_basins_should_return_1134() {
        // given
        let input = "2199943210\r\n3987894921\r\n9856789892\r\n8767896789\r\n9899965678";

        // when
        let product_of_sizes_of_three_largest_basins =
            calculate_product_of_sizes_of_three_largest_basins(&input);

        // then
        assert_eq!(product_of_sizes_of_three_largest_basins, Ok(1134));
    }
}
