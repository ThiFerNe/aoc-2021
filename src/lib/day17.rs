use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day17";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 17: Trick Shot")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day17-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day17Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day17Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let highest_y_position_possible = find_highest_y_position_possible(&file_contents)?;
    println!(
        "The highest y position possible is {}.",
        highest_y_position_possible
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day17Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not find highest y position possible ({0})")]
    FindHighestYPositionPossible(#[from] FindHighestYPositionPossibleError),
}

pub fn find_highest_y_position_possible(
    input_target_area: &str,
) -> Result<i128, FindHighestYPositionPossibleError> {
    let target_area = TargetArea::from_str(input_target_area)?;
    (0..=target_area.max_x)
        .flat_map(|x| {
            (0..=(target_area.min_y.abs().max(target_area.max_x) * 2))
                .map(move |y| Velocity { x, y })
        })
        .filter_map(|velocity| simulate_shot(&Position { x: 0, y: 0 }, &velocity, &target_area))
        .reduce(|a, b| {
            if a.highest_y_position_reached > b.highest_y_position_reached {
                a
            } else {
                b
            }
        })
        .map(|simulated_shot| simulated_shot.highest_y_position_reached)
        .ok_or(FindHighestYPositionPossibleError::UnableToFind)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindHighestYPositionPossibleError {
    #[error("Could not parse target area from string ({0})")]
    TargetAreaFromStr(#[from] TargetAreaFromStrError),
    #[error("Was unable to find highest shot, maybe because there was no successful shot")]
    UnableToFind,
}

fn simulate_shot(
    start_position: &Position,
    initial_velocity: &Velocity,
    target_area: &TargetArea,
) -> Option<SimulatedShot> {
    let mut current_position = *start_position;
    let mut current_velocity = *initial_velocity;
    let mut step = 0;
    let mut highest_y_reached = current_position.y;
    while current_position.x < target_area.max_x && current_position.y > target_area.max_y {
        current_position.x += current_velocity.x;
        current_position.y += current_velocity.y;
        current_velocity.x -= current_velocity.x.signum();
        current_velocity.y -= 1;
        step += 1;
        if current_position.y > highest_y_reached {
            highest_y_reached = current_position.y;
        }
    }
    if target_area.contains(&current_position) {
        Some(SimulatedShot {
            end_position: current_position,
            step_count: step,
            highest_y_position_reached: highest_y_reached,
        })
    } else {
        None
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Position {
    x: i128,
    y: i128,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Velocity {
    x: i128,
    y: i128,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct TargetArea {
    min_x: i128,
    max_x: i128,
    min_y: i128,
    max_y: i128,
}

impl TargetArea {
    fn contains(&self, position: &Position) -> bool {
        position.x >= self.min_x
            && position.x <= self.max_x
            && position.y >= self.min_y
            && position.y <= self.max_y
    }
}

impl FromStr for TargetArea {
    type Err = TargetAreaFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: [&str; 2] = s
            .trim()
            .strip_prefix("target area: x=")
            .ok_or(TargetAreaFromStrError::MissingPrefix)?
            .split(", y=")
            .collect::<Vec<&str>>()
            .try_into()
            .map_err(|v: Vec<&str>| TargetAreaFromStrError::NotJustXAndYValues(v.len()))?;
        let parts: [[i128; 2]; 2] = parts
            .into_iter()
            .map(|part| {
                part.split("..")
                    .map(|value| {
                        value.parse().map_err(|error| {
                            TargetAreaFromStrError::ParseInt(value.to_string(), error)
                        })
                    })
                    .collect::<Result<Vec<i128>, TargetAreaFromStrError>>()
                    .and_then(|value| {
                        value.try_into().map_err(|v: Vec<i128>| {
                            TargetAreaFromStrError::NotJustMinAndMaxValues(v.len())
                        })
                    })
            })
            .collect::<Result<Vec<[i128; 2]>, TargetAreaFromStrError>>()?
            .try_into()
            .unwrap();
        Ok(Self {
            min_x: parts[0][0],
            max_x: parts[0][1],
            min_y: parts[1][0],
            max_y: parts[1][1],
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TargetAreaFromStrError {
    #[error("Target area string has wrong format, missing \"target area: x=\" prefix")]
    MissingPrefix,
    #[error("Target area string has wrong format, not just x and y values (got {0} part(s)) by splitting on \", y=\"")]
    NotJustXAndYValues(usize),
    #[error("Could not parse value \"{0}\" to integer ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Target area string has wrong format, not just min and max value (got {0} part(s)) by splitting on \"..\"")]
    NotJustMinAndMaxValues(usize),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct SimulatedShot {
    end_position: Position,
    step_count: u128,
    highest_y_position_reached: i128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_highest_y_position_possible() {
        // given
        let input = "target area: x=20..30, y=-10..-5\r\n";

        // when
        let highest_y_position_possible = find_highest_y_position_possible(input);

        // then
        assert_eq!(highest_y_position_possible, Ok(45));
    }

    #[test]
    fn simulate_shot_should_return_28_7_7() {
        // given
        let start_position = Position { x: 0, y: 0 };
        let initial_velocity = Velocity { x: 7, y: 2 };
        let target_area = TargetArea {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };

        // when
        let simulation_result = simulate_shot(&start_position, &initial_velocity, &target_area);

        // then
        assert_eq!(
            simulation_result,
            Some(SimulatedShot {
                end_position: Position { x: 28, y: -7 },
                step_count: 7,
                highest_y_position_reached: 3
            })
        );
    }

    #[test]
    fn simulate_shot_should_return_21_9_9() {
        // given
        let start_position = Position { x: 0, y: 0 };
        let initial_velocity = Velocity { x: 6, y: 3 };
        let target_area = TargetArea {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };

        // when
        let simulation_result = simulate_shot(&start_position, &initial_velocity, &target_area);

        // then
        assert_eq!(
            simulation_result,
            Some(SimulatedShot {
                end_position: Position { x: 21, y: -9 },
                step_count: 9,
                highest_y_position_reached: 6
            })
        );
    }

    #[test]
    fn simulate_shot_should_return_30_6_4() {
        // given
        let start_position = Position { x: 0, y: 0 };
        let initial_velocity = Velocity { x: 9, y: 0 };
        let target_area = TargetArea {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };

        // when
        let simulation_result = simulate_shot(&start_position, &initial_velocity, &target_area);

        // then
        assert_eq!(
            simulation_result,
            Some(SimulatedShot {
                end_position: Position { x: 30, y: -6 },
                step_count: 4,
                highest_y_position_reached: 0
            })
        );
    }

    #[test]
    fn simulate_shot_should_return_none() {
        // given
        let start_position = Position { x: 0, y: 0 };
        let initial_velocity = Velocity { x: 17, y: -4 };
        let target_area = TargetArea {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };

        // when
        let simulation_result = simulate_shot(&start_position, &initial_velocity, &target_area);

        // then
        assert_eq!(simulation_result, None);
    }
}
