use std::num::ParseIntError;
use std::ops::Add;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day07";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 7: The Treachery of Whales")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day07-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day07Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day07Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let needed_fuel_calculation = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => NeededFuelCalculation::Exponential,
        _ => NeededFuelCalculation::Linear,
    };
    let (position, usage) = determine_horizontal_position_with_least_fuel_usage(
        &file_contents,
        needed_fuel_calculation,
    )?;
    println!(
        "Horizontal position {} has with {} fuel usage the least usage with {:?} fuel usage",
        position, usage, needed_fuel_calculation
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day07Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not determine horizontal position with least fuel usage ({0})")]
    DetermineHorizontalPositionWithLeastFuelUsage(
        #[from] DetermineHorizontalPositionWithLeastFuelUsageError,
    ),
}

pub fn determine_horizontal_position_with_least_fuel_usage(
    horizontal_crab_positions: &str,
    needed_fuel_calculation: NeededFuelCalculation,
) -> Result<(HorizontalPosition, FuelUsage), DetermineHorizontalPositionWithLeastFuelUsageError> {
    let horizontal_positions = parse_horizontal_crab_positions(horizontal_crab_positions)?;
    let (min_pos, max_pos) = find_minimum_and_maximum(&horizontal_positions).ok_or(
        DetermineHorizontalPositionWithLeastFuelUsageError::MissingHorizontalCrabPositions,
    )?;
    (min_pos.value()..=max_pos.value())
        .map(HorizontalPosition::of)
        .filter_map(|target_position| {
            horizontal_positions
                .iter()
                .map(|start_position| {
                    target_position.needed_fuel_to(start_position, needed_fuel_calculation)
                })
                .reduce(FuelUsage::add)
                .map(|fuel_usage| (target_position, fuel_usage))
        })
        .reduce(
            |(target_position_a, fuel_usage_a), (target_position_b, fuel_usage_b)| {
                if fuel_usage_a < fuel_usage_b {
                    (target_position_a, fuel_usage_a)
                } else {
                    (target_position_b, fuel_usage_b)
                }
            },
        )
        .ok_or(DetermineHorizontalPositionWithLeastFuelUsageError::MissingHorizontalCrabPositions)
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct HorizontalPosition(u128);

impl HorizontalPosition {
    pub fn of(value: u128) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u128 {
        self.0
    }

    pub fn needed_fuel_to(
        &self,
        other: &Self,
        needed_fuel_calculation: NeededFuelCalculation,
    ) -> FuelUsage {
        match needed_fuel_calculation {
            NeededFuelCalculation::Linear => FuelUsage(
                self.0
                    .checked_sub(other.0)
                    .unwrap_or_else(|| other.0 - self.0),
            ),
            NeededFuelCalculation::Exponential => {
                let (min, max) = (self.0.min(other.0), self.0.max(other.0));
                FuelUsage((min..=max).map(|val| val - min).sum())
            }
        }
    }
}

impl std::fmt::Display for HorizontalPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == u128::MAX {
            write!(f, "INFINITE+")
        } else if self.0 == u128::MIN {
            write!(f, "INFINITE-")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NeededFuelCalculation {
    Linear,
    Exponential,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct FuelUsage(u128);

impl Add for FuelUsage {
    type Output = FuelUsage;

    fn add(self, rhs: Self) -> Self::Output {
        FuelUsage(self.0.saturating_add(rhs.0))
    }
}

impl std::fmt::Display for FuelUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == u128::MAX {
            write!(f, "INFINITE+")
        } else if self.0 == u128::MIN {
            write!(f, "INFINITE-")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DetermineHorizontalPositionWithLeastFuelUsageError {
    #[error(transparent)]
    ParseHorizontalCrabPositions(#[from] ParseHorizontalCrabPositionsError),
    #[error("Missing horizontal crab positions")]
    MissingHorizontalCrabPositions,
}

fn parse_horizontal_crab_positions(
    horizontal_crab_positions: &str,
) -> Result<Vec<HorizontalPosition>, ParseHorizontalCrabPositionsError> {
    Ok(horizontal_crab_positions
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split(',')
                .map(|element| {
                    element
                        .parse::<u128>()
                        .map(HorizontalPosition::of)
                        .map_err(|error| {
                            ParseHorizontalCrabPositionsError::Parse(element.to_string(), error)
                        })
                })
                .collect::<Result<Vec<HorizontalPosition>, ParseHorizontalCrabPositionsError>>()
        })
        .collect::<Result<Vec<Vec<HorizontalPosition>>, ParseHorizontalCrabPositionsError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<HorizontalPosition>>())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseHorizontalCrabPositionsError {
    #[error("Could not parse crab position \"{0}\" ({1})")]
    Parse(String, ParseIntError),
}

fn find_minimum_and_maximum(
    horizontal_positions: &[HorizontalPosition],
) -> Option<(HorizontalPosition, HorizontalPosition)> {
    if horizontal_positions.is_empty() {
        None
    } else {
        Some((
            *horizontal_positions.iter().min().unwrap(),
            *horizontal_positions.iter().max().unwrap(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_position_of() {
        // given
        let position = rand::random();

        // when
        let horizontal_position = HorizontalPosition::of(position);

        // then
        assert_eq!(horizontal_position.0, position);
    }

    #[test]
    fn horizontal_position_value() {
        // given
        let position = rand::random();
        let horizontal_position = HorizontalPosition::of(position);

        // when
        let value = horizontal_position.value();

        // then
        assert_eq!(value, position);
    }

    #[test]
    fn horizontal_position_needed_fuel_to_linear() {
        // given
        let horizontal_position_a = HorizontalPosition::of(4);
        let horizontal_position_b = HorizontalPosition::of(10);

        // when
        let fuel_usage = horizontal_position_a
            .needed_fuel_to(&horizontal_position_b, NeededFuelCalculation::Linear);

        // then
        assert_eq!(fuel_usage.0, 6);
    }

    #[test]
    fn horizontal_position_needed_fuel_to_exponential() {
        // given
        let horizontal_position_a = HorizontalPosition::of(16);
        let horizontal_position_b = HorizontalPosition::of(5);

        // when
        let fuel_usage = horizontal_position_a
            .needed_fuel_to(&horizontal_position_b, NeededFuelCalculation::Exponential);

        // then
        assert_eq!(fuel_usage.0, 66);
    }

    #[test]
    fn determine_horizontal_position_with_least_fuel_usage_should_return_2_37() {
        // given
        let input = "16,1,2,0,4,2,7,1,2,14\r\n";

        // when
        let result = determine_horizontal_position_with_least_fuel_usage(
            input,
            NeededFuelCalculation::Linear,
        );

        // then
        assert_eq!(result, Ok((HorizontalPosition::of(2), FuelUsage(37))))
    }

    #[test]
    fn determine_horizontal_position_with_least_fuel_usage_should_return_5_168() {
        // given
        let input = "16,1,2,0,4,2,7,1,2,14\r\n";

        // when
        let result = determine_horizontal_position_with_least_fuel_usage(
            input,
            NeededFuelCalculation::Exponential,
        );

        // then
        assert_eq!(result, Ok((HorizontalPosition::of(5), FuelUsage(168))))
    }
}
