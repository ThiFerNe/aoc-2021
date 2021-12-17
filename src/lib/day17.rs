use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

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
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day17Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day17Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let count_of_distinct_initial_velocities =
                count_distinct_initial_velocities(&file_contents)?;
            println!(
                "There are {} distinct initial velocity values causing the probe to be within the target area after any step.",
                count_of_distinct_initial_velocities
            );
        }
        _ => {
            let highest_y_position_possible = find_highest_y_position_possible(&file_contents)?;
            println!(
                "The highest y position possible is {}.",
                highest_y_position_possible
            );
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day17Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count distinct initial velocities ({0})")]
    CountDistinctInitialVelocities(#[from] CountDistinctInitialVelocitiesError),
    #[error("Could not find highest y position possible ({0})")]
    FindHighestYPositionPossible(#[from] FindHighestYPositionPossibleError),
}

pub fn count_distinct_initial_velocities(
    input_target_area: &str,
) -> Result<usize, CountDistinctInitialVelocitiesError> {
    let target_area = TargetArea::from_str(input_target_area)?;
    Ok(get_all_p(&target_area)
        .into_iter()
        .fold(Vec::new(), |mut distinct, simulated_shot| {
            if !distinct.contains(&simulated_shot) {
                distinct.push(simulated_shot);
            }
            distinct
        })
        .len())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountDistinctInitialVelocitiesError {
    #[error("Could not parse target area from string ({0})")]
    TargetAreaFromStr(#[from] TargetAreaFromStrError),
}

pub fn find_highest_y_position_possible(
    input_target_area: &str,
) -> Result<i128, FindHighestYPositionPossibleError> {
    let target_area = TargetArea::from_str(input_target_area)?;
    get_all_p(&target_area)
        .into_iter()
        .reduce(|a, b| {
            if a.highest_y_position_reached() > b.highest_y_position_reached() {
                a
            } else {
                b
            }
        })
        .map(|simulated_shot| simulated_shot.highest_y_position_reached())
        .ok_or(FindHighestYPositionPossibleError::UnableToFind)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindHighestYPositionPossibleError {
    #[error("Could not parse target area from string ({0})")]
    TargetAreaFromStr(#[from] TargetAreaFromStrError),
    #[error("Was unable to find highest shot, maybe because there was no successful shot")]
    UnableToFind,
}

fn get_all_p(target_area: &TargetArea) -> Vec<SimulatedShot> {
    get_all(
        0,
        target_area.max_x,
        -target_area.min_y.abs(),
        target_area.min_y.abs(),
        &Position { x: 0, y: 0 },
        target_area,
    )
}

fn get_all(
    min_x: i128,
    max_x: i128,
    min_y: i128,
    max_y: i128,
    initial_position: &Position,
    target_area: &TargetArea,
) -> Vec<SimulatedShot> {
    (min_x..=max_x)
        .flat_map(|x| (min_y..=max_y).map(move |y| Velocity { x, y }))
        .map(|velocity| simulate_shot(initial_position, &velocity, target_area))
        .filter(SimulatedShot::was_successful)
        .collect()
}

fn simulate_shot(
    start_position: &Position,
    initial_velocity: &Velocity,
    target_area: &TargetArea,
) -> SimulatedShot {
    let mut current_position = *start_position;
    let mut current_velocity = *initial_velocity;
    let mut mid_positions = Vec::new();
    while current_position.x <= target_area.max_x
        && current_position.y >= target_area.min_y
        && !target_area.contains(&current_position)
    {
        if current_position != *start_position {
            mid_positions.push(current_position);
        }
        current_position.x += current_velocity.x;
        current_position.y += current_velocity.y;
        current_velocity.x -= current_velocity.x.signum();
        current_velocity.y -= 1;
    }
    SimulatedShot {
        start_position: *start_position,
        initial_velocity: *initial_velocity,
        target_area: *target_area,
        mid_positions,
        end_position: if target_area.contains(&current_position) {
            Some(current_position)
        } else {
            None
        },
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

#[derive(Debug, Eq, PartialEq, Clone)]
struct SimulatedShot {
    start_position: Position,
    initial_velocity: Velocity,
    target_area: TargetArea,
    mid_positions: Vec<Position>,
    end_position: Option<Position>,
}

impl SimulatedShot {
    fn was_successful(&self) -> bool {
        self.end_position.is_some()
    }

    fn highest_y_position_reached(&self) -> i128 {
        let mid_positions_max = self
            .mid_positions
            .iter()
            .map(|pos| pos.y)
            .max()
            .unwrap_or(self.start_position.y);
        self.start_position.y.max(match &self.end_position {
            None => mid_positions_max,
            Some(end_position) => end_position.y.max(mid_positions_max),
        })
    }
}

impl std::fmt::Display for SimulatedShot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min_x = self
            .mid_positions
            .iter()
            .map(|p| p.x)
            .min()
            .unwrap_or(self.start_position.x)
            .min(self.start_position.x)
            .min(self.target_area.min_x)
            .min(self.target_area.max_x);
        let min_x = match &self.end_position {
            None => min_x,
            Some(end_position) => min_x.min(end_position.x),
        };

        let max_x = self
            .mid_positions
            .iter()
            .map(|p| p.x)
            .max()
            .unwrap_or(self.start_position.x)
            .max(self.start_position.x)
            .max(self.target_area.min_x)
            .max(self.target_area.max_x);
        let max_x = match &self.end_position {
            None => max_x,
            Some(end_position) => max_x.max(end_position.x),
        };

        let min_y = self
            .mid_positions
            .iter()
            .map(|p| p.y)
            .min()
            .unwrap_or(self.start_position.y)
            .min(self.start_position.y)
            .min(self.target_area.min_y)
            .min(self.target_area.max_y);
        let min_y = match &self.end_position {
            None => min_y,
            Some(end_position) => min_y.min(end_position.y),
        };

        let max_y = self
            .mid_positions
            .iter()
            .map(|p| p.y)
            .max()
            .unwrap_or(self.start_position.y)
            .max(self.start_position.y)
            .max(self.target_area.min_y)
            .max(self.target_area.max_y);
        let max_y = match &self.end_position {
            None => max_y,
            Some(end_position) => max_y.max(end_position.y),
        };

        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                let position = Position { x, y };
                if self.start_position == position {
                    write!(f, "S")?;
                } else if self.end_position.map(|p| p == position).unwrap_or(false)
                    || self.mid_positions.iter().any(|p| *p == position)
                {
                    write!(f, "#")?;
                } else if self.target_area.contains(&position) {
                    write!(f, "T")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
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
    fn test_count_distinct_initial_velocities() {
        // given
        let input = "target area: x=20..30, y=-10..-5\r\n";

        // when
        let count_of_distinct_initial_velocities = count_distinct_initial_velocities(input);

        // then
        assert_eq!(count_of_distinct_initial_velocities, Ok(112));
    }

    #[test]
    fn test_get_all_p() {
        // given
        let input = "target area: x=20..30, y=-10..-5\r\n";

        // when
        let got = get_all_p(&TargetArea::from_str(input).unwrap())
            .into_iter()
            .map(|simulated_shot| simulated_shot.initial_velocity)
            .collect::<Vec<Velocity>>();

        // then
        let mut expected = {
            vec![
                Velocity { x: 23, y: -10 },
                Velocity { x: 25, y: -9 },
                Velocity { x: 27, y: -5 },
                Velocity { x: 29, y: -6 },
                Velocity { x: 22, y: -6 },
                Velocity { x: 21, y: -7 },
                Velocity { x: 9, y: 0 },
                Velocity { x: 27, y: -7 },
                Velocity { x: 24, y: -5 },
                Velocity { x: 25, y: -7 },
                Velocity { x: 26, y: -6 },
                Velocity { x: 25, y: -5 },
                Velocity { x: 6, y: 8 },
                Velocity { x: 11, y: -2 },
                Velocity { x: 20, y: -5 },
                Velocity { x: 29, y: -10 },
                Velocity { x: 6, y: 3 },
                Velocity { x: 28, y: -7 },
                Velocity { x: 8, y: 0 },
                Velocity { x: 30, y: -6 },
                Velocity { x: 29, y: -8 },
                Velocity { x: 20, y: -10 },
                Velocity { x: 6, y: 7 },
                Velocity { x: 6, y: 4 },
                Velocity { x: 6, y: 1 },
                Velocity { x: 14, y: -4 },
                Velocity { x: 21, y: -6 },
                Velocity { x: 26, y: -10 },
                Velocity { x: 7, y: -1 },
                Velocity { x: 7, y: 7 },
                Velocity { x: 8, y: -1 },
                Velocity { x: 21, y: -9 },
                Velocity { x: 6, y: 2 },
                Velocity { x: 20, y: -7 },
                Velocity { x: 30, y: -10 },
                Velocity { x: 14, y: -3 },
                Velocity { x: 20, y: -8 },
                Velocity { x: 13, y: -2 },
                Velocity { x: 7, y: 3 },
                Velocity { x: 28, y: -8 },
                Velocity { x: 29, y: -9 },
                Velocity { x: 15, y: -3 },
                Velocity { x: 22, y: -5 },
                Velocity { x: 26, y: -8 },
                Velocity { x: 25, y: -8 },
                Velocity { x: 25, y: -6 },
                Velocity { x: 15, y: -4 },
                Velocity { x: 9, y: -2 },
                Velocity { x: 15, y: -2 },
                Velocity { x: 12, y: -2 },
                Velocity { x: 28, y: -9 },
                Velocity { x: 12, y: -3 },
                Velocity { x: 24, y: -6 },
                Velocity { x: 23, y: -7 },
                Velocity { x: 25, y: -10 },
                Velocity { x: 7, y: 8 },
                Velocity { x: 11, y: -3 },
                Velocity { x: 26, y: -7 },
                Velocity { x: 7, y: 1 },
                Velocity { x: 23, y: -9 },
                Velocity { x: 6, y: 0 },
                Velocity { x: 22, y: -10 },
                Velocity { x: 27, y: -6 },
                Velocity { x: 8, y: 1 },
                Velocity { x: 22, y: -8 },
                Velocity { x: 13, y: -4 },
                Velocity { x: 7, y: 6 },
                Velocity { x: 28, y: -6 },
                Velocity { x: 11, y: -4 },
                Velocity { x: 12, y: -4 },
                Velocity { x: 26, y: -9 },
                Velocity { x: 7, y: 4 },
                Velocity { x: 24, y: -10 },
                Velocity { x: 23, y: -8 },
                Velocity { x: 30, y: -8 },
                Velocity { x: 7, y: 0 },
                Velocity { x: 9, y: -1 },
                Velocity { x: 10, y: -1 },
                Velocity { x: 26, y: -5 },
                Velocity { x: 22, y: -9 },
                Velocity { x: 6, y: 5 },
                Velocity { x: 7, y: 5 },
                Velocity { x: 23, y: -6 },
                Velocity { x: 28, y: -10 },
                Velocity { x: 10, y: -2 },
                Velocity { x: 11, y: -1 },
                Velocity { x: 20, y: -9 },
                Velocity { x: 14, y: -2 },
                Velocity { x: 29, y: -7 },
                Velocity { x: 13, y: -3 },
                Velocity { x: 23, y: -5 },
                Velocity { x: 24, y: -8 },
                Velocity { x: 27, y: -9 },
                Velocity { x: 30, y: -7 },
                Velocity { x: 28, y: -5 },
                Velocity { x: 21, y: -10 },
                Velocity { x: 7, y: 9 },
                Velocity { x: 6, y: 6 },
                Velocity { x: 21, y: -5 },
                Velocity { x: 27, y: -10 },
                Velocity { x: 7, y: 2 },
                Velocity { x: 30, y: -9 },
                Velocity { x: 21, y: -8 },
                Velocity { x: 22, y: -7 },
                Velocity { x: 24, y: -9 },
                Velocity { x: 20, y: -6 },
                Velocity { x: 6, y: 9 },
                Velocity { x: 29, y: -5 },
                Velocity { x: 8, y: -2 },
                Velocity { x: 27, y: -8 },
                Velocity { x: 30, y: -5 },
                Velocity { x: 24, y: -7 },
            ]
        };
        for n in got {
            expected.retain(|v| *v != n);
        }
        assert_eq!(expected, Vec::new());
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
            SimulatedShot {
                start_position,
                initial_velocity,
                target_area,
                mid_positions: vec![
                    Position { x: 7, y: 2 },
                    Position { x: 13, y: 3 },
                    Position { x: 18, y: 3 },
                    Position { x: 22, y: 2 },
                    Position { x: 25, y: 0 },
                    Position { x: 27, y: -3 },
                ],
                end_position: Some(Position { x: 28, y: -7 }),
            }
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
            SimulatedShot {
                start_position,
                initial_velocity,
                target_area,
                mid_positions: vec![
                    Position { x: 6, y: 3 },
                    Position { x: 11, y: 5 },
                    Position { x: 15, y: 6 },
                    Position { x: 18, y: 6 },
                    Position { x: 20, y: 5 },
                    Position { x: 21, y: 3 },
                    Position { x: 21, y: 0 },
                    Position { x: 21, y: -4 },
                ],
                end_position: Some(Position { x: 21, y: -9 }),
            }
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
            SimulatedShot {
                start_position,
                initial_velocity,
                target_area,
                mid_positions: vec![
                    Position { x: 9, y: 0 },
                    Position { x: 17, y: -1 },
                    Position { x: 24, y: -3 },
                ],
                end_position: Some(Position { x: 30, y: -6 }),
            }
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
        assert_eq!(
            simulation_result,
            SimulatedShot {
                start_position,
                initial_velocity,
                target_area,
                mid_positions: vec![Position { x: 17, y: -4 },],
                end_position: None,
            }
        );
    }
}
