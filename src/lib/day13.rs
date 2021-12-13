use std::cmp::Ordering;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day13";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 13: Transparent Origami")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day13-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day13Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day13Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let folded_transparent_paper = fully_fold_transparent_paper(&file_contents)?;
            println!(
                "The fully folded transparent paper looks like:\r\n\r\n{}",
                folded_transparent_paper
            );
        }
        _ => {
            let count_of_dots_visible_after_folding =
                count_dots_visible_after_folding_once(&file_contents)?;
            println!(
                "There are {} dots visible after completing just the first fold instruction.",
                count_of_dots_visible_after_folding
            );
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day13Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count dots visible after folding ({0})")]
    CountDotsVisibleAfterFolding(#[from] CountDotsVisibleAfterFoldingError),
    #[error("Could not fully fold transparent paper ({0})")]
    FullyFoldTransparentPaper(#[from] FullyFoldTransparentPaperError),
}

pub fn count_dots_visible_after_folding_once(
    transparent_paper: &str,
) -> Result<u128, CountDotsVisibleAfterFoldingError> {
    let mut transparent_paper = TransparentPaper::from_str(transparent_paper)?;
    transparent_paper.fold();
    Ok(transparent_paper.marked_dot_positions.len() as u128)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountDotsVisibleAfterFoldingError {
    #[error("Could not parse transparent paper ({0})")]
    TransparentPaperFromStr(#[from] TransparentPaperFromStrError),
}

pub fn fully_fold_transparent_paper(
    transparent_paper: &str,
) -> Result<TransparentPaper, FullyFoldTransparentPaperError> {
    let mut transparent_paper = TransparentPaper::from_str(transparent_paper)?;
    while !transparent_paper.instructions.is_empty() {
        transparent_paper.fold();
    }
    Ok(transparent_paper)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FullyFoldTransparentPaperError {
    #[error("Could not parse transparent paper ({0})")]
    TransparentPaperFromStr(#[from] TransparentPaperFromStrError),
}

#[derive(Debug, Clone)]
pub struct TransparentPaper {
    marked_dot_positions: Vec<Position>,
    size: Size,
    instructions: Vec<FoldInstruction>,
}

impl TransparentPaper {
    fn fold(&mut self) {
        if !self.instructions.is_empty() {
            let instruction = self.instructions.remove(0);
            self.marked_dot_positions = self
                .marked_dot_positions
                .iter()
                .filter_map(|dot| match instruction {
                    FoldInstruction::FoldAlongX(x_fold_index) => match dot.x.cmp(&x_fold_index) {
                        Ordering::Less => Some(*dot),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Position {
                            x: 2 * x_fold_index - dot.x,
                            y: dot.y,
                        }),
                    },
                    FoldInstruction::FoldAlongY(y_fold_index) => match dot.y.cmp(&y_fold_index) {
                        Ordering::Less => Some(*dot),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(Position {
                            x: dot.x,
                            y: 2 * y_fold_index - dot.y,
                        }),
                    },
                })
                .fold(Vec::new(), |mut collections, next| {
                    if !collections.contains(&next) {
                        collections.push(next);
                    }
                    collections
                });
            self.size = Self::calculate_size(&self.marked_dot_positions);
        }
    }

    fn calculate_size(marked_dot_positions: &[Position]) -> Size {
        marked_dot_positions
            .iter()
            .fold(None, |optional_size, position| match optional_size {
                None => Some(Size {
                    width: position.x,
                    height: position.y,
                }),
                Some(size) => Some(Size {
                    width: size.width.max(position.x),
                    height: size.height.max(position.y),
                }),
            })
            .map_or(
                Size {
                    width: 0,
                    height: 0,
                },
                |size| Size {
                    width: size.width + 1,
                    height: size.height + 1,
                },
            )
    }
}

impl Display for TransparentPaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                if self.marked_dot_positions.contains(&Position { x, y }) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}

impl FromStr for TransparentPaper {
    type Err = TransparentPaperFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (points, optional_instructions) = s.lines().fold(
            (Vec::new(), None),
            |(mut points, mut optional_instructions), line| {
                match &mut optional_instructions {
                    None => {
                        if line.trim().is_empty() {
                            optional_instructions = Some(Vec::new());
                        } else {
                            points.push(line);
                        }
                    }
                    Some(instructions) => {
                        if !line.trim().is_empty() {
                            instructions.push(line)
                        }
                    }
                }
                (points, optional_instructions)
            },
        );
        let marked_dot_positions = points
            .into_iter()
            .map(|line| {
                Position::from_str(line).map_err(|error| {
                    TransparentPaperFromStrError::PositionFromStr(line.to_string(), error)
                })
            })
            .collect::<Result<Vec<Position>, TransparentPaperFromStrError>>()?;
        let size = Self::calculate_size(&marked_dot_positions);
        Ok(Self {
            marked_dot_positions,
            size,
            instructions: optional_instructions
                .unwrap_or_default()
                .into_iter()
                .map(|line| {
                    FoldInstruction::from_str(line).map_err(|error| {
                        TransparentPaperFromStrError::FoldInstructionFromStr(
                            line.to_string(),
                            error,
                        )
                    })
                })
                .collect::<Result<Vec<FoldInstruction>, TransparentPaperFromStrError>>()?,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TransparentPaperFromStrError {
    #[error("Could not parse position \"{0}\" ({1})")]
    PositionFromStr(String, #[source] PositionFromStrError),
    #[error("Could not parse fold instruction \"{0}\" ({1})")]
    FoldInstructionFromStr(String, #[source] FoldInstructionFromStrError),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Position {
    x: usize,
    y: usize,
}

impl FromStr for Position {
    type Err = PositionFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let position: [usize; 2] = s
            .split(',')
            .map(|element| {
                element
                    .parse::<usize>()
                    .map_err(|error| PositionFromStrError::ParseInt(element.to_string(), error))
            })
            .collect::<Result<Vec<usize>, PositionFromStrError>>()?
            .try_into()
            .map_err(|v: Vec<usize>| PositionFromStrError::UnexpectedCountOfElements(v.len()))?;
        Ok(Self {
            x: position[0],
            y: position[1],
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PositionFromStrError {
    #[error("Could not parse \"{0}\" ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Did not expect {0} elements in position")]
    UnexpectedCountOfElements(usize),
}

#[derive(Debug, Copy, Clone)]
struct Size {
    width: usize,
    height: usize,
}

#[derive(Debug, Copy, Clone)]
enum FoldInstruction {
    FoldAlongX(usize),
    FoldAlongY(usize),
}

impl FromStr for FoldInstruction {
    type Err = FoldInstructionFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(value) = s.strip_prefix("fold along y=") {
            Ok(Self::FoldAlongY(value.trim().parse::<usize>().map_err(
                |error| FoldInstructionFromStrError::ParseInt(value.to_string(), error),
            )?))
        } else if let Some(value) = s.strip_prefix("fold along x=") {
            Ok(Self::FoldAlongX(value.trim().parse::<usize>().map_err(
                |error| FoldInstructionFromStrError::ParseInt(value.to_string(), error),
            )?))
        } else {
            Err(FoldInstructionFromStrError::PrefixUnknown)
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FoldInstructionFromStrError {
    #[error("Could not parse number \"{0}\" ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Prefix is unknown")]
    PrefixUnknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_dots_visible_after_folding_once_should_return_17() {
        // given
        let input = "6,10\r\n0,14\r\n9,10\r\n0,3\r\n10,4\r\n4,11\r\n6,0\r\n6,12\r\n4,1\r\n\
                            0,13\r\n10,12\r\n3,4\r\n3,0\r\n8,4\r\n1,10\r\n2,14\r\n8,10\r\n9,0\r\n\
                            \r\nfold along y=7\r\nfold along x=5";

        // when
        let dot_count_after_fold = count_dots_visible_after_folding_once(input);

        // then
        assert_eq!(dot_count_after_fold, Ok(17));
    }
}
