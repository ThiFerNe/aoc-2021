use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day05";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 5: Hydrothermal Venture")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day05-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day05Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day05Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let consider_diagonal_lines = matches!(
        matches.value_of("puzzle_part").unwrap_or("two"),
        "two" | "2"
    );
    let count =
        calculate_count_of_line_overlapping_points(&file_contents, consider_diagonal_lines)?;
    println!("At {} points do at least two lines overlap.", count);
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day05Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate count of line overlapping points ({0})")]
    CalculateCountOfLineOverlappingPoints(#[from] CalculateCountOfLineOverlappingPointsError),
}

pub fn calculate_count_of_line_overlapping_points(
    vent_lines_list: &str,
    consider_diagonal_lines: bool,
) -> Result<usize, CalculateCountOfLineOverlappingPointsError> {
    Ok(parse_vent_lines(vent_lines_list)?
        .into_iter()
        .fold(Ok(HashMap::new()), |optional_field, line| {
            optional_field.and_then(|field| draw_vent_line(field, line, consider_diagonal_lines))
        })?
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .count())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateCountOfLineOverlappingPointsError {
    #[error(transparent)]
    ParseVentLines(#[from] ParseVentLinesError),
    #[error(transparent)]
    DrawVentLine(#[from] DrawVentLineError),
}

fn parse_vent_lines(vent_lines_list: &str) -> Result<Vec<VentLine>, ParseVentLinesError> {
    vent_lines_list
        .lines()
        .map(|line| {
            VentLine::from_str(line)
                .map_err(|error| ParseVentLinesError::VentLineFromStr(line.to_string(), error))
        })
        .collect::<Result<Vec<VentLine>, ParseVentLinesError>>()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseVentLinesError {
    #[error("Could not parse vent line \"{0}\" ({1})")]
    VentLineFromStr(String, #[source] LineFromStrError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct VentLine {
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
}

impl VentLine {
    fn is_horizontal(&self) -> bool {
        self.y1 == self.y2
    }

    fn is_vertical(&self) -> bool {
        self.x1 == self.x2
    }
}

impl FromStr for VentLine {
    type Err = LineFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: [&str; 5] = s
            .split(|c| c == ',' || c == ' ')
            .collect::<Vec<&str>>()
            .try_into()
            .map_err(|_| LineFromStrError::WrongFormat(s.to_string()))?;
        Ok(Self {
            x1: values[0]
                .parse::<u16>()
                .map_err(|error| LineFromStrError::Parse(values[0].to_string(), error))?,
            y1: values[1]
                .parse::<u16>()
                .map_err(|error| LineFromStrError::Parse(values[1].to_string(), error))?,
            x2: values[3]
                .parse::<u16>()
                .map_err(|error| LineFromStrError::Parse(values[3].to_string(), error))?,
            y2: values[4]
                .parse::<u16>()
                .map_err(|error| LineFromStrError::Parse(values[4].to_string(), error))?,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum LineFromStrError {
    #[error("Line \"{0}\" has wrong format, needs \"X1,Y1 -> X2,Y2\"")]
    WrongFormat(String),
    #[error("Could not parse \"{0}\" ({1})")]
    Parse(String, ParseIntError),
}

fn draw_vent_line(
    mut field: HashMap<(u16, u16), u128>,
    line: VentLine,
    consider_diagonal_lines: bool,
) -> Result<HashMap<(u16, u16), u128>, DrawVentLineError> {
    let max_x = line.x1.max(line.x2);
    let min_x = line.x1.min(line.x2);
    let max_y = line.y1.max(line.y2);
    let min_y = line.y1.min(line.y2);
    if line.is_horizontal() {
        for x in min_x..=max_x {
            field
                .entry((x, line.y1))
                .and_modify(|cell| *cell += 1)
                .or_insert(1u128);
        }
    } else if line.is_vertical() {
        for y in min_y..=max_y {
            field
                .entry((line.x1, y))
                .and_modify(|cell| *cell += 1)
                .or_insert(1u128);
        }
    } else if consider_diagonal_lines {
        let width = max_x - min_x;
        let height = max_y - min_y;
        if width == height {
            let mut x = line.x1;
            let mut y = line.y1;
            field
                .entry((x, y))
                .and_modify(|cell| *cell += 1)
                .or_insert(1u128);
            while x != line.x2 && y != line.y2 {
                if line.x2 < line.x1 {
                    x -= 1;
                } else {
                    x += 1;
                }
                if line.y2 < line.y1 {
                    y -= 1;
                } else {
                    y += 1;
                }
                field
                    .entry((x, y))
                    .and_modify(|cell| *cell += 1)
                    .or_insert(1u128);
            }
        } else {
            return Err(DrawVentLineError::InvalidDiagonalLineFound(line));
        }
    } else {
        println!("INFO: ignoring non-vertical/-horizontal {:?}", line);
    }
    Ok(field)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DrawVentLineError {
    #[error("Found invalid diagonal {0:?}")]
    InvalidDiagonalLineFound(VentLine),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_try_from_str() {
        // given
        let input = "0,9 -> 5,9";

        // given
        let line = VentLine::from_str(input);

        // then
        assert_eq!(
            line,
            Ok(VentLine {
                x1: 0,
                y1: 9,
                x2: 5,
                y2: 9
            })
        );
    }

    #[test]
    fn calculate_count_of_line_overlapping_points_should_return_5() {
        // given
        let input = "0,9 -> 5,9\r\n8,0 -> 0,8\r\n9,4 -> 3,4\r\n2,2 -> 2,1\r\n7,0 -> 7,4\r\n\
                            6,4 -> 2,0\r\n0,9 -> 2,9\r\n3,4 -> 1,4\r\n0,0 -> 8,8\r\n5,5 -> 8,2";

        // when
        let count_of_line_overlapping_points =
            calculate_count_of_line_overlapping_points(input, false);

        // then
        assert_eq!(count_of_line_overlapping_points, Ok(5));
    }

    #[test]
    fn calculate_count_of_line_overlapping_points_should_return_12() {
        // given
        let input = "0,9 -> 5,9\r\n8,0 -> 0,8\r\n9,4 -> 3,4\r\n2,2 -> 2,1\r\n7,0 -> 7,4\r\n\
                            6,4 -> 2,0\r\n0,9 -> 2,9\r\n3,4 -> 1,4\r\n0,0 -> 8,8\r\n5,5 -> 8,2";

        // when
        let count_of_line_overlapping_points =
            calculate_count_of_line_overlapping_points(input, true);

        // then
        assert_eq!(count_of_line_overlapping_points, Ok(12));
    }
}
