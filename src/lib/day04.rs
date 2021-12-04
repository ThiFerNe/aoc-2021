use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day04";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 4: Giant Squid")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day04-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day04Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day04Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let board_selection = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => BoardSelection::Loosing,
        _ => BoardSelection::Winning,
    };
    let scores = calculate_winning_bingo_board_scores(&file_contents, board_selection)?;
    println!("The {} bingo board has {:?}.", board_selection, scores);
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day04Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate winning bingo board scores ({0})")]
    CalculateWinningBingoBoardScores(#[from] CalculateWinningBingoBoardScoresError),
}

pub fn calculate_winning_bingo_board_scores(
    bingo_play_data: &str,
    board_selection: BoardSelection,
) -> Result<Scores, CalculateWinningBingoBoardScoresError> {
    let (drawn_number_strings, bingo_board_strings): (Option<&str>, Vec<Vec<&str>>) =
        bingo_play_data
            .lines()
            .filter(|line| !line.is_empty())
            .fold((None, Vec::new()), |(mut a, mut b), next| {
                if a.is_none() {
                    a = Some(next);
                } else {
                    if b.is_empty() || b[b.len() - 1].len() >= 5 {
                        b.push(Vec::new());
                    }
                    let current_index = b.len() - 1;
                    b[current_index].push(next);
                }
                (a, b)
            });
    let drawn_numbers = drawn_number_strings
        .ok_or(CalculateWinningBingoBoardScoresError::MissingDrawnNumbers)?
        .split(',')
        .map(|value| {
            value.parse::<u8>().map_err(|error| {
                CalculateWinningBingoBoardScoresError::ParseDrawnNumbers(value.to_string(), error)
            })
        })
        .collect::<Result<Vec<u8>, CalculateWinningBingoBoardScoresError>>()?;
    let mut bingo_boards = bingo_board_strings
        .into_iter()
        .map(|bingo_board| bingo_board.join("\r\n"))
        .map(|bingo_board| {
            BingoBoard::from_str(&bingo_board).map_err(|error| {
                CalculateWinningBingoBoardScoresError::BingoBoardFromStr(bingo_board, error)
            })
        })
        .collect::<Result<Vec<BingoBoard>, CalculateWinningBingoBoardScoresError>>()?;

    let mut optional_winning_board = None;
    let mut optional_last_drawn_number = None;
    for drawn_number in drawn_numbers {
        optional_last_drawn_number = Some(drawn_number);
        for bingo_board in &mut bingo_boards {
            bingo_board.mark(drawn_number);
            if board_selection == BoardSelection::Winning && bingo_board.contains_bingo() {
                optional_winning_board = Some(bingo_board.clone());
                break;
            }
        }
        if board_selection == BoardSelection::Loosing {
            if bingo_boards.len() == 1 && bingo_boards[0].contains_bingo() {
                optional_winning_board = Some(bingo_boards[0].clone());
            } else {
                bingo_boards.retain(|bingo_board| !bingo_board.contains_bingo());
            }
        }
        if optional_winning_board.is_some() {
            break;
        }
    }
    match optional_last_drawn_number {
        None => Err(CalculateWinningBingoBoardScoresError::NoNumberHasBeenDrawn),
        Some(last_drawn_number) => match optional_winning_board {
            None => Err(CalculateWinningBingoBoardScoresError::NoBoardWon),
            Some(winning_board) => Ok(Scores::of(
                winning_board
                    .get_unmarked_cell_values()
                    .iter()
                    .map(|v| (*v) as u16)
                    .sum(),
                last_drawn_number,
            )),
        },
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateWinningBingoBoardScoresError {
    #[error("Missing line with drawn numbers")]
    MissingDrawnNumbers,
    #[error("Could not parse drawn number \"{0}\" ({1})")]
    ParseDrawnNumbers(String, #[source] ParseIntError),
    #[error("Could not parse bingo board \"{0}\" ({1})")]
    BingoBoardFromStr(String, #[source] BingoBoardFromStrError),
    #[error("No number has been drawn")]
    NoNumberHasBeenDrawn,
    #[error("No bingo board won")]
    NoBoardWon,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Scores {
    sum_all_unmarked_numbers: u16,
    lastly_called_number: u8,
}

impl Scores {
    fn of(sum_all_unmarked_numbers: u16, lastly_called_number: u8) -> Self {
        Self {
            sum_all_unmarked_numbers,
            lastly_called_number,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct BingoBoard {
    cells: [[u8; 5]; 5],
    marked: [[bool; 5]; 5],
}

impl BingoBoard {
    fn mark(&mut self, number: u8) {
        for y in 0..5 {
            for x in 0..5 {
                if self.cells[y][x] == number {
                    self.marked[y][x] = true;
                }
            }
        }
    }

    fn contains_bingo(&self) -> bool {
        for column in 0..5 {
            if self.marked.iter().all(|line| line[column]) {
                return true;
            }
        }
        self.marked
            .iter()
            .any(|line| line.iter().all(|value| *value))
    }

    fn get_unmarked_cell_values(&self) -> Vec<u8> {
        let mut output = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                if !self.marked[y][x] {
                    output.push(self.cells[y][x]);
                }
            }
        }
        output
    }
}

impl FromStr for BingoBoard {
    type Err = BingoBoardFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            cells: s
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    line.split(' ')
                        .filter(|value| !value.is_empty())
                        .map(|value| {
                            value.parse::<u8>().map_err(|error| {
                                BingoBoardFromStrError::Parse(value.to_string(), error)
                            })
                        })
                        .collect::<Result<Vec<u8>, BingoBoardFromStrError>>()
                })
                .collect::<Result<Vec<Vec<u8>>, BingoBoardFromStrError>>()?
                .into_iter()
                .enumerate()
                .map(|(line_no, line)| {
                    line.try_into().map_err(|line| {
                        BingoBoardFromStrError::LineCountOfElementsNotFive(line_no, line)
                    })
                })
                .collect::<Result<Vec<[u8; 5]>, BingoBoardFromStrError>>()?
                .try_into()
                .map_err(|lines: Vec<[u8; 5]>| {
                    BingoBoardFromStrError::LineCountNotFive(lines.len(), lines)
                })?,
            marked: [[false; 5]; 5],
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BingoBoardFromStrError {
    #[error("Could not parse \"{0}\" to number ({1})")]
    Parse(String, #[source] ParseIntError),
    #[error("Elements count of line no. {0} is not five ({1:?}) ({})")]
    LineCountOfElementsNotFive(usize, Vec<u8>),
    #[error("Count of lines is {0} and not five ({1:?})")]
    LineCountNotFive(usize, Vec<[u8; 5]>),
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum BoardSelection {
    Winning,
    Loosing,
}

impl std::fmt::Display for BoardSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Winning => write!(f, "winning"),
            Self::Loosing => write!(f, "loosing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_of() {
        // given
        let sum_all_unmarked_numbers = rand::random();
        let lastly_called_number = rand::random();

        // when
        let scores = Scores::of(sum_all_unmarked_numbers, lastly_called_number);

        // then
        assert_eq!(scores.sum_all_unmarked_numbers, sum_all_unmarked_numbers);
        assert_eq!(scores.lastly_called_number, lastly_called_number);
    }

    #[test]
    fn bingo_board_of() {
        // given
        let input = "\r\n14 21 17 24  4\r\n10 16 15  9 19\r\n18  8 23 26 20\r\n\
                            22 11 13  6  5\r\n 2  0 12  3  7";

        // when
        let bingo_board = BingoBoard::from_str(input);

        // then
        assert_eq!(
            bingo_board,
            Ok(BingoBoard {
                cells: [
                    [14, 21, 17, 24, 4],
                    [10, 16, 15, 9, 19],
                    [18, 8, 23, 26, 20],
                    [22, 11, 13, 6, 5],
                    [2, 0, 12, 3, 7]
                ],
                marked: [[false; 5]; 5]
            })
        );
    }

    #[test]
    fn calculate_winning_bingo_board_scores_with_winning_should_return_188_24() {
        // given
        let input = "7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1\
                            \r\n\r\n22 13 17 11  0\r\n 8  2 23  4 24\r\n21  9 14 16  7\
                            \r\n 6 10  3 18  5\r\n 1 12 20 15 19\r\n\r\n 3 15  0  2 22\
                            \r\n 9 18 13 17  5\r\n19  8  7 25 23\r\n20 11 10 24  4\r\n\
                            14 21 16 12  6\r\n\r\n14 21 17 24  4\r\n10 16 15  9 19\r\n\
                            18  8 23 26 20\r\n22 11 13  6  5\r\n 2  0 12  3  7";

        // when
        let scores = calculate_winning_bingo_board_scores(input, BoardSelection::Winning);

        // then
        assert_eq!(scores, Ok(Scores::of(188, 24)));
    }

    #[test]
    fn calculate_winning_bingo_board_scores_with_loosing_should_return_148_13() {
        // given
        let input = "7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1\
                            \r\n\r\n22 13 17 11  0\r\n 8  2 23  4 24\r\n21  9 14 16  7\
                            \r\n 6 10  3 18  5\r\n 1 12 20 15 19\r\n\r\n 3 15  0  2 22\
                            \r\n 9 18 13 17  5\r\n19  8  7 25 23\r\n20 11 10 24  4\r\n\
                            14 21 16 12  6\r\n\r\n14 21 17 24  4\r\n10 16 15  9 19\r\n\
                            18  8 23 26 20\r\n22 11 13  6  5\r\n 2  0 12  3  7";

        // when
        let scores = calculate_winning_bingo_board_scores(input, BoardSelection::Loosing);

        // then
        assert_eq!(scores, Ok(Scores::of(148, 13)));
    }
}
