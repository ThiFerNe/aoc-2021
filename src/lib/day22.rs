use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day22";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 22: Reactor Reboot")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day22-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day22Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day22Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let count_of_on_cubes_after_reboot_steps = count_on_cubes_after_reboot_steps(&file_contents)?;
    println!(
        "The count of on cubes after reboot steps is {}.",
        count_of_on_cubes_after_reboot_steps
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day22Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count on cubes after reboot steps ({0})")]
    CountOnCubesAfterRebootSteps(#[from] CountOnCubesAfterRebootStepsError),
}

pub fn count_on_cubes_after_reboot_steps(
    reboot_steps: &str,
) -> Result<u128, CountOnCubesAfterRebootStepsError> {
    let parsed_reboot_steps = parse_reboot_steps(reboot_steps)?;
    let mut reactor_core = ReactorCore::new();
    for reboot_step in &parsed_reboot_steps {
        reactor_core.perform(reboot_step);
    }
    Ok(reactor_core.count_on() as u128)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountOnCubesAfterRebootStepsError {
    #[error("Could not parse reboot steps ({0})")]
    ParseRebootSteps(#[from] ParseRebootStepsError),
}

struct ReactorCore(Box<[[[CubeStatus; 101]; 101]; 101]>);

impl ReactorCore {
    fn new() -> Self {
        Self(Box::new([[[CubeStatus::Off; 101]; 101]; 101]))
    }

    fn perform(&mut self, reboot_step: &RebootStep) {
        for z in reboot_step.from_z.max(-50)..=reboot_step.to_z.min(50) {
            for y in reboot_step.from_y.max(-50)..=reboot_step.to_y.min(50) {
                for x in reboot_step.from_x.max(-50)..=reboot_step.to_x.min(50) {
                    let x = x + 50;
                    let y = y + 50;
                    let z = z + 50;
                    if (0..=100).contains(&x) && (0..=100).contains(&y) && (0..=100).contains(&z) {
                        self.0[z as usize][y as usize][x as usize] = reboot_step.target_status;
                    }
                }
            }
        }
    }

    fn count_on(&self) -> usize {
        self.0
            .iter()
            .flatten()
            .flatten()
            .filter(|cube_status| matches!(cube_status, CubeStatus::On))
            .count()
    }
}

fn parse_reboot_steps(reboot_steps: &str) -> Result<Vec<RebootStep>, ParseRebootStepsError> {
    reboot_steps
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            RebootStep::from_str(line)
                .map_err(|error| ParseRebootStepsError::RebootStepFromStr(line.to_string(), error))
        })
        .collect::<Result<Vec<_>, ParseRebootStepsError>>()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseRebootStepsError {
    #[error("Could not parse reboot step from string \"{0}\" ({1})")]
    RebootStepFromStr(String, #[source] RebootStepFromStrError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct RebootStep {
    from_x: isize,
    to_x: isize,
    from_y: isize,
    to_y: isize,
    from_z: isize,
    to_z: isize,
    target_status: CubeStatus,
}

impl FromStr for RebootStep {
    type Err = RebootStepFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (target_status, suffix) = if let Some(suffix) = s.strip_prefix("on ") {
            (CubeStatus::On, suffix)
        } else if let Some(suffix) = s.strip_prefix("off ") {
            (CubeStatus::Off, suffix)
        } else {
            return Err(RebootStepFromStrError::WrongPrefix(s.to_string()));
        };
        let parsed_ranges: [[isize; 2]; 3] = suffix
            .split(',')
            .map(|part| {
                part.split_at(2)
                    .1
                    .split("..")
                    .map(|value| {
                        value.parse::<isize>().map_err(|error| {
                            RebootStepFromStrError::ParseInt(value.to_string(), error)
                        })
                    })
                    .collect::<Result<Vec<isize>, RebootStepFromStrError>>()
                    .and_then(|v| {
                        v.try_into().map_err(|v: Vec<isize>| {
                            RebootStepFromStrError::UnexpectedRangeParts(v.len())
                        })
                    })
            })
            .collect::<Result<Vec<[isize; 2]>, RebootStepFromStrError>>()
            .and_then(|v| {
                v.try_into().map_err(|v: Vec<[isize; 2]>| {
                    RebootStepFromStrError::UnexpectedCoordinateParts(v.len())
                })
            })?;
        Ok(Self {
            from_x: parsed_ranges[0][0],
            to_x: parsed_ranges[0][1],
            from_y: parsed_ranges[1][0],
            to_y: parsed_ranges[1][1],
            from_z: parsed_ranges[2][0],
            to_z: parsed_ranges[2][1],
            target_status,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RebootStepFromStrError {
    #[error("Encountered wrong prefix in \"{0}\", expecting on of \"on \" or \"off \"")]
    WrongPrefix(String),
    #[error("Could not parse \"{0}\" ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Unexpected count of range parts of {0}, but expected 2")]
    UnexpectedRangeParts(usize),
    #[error("Unexpected count of coordinate parts of {0}, but expected 3")]
    UnexpectedCoordinateParts(usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CubeStatus {
    On,
    Off,
}

impl Default for CubeStatus {
    fn default() -> Self {
        Self::Off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_on_cubes_after_reboot_steps() {
        // given
        let input = "on x=-20..26,y=-36..17,z=-47..7\r\non x=-20..33,y=-21..23,z=-26..28\r\n\
                            on x=-22..28,y=-29..23,z=-38..16\r\non x=-46..7,y=-6..46,z=-50..-1\r\n\
                            on x=-49..1,y=-3..46,z=-24..28\r\non x=2..47,y=-22..22,z=-23..27\r\n\
                            on x=-27..23,y=-28..26,z=-21..29\r\non x=-39..5,y=-6..47,z=-3..44\r\n\
                            on x=-30..21,y=-8..43,z=-13..34\r\non x=-22..26,y=-27..20,z=-29..19\r\n\
                            off x=-48..-32,y=26..41,z=-47..-37\r\non x=-12..35,y=6..50,z=-50..-2\r\n\
                            off x=-48..-32,y=-32..-16,z=-15..-5\r\non x=-18..26,y=-33..15,z=-7..46\r\n\
                            off x=-40..-22,y=-38..-28,z=23..41\r\non x=-16..35,y=-41..10,z=-47..6\r\n\
                            off x=-32..-23,y=11..30,z=-14..3\r\non x=-49..-5,y=-3..45,z=-29..18\r\n\
                            off x=18..30,y=-20..-8,z=-3..13\r\non x=-41..9,y=-7..43,z=-33..15\r\n\
                            on x=-54112..-39298,y=-85059..-49293,z=-27449..7877\r\n\
                            on x=967..23432,y=45373..81175,z=27513..53682";

        // when
        let count_of_on_cubes_after_reboot_steps = count_on_cubes_after_reboot_steps(input);

        // then
        assert_eq!(count_of_on_cubes_after_reboot_steps, Ok(590784));
    }
}
