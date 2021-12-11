use clap::{App, Arg, ArgMatches, SubCommand};
use std::str::FromStr;

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day11";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 11: Dumbo Octopus")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day11-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day11Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day11Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let total_flashes_after_100_steps = calculate_total_flashes_after_100_steps(&file_contents)?;
    println!(
        "There were {} total flashes after 100 steps.",
        total_flashes_after_100_steps
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day11Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate total flashes after 100 steps ({0})")]
    CalculateTotalFlashesAfter100Steps(#[from] CalculateTotalFlashesAfter100StepsError),
}

pub fn calculate_total_flashes_after_100_steps(
    octopus_grid: &str,
) -> Result<u128, CalculateTotalFlashesAfter100StepsError> {
    let octopus_grid = OctopusGrid::from_str(octopus_grid)?;
    Ok((0..100)
        .fold((octopus_grid, 0), |(octopus_grid, flash_count), _| {
            let (new_octopus_grid, additional_flashes) = simulate_step(octopus_grid);
            (new_octopus_grid, flash_count + additional_flashes)
        })
        .1)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateTotalFlashesAfter100StepsError {
    #[error("Could not parse octopus grid ({0})")]
    OctopusGridFromStr(#[from] OctopusGridFromStrError),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct OctopusGrid([[Octopus; 10]; 10]);

impl FromStr for OctopusGrid {
    type Err = OctopusGridFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.lines()
                .enumerate()
                .map(|(lines_index, line)| {
                    line.chars()
                        .map(|character| match character {
                            '0' => Ok(Octopus { energy_level: 0 }),
                            '1' => Ok(Octopus { energy_level: 1 }),
                            '2' => Ok(Octopus { energy_level: 2 }),
                            '3' => Ok(Octopus { energy_level: 3 }),
                            '4' => Ok(Octopus { energy_level: 4 }),
                            '5' => Ok(Octopus { energy_level: 5 }),
                            '6' => Ok(Octopus { energy_level: 6 }),
                            '7' => Ok(Octopus { energy_level: 7 }),
                            '8' => Ok(Octopus { energy_level: 8 }),
                            '9' => Ok(Octopus { energy_level: 9 }),
                            _ => Err(OctopusGridFromStrError::InvalidCharacter(character)),
                        })
                        .collect::<Result<Vec<Octopus>, OctopusGridFromStrError>>()
                        .and_then(|line_elements| {
                            line_elements
                                .try_into()
                                .map_err(|line_elements_err: Vec<Octopus>| {
                                    OctopusGridFromStrError::InvalidLineLength(
                                        lines_index,
                                        line_elements_err.len(),
                                    )
                                })
                        })
                })
                .collect::<Result<Vec<[Octopus; 10]>, OctopusGridFromStrError>>()?
                .try_into()
                .map_err(|lines_element_err: Vec<[Octopus; 10]>| {
                    OctopusGridFromStrError::InvalidLineCount(lines_element_err.len())
                })?,
        ))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
// allowing clippy::enum_variant_names because i don't know how to name them otherwise
#[allow(clippy::enum_variant_names)]
pub enum OctopusGridFromStrError {
    #[error("Expected to find number, but found '{0}'")]
    InvalidCharacter(char),
    #[error("Line no. {0} has {1} elements, but 10 were expected")]
    InvalidLineLength(usize, usize),
    #[error("Found {0} lines, but 10 were expected")]
    InvalidLineCount(usize),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Octopus {
    energy_level: u8,
}

fn simulate_step(mut octopus_grid: OctopusGrid) -> (OctopusGrid, u128) {
    let mut flash_queue = (0..10)
        .into_iter()
        .flat_map(|y| (0..10).into_iter().map(move |x| (x, y)))
        .filter_map(|(x, y)| {
            octopus_grid.0[y][x].energy_level += 1;
            if octopus_grid.0[y][x].energy_level > 9 {
                Some((x, y))
            } else {
                None
            }
        })
        .collect::<Vec<(usize, usize)>>();
    while !flash_queue.is_empty() {
        let (x, y) = flash_queue.remove(0);
        let (x, y) = (x as isize, y as isize);

        let mut increase_neighbour = |x: isize, y: isize| {
            if y >= 0
                && y < octopus_grid.0.len() as isize
                && x >= 0
                && x < octopus_grid.0[y as usize].len() as isize
            {
                let (x, y) = (x as usize, y as usize);
                octopus_grid.0[y][x].energy_level += 1;
                if octopus_grid.0[y][x].energy_level == 10 {
                    flash_queue.push((x, y));
                }
            }
        };

        for iy in (y - 1)..=(y + 1) {
            for ix in (x - 1)..=(x + 1) {
                if !(iy == y && ix == x) {
                    increase_neighbour(ix, iy);
                }
            }
        }
    }
    let flash_counter = (0..10)
        .into_iter()
        .flat_map(|y| (0..10).into_iter().map(move |x| (x, y)))
        .map(|(x, y)| {
            if octopus_grid.0[y][x].energy_level > 9 {
                octopus_grid.0[y][x].energy_level = 0;
                1
            } else {
                0
            }
        })
        .sum();
    (octopus_grid, flash_counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_total_flashes_after_100_steps_should_return_1656() {
        // given
        let input = "5483143223\r\n2745854711\r\n5264556173\r\n6141336146\r\n6357385478\r\n4167524645\r\n2176841721\r\n6882881134\r\n4846848554\r\n5283751526";

        // when
        let total_flashes_after_100_steps = calculate_total_flashes_after_100_steps(input);

        // then
        assert_eq!(total_flashes_after_100_steps, Ok(1656));
    }

    #[test]
    fn test_simulate_step() {
        // given
        let before_any_step = "5483143223\r\n2745854711\r\n5264556173\r\n6141336146\r\n\
                                        6357385478\r\n4167524645\r\n2176841721\r\n6882881134\r\n\
                                        4846848554\r\n5283751526";
        let before_any_step = OctopusGrid::from_str(before_any_step).unwrap();

        // when after step 1
        let (after_step_1, after_step_1_flash_count) = simulate_step(before_any_step);

        // then after step 1
        let expected_after_step_1 = "6594254334\r\n3856965822\r\n6375667284\r\n7252447257\r\n\
                                            7468496589\r\n5278635756\r\n3287952832\r\n\
                                            7993992245\r\n5957959665\r\n6394862637";
        let expected_after_step_1 = OctopusGrid::from_str(expected_after_step_1).unwrap();
        assert_eq!(after_step_1, expected_after_step_1);
        assert_eq!(after_step_1_flash_count, 0);

        // when after step 2
        let (after_step_2, after_step_2_flash_count) = simulate_step(after_step_1);

        // then after step 2
        let expected_after_step_2 = "8807476555\r\n5089087054\r\n8597889608\r\n8485769600\r\n\
                                            8700908800\r\n6600088989\r\n6800005943\r\n\
                                            0000007456\r\n9000000876\r\n8700006848";
        let expected_after_step_2 = OctopusGrid::from_str(expected_after_step_2).unwrap();
        assert_eq!(after_step_2, expected_after_step_2);
        assert_eq!(after_step_2_flash_count, 35);

        // when after step 3
        let (after_step_3, after_step_3_flash_count) = simulate_step(after_step_2);

        // then after step 2
        let expected_after_step_3 = "0050900866\r\n8500800575\r\n9900000039\r\n9700000041\r\n\
                                            9935080063\r\n7712300000\r\n7911250009\r\n\
                                            2211130000\r\n0421125000\r\n0021119000";
        let expected_after_step_3 = OctopusGrid::from_str(expected_after_step_3).unwrap();
        assert_eq!(after_step_3, expected_after_step_3);
        assert_eq!(after_step_3_flash_count, 45);
    }
}
