use std::fmt::Display;
use std::num::ParseIntError;
use std::ops::Add;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day18";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 18: Snailfish")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day18-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day18Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day18Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let largest_magnitude_of_any_addition =
                find_largest_magnitude_of_any_addition(&file_contents)?;
            println!(
                "The largest magnitude of any addition is {}.",
                largest_magnitude_of_any_addition
            );
        }
        _ => {
            let magnitude_of_added_snailfish_numbers =
                find_magnitude_of_added_snailfish_numbers(&file_contents)?;
            println!(
                "The magnitude of added snailfish numbers is {}.",
                magnitude_of_added_snailfish_numbers
            );
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day18Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not find largest magnitude of any addition ({0})")]
    FindLargestMagnitudeOfAnyAddition(#[from] FindLargestMagnitudeOfAnyAdditionError),
    #[error("Could not find magnitude of added snailfish numbers ({0})")]
    FindMagnitudeOfAddedSnailfishNumbers(#[from] FindMagnitudeOfAddedSnailfishNumbersError),
}

pub fn find_largest_magnitude_of_any_addition(
    snailfish_numbers: &str,
) -> Result<u128, FindLargestMagnitudeOfAnyAdditionError> {
    let snailfish_numbers = snailfish_numbers
        .lines()
        .filter(|line| !line.is_empty())
        .map(SnailfishNumber::from_str)
        .collect::<Result<Vec<SnailfishNumber>, SnailfishNumberFromStrError>>()?;
    (0..snailfish_numbers.len())
        .flat_map(|a| {
            (0..snailfish_numbers.len())
                .filter_map(move |b| if a == b { None } else { Some((a, b)) })
        })
        .map(|(a, b)| snailfish_numbers[a].clone() + snailfish_numbers[b].clone())
        .map(|snailfish_number| SnailfishNumber::magnitude(&snailfish_number))
        .max()
        .ok_or(FindLargestMagnitudeOfAnyAdditionError::MissingSnailfishNumberInInput)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindLargestMagnitudeOfAnyAdditionError {
    #[error("Could not parse snailfish number from string ({0})")]
    SnailfishNumberFromStr(#[from] SnailfishNumberFromStrError),
    #[error("There was no snailfish number in input")]
    MissingSnailfishNumberInInput,
}

pub fn find_magnitude_of_added_snailfish_numbers(
    snailfish_numbers: &str,
) -> Result<u128, FindMagnitudeOfAddedSnailfishNumbersError> {
    snailfish_numbers
        .lines()
        .filter(|line| !line.is_empty())
        .map(SnailfishNumber::from_str)
        .collect::<Result<Vec<SnailfishNumber>, SnailfishNumberFromStrError>>()?
        .into_iter()
        .reduce(|a, b| a + b)
        .ok_or(FindMagnitudeOfAddedSnailfishNumbersError::MissingSnailfishNumberInInput)
        .map(|snailfish_number| SnailfishNumber::magnitude(&snailfish_number))
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindMagnitudeOfAddedSnailfishNumbersError {
    #[error("Could not parse snailfish number from string ({0})")]
    SnailfishNumberFromStr(#[from] SnailfishNumberFromStrError),
    #[error("There was no snailfish number in input")]
    MissingSnailfishNumberInInput,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SnailfishNumber(InnerSnailfishNumber);

impl SnailfishNumber {
    fn magnitude(&self) -> u128 {
        self.0.magnitude()
    }
}

impl Add for SnailfishNumber {
    type Output = SnailfishNumber;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for SnailfishNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SnailfishNumber {
    type Err = SnailfishNumberFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = InnerSnailfishNumber::from_str(s)?;
        if matches!(parsed, InnerSnailfishNumber::SnailfishNumber(_, _)) {
            Ok(Self(parsed))
        } else {
            Err(SnailfishNumberFromStrError::ExpectedSnailfishNumberOnTop)
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SnailfishNumberFromStrError {
    #[error(transparent)]
    InnerSnailfishNumberFromStr(#[from] InnerSnailfishNumberFromStrError),
    #[error("Expected snailfish number on top")]
    ExpectedSnailfishNumberOnTop,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum InnerSnailfishNumber {
    SimpleNumber(u8),
    SnailfishNumber(Box<InnerSnailfishNumber>, Box<InnerSnailfishNumber>),
}

impl InnerSnailfishNumber {
    fn explode(&mut self) {
        fn inner_explode(
            inner: &mut InnerSnailfishNumber,
            depth: u128,
        ) -> (Option<u8>, Option<InnerSnailfishNumber>, Option<u8>, bool) {
            fn add_left_most(inner: &mut InnerSnailfishNumber, value: u8) {
                match inner {
                    InnerSnailfishNumber::SimpleNumber(simple_number) => *simple_number += value,
                    InnerSnailfishNumber::SnailfishNumber(left, _) => {
                        add_left_most(left.as_mut(), value)
                    }
                }
            }

            fn add_right_most(inner: &mut InnerSnailfishNumber, value: u8) {
                match inner {
                    InnerSnailfishNumber::SimpleNumber(simple_number) => *simple_number += value,
                    InnerSnailfishNumber::SnailfishNumber(_, right) => {
                        add_right_most(right.as_mut(), value)
                    }
                }
            }

            let double_inner_explode_left_and_right =
                |left: &mut Box<InnerSnailfishNumber>, right: &mut Box<InnerSnailfishNumber>| {
                    let (
                        optional_exploded_left_left,
                        optional_new_left_inner_snailfish_number,
                        optional_exploded_left_right,
                        exploded,
                    ) = inner_explode(left, depth + 1);
                    if let Some(new_left_inner_snailfish_number) =
                        optional_new_left_inner_snailfish_number
                    {
                        *left = Box::new(new_left_inner_snailfish_number);
                    }
                    if let Some(exploded_left_right) = optional_exploded_left_right {
                        add_left_most(right, exploded_left_right);
                    }
                    if exploded {
                        (optional_exploded_left_left, None, None, exploded)
                    } else {
                        let (
                            optional_exploded_right_left,
                            optional_new_right_inner_snailfish_number,
                            optional_exploded_right_right,
                            exploded,
                        ) = inner_explode(right, depth + 1);
                        if let Some(new_right_inner_snailfish_number) =
                            optional_new_right_inner_snailfish_number
                        {
                            *right = Box::new(new_right_inner_snailfish_number);
                        }
                        if let Some(exploded_right_left) = optional_exploded_right_left {
                            add_right_most(left, exploded_right_left);
                        }
                        (None, None, optional_exploded_right_right, exploded)
                    }
                };
            if depth >= 4 {
                if let InnerSnailfishNumber::SnailfishNumber(left, right) = inner {
                    if let InnerSnailfishNumber::SimpleNumber(left_simple_number) = left.as_mut() {
                        if let InnerSnailfishNumber::SimpleNumber(right_simple_number) =
                            right.as_mut()
                        {
                            (
                                Some(*left_simple_number),
                                Some(InnerSnailfishNumber::SimpleNumber(0)),
                                Some(*right_simple_number),
                                true,
                            )
                        } else {
                            let (
                                optional_exploded_right_left,
                                optional_new_right_inner_snailfish_number,
                                optional_exploded_right_right,
                                exploded,
                            ) = inner_explode(right, depth + 1);
                            if let Some(new_right_inner_snailfish_number) =
                                optional_new_right_inner_snailfish_number
                            {
                                *right = Box::new(new_right_inner_snailfish_number);
                            }
                            if let Some(exploded_right_left) = optional_exploded_right_left {
                                *left_simple_number += exploded_right_left;
                            }
                            (None, None, optional_exploded_right_right, exploded)
                        }
                    } else if let InnerSnailfishNumber::SimpleNumber(right_simple_number) =
                        right.as_mut()
                    {
                        let (
                            optional_exploded_left_left,
                            optional_new_left_inner_snailfish_number,
                            optional_exploded_left_right,
                            exploded,
                        ) = inner_explode(left, depth + 1);
                        if let Some(new_left_inner_snailfish_number) =
                            optional_new_left_inner_snailfish_number
                        {
                            *left = Box::new(new_left_inner_snailfish_number);
                        }
                        if let Some(exploded_left_right) = optional_exploded_left_right {
                            *right_simple_number += exploded_left_right;
                        }
                        (optional_exploded_left_left, None, None, exploded)
                    } else {
                        double_inner_explode_left_and_right(left, right)
                    }
                } else {
                    (None, None, None, false)
                }
            } else if let InnerSnailfishNumber::SnailfishNumber(left, right) = inner {
                double_inner_explode_left_and_right(left, right)
            } else {
                (None, None, None, false)
            }
        }

        inner_explode(self, 0);
    }

    fn split(&mut self) {
        fn inner_split(inner: &mut InnerSnailfishNumber) -> bool {
            match inner {
                InnerSnailfishNumber::SimpleNumber(simple_number) => {
                    if *simple_number >= 10 {
                        let half_simple_number = (*simple_number as f64) / 2f64;
                        *inner = InnerSnailfishNumber::SnailfishNumber(
                            Box::new(InnerSnailfishNumber::SimpleNumber(
                                half_simple_number.floor() as u8,
                            )),
                            Box::new(InnerSnailfishNumber::SimpleNumber(
                                half_simple_number.ceil() as u8,
                            )),
                        );
                        true
                    } else {
                        false
                    }
                }
                InnerSnailfishNumber::SnailfishNumber(left, right) => {
                    if inner_split(left.as_mut()) {
                        true
                    } else {
                        inner_split(right.as_mut())
                    }
                }
            }
        }

        inner_split(self);
    }

    fn maximum_depth(&self) -> u128 {
        match self {
            InnerSnailfishNumber::SimpleNumber(_) => 0,
            InnerSnailfishNumber::SnailfishNumber(left, right) => {
                left.maximum_depth().max(right.maximum_depth()) + 1
            }
        }
    }

    fn biggest_simple_number(&self) -> u8 {
        match self {
            InnerSnailfishNumber::SimpleNumber(simple_number) => *simple_number,
            InnerSnailfishNumber::SnailfishNumber(left, right) => left
                .biggest_simple_number()
                .max(right.biggest_simple_number()),
        }
    }

    fn magnitude(&self) -> u128 {
        match self {
            InnerSnailfishNumber::SimpleNumber(simple_number) => *simple_number as u128,
            InnerSnailfishNumber::SnailfishNumber(left, right) => {
                3 * left.magnitude() + 2 * right.magnitude()
            }
        }
    }
}

impl Add for InnerSnailfishNumber {
    type Output = InnerSnailfishNumber;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new = Self::SnailfishNumber(Box::new(self), Box::new(rhs));
        loop {
            if new.maximum_depth() >= 5 {
                new.explode();
            } else if new.biggest_simple_number() >= 10 {
                new.split();
            } else {
                break new;
            }
        }
    }
}

impl Display for InnerSnailfishNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerSnailfishNumber::SimpleNumber(simple_number) => write!(f, "{}", simple_number),
            InnerSnailfishNumber::SnailfishNumber(left, right) => write!(f, "[{},{}]", left, right),
        }
    }
}

impl FromStr for InnerSnailfishNumber {
    type Err = InnerSnailfishNumberFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(char::is_numeric) {
            s.parse::<u8>()
                .map(Self::SimpleNumber)
                .map_err(|error| InnerSnailfishNumberFromStrError::ParseInt(s.to_string(), error))
        } else if s.starts_with('[') {
            let mut opened_brackets = 0;
            let mut optional_middle_index: Option<usize> = None;
            let mut optional_first_part: Option<InnerSnailfishNumber> = None;
            let mut optional_second_part: Option<InnerSnailfishNumber> = None;
            for (index, character) in s.chars().enumerate() {
                if character == '[' {
                    opened_brackets += 1;
                } else if character == ']' {
                    opened_brackets -= 1;
                    if opened_brackets == 0 {
                        if let Some(middle_index) = &optional_middle_index {
                            let second_part = &s[((*middle_index) + 1)..index];
                            optional_second_part = Some(
                                InnerSnailfishNumber::from_str(second_part).map_err(|error| {
                                    InnerSnailfishNumberFromStrError::InnerSnailfishNumberFromStr(
                                        second_part.to_string(),
                                        Box::new(error),
                                    )
                                })?,
                            );
                        } else {
                            return Err(InnerSnailfishNumberFromStrError::SnailfishNumberEndedButNoFirstPartEncountered);
                        }
                    }
                } else if character == ',' && opened_brackets == 1 {
                    optional_middle_index = Some(index);
                    let first_part = &s[1..index];
                    optional_first_part =
                        Some(InnerSnailfishNumber::from_str(first_part).map_err(|error| {
                            InnerSnailfishNumberFromStrError::InnerSnailfishNumberFromStr(
                                first_part.to_string(),
                                Box::new(error),
                            )
                        })?);
                }
            }
            if opened_brackets == 0 {
                Ok(InnerSnailfishNumber::SnailfishNumber(
                    Box::new(
                        optional_first_part
                            .ok_or(InnerSnailfishNumberFromStrError::MissingFirstPart)?,
                    ),
                    Box::new(
                        optional_second_part
                            .ok_or(InnerSnailfishNumberFromStrError::MissingSecondPart)?,
                    ),
                ))
            } else {
                Err(InnerSnailfishNumberFromStrError::MissingClosingBrackets(
                    opened_brackets,
                ))
            }
        } else {
            Err(
                InnerSnailfishNumberFromStrError::UnexpectedStartingCharacter(
                    s.chars().collect::<Vec<char>>().get(0).copied(),
                ),
            )
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum InnerSnailfishNumberFromStrError {
    #[error(
        "Encountered unexpected starting character '{0:?}', expected numeric or opening bracket"
    )]
    UnexpectedStartingCharacter(Option<char>),
    #[error("Could not parse simple number from \"{0}\" ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Missing closing {0} brackets")]
    MissingClosingBrackets(u128),
    #[error("Could not parse sub snailfish number \"{0}\" ({1})")]
    InnerSnailfishNumberFromStr(String, #[source] Box<InnerSnailfishNumberFromStrError>),
    #[error("Snailfish number ended, but no first part has been encountered")]
    SnailfishNumberEndedButNoFirstPartEncountered,
    #[error("Missing first part")]
    MissingFirstPart,
    #[error("Missing second part")]
    MissingSecondPart,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_largest_magnitude_of_any_addition() {
        // given
        let input = "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]\r\n\
                            [[[5,[2,8]],4],[5,[[9,9],0]]]\r\n[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]\r\n\
                            [[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]\r\n\
                            [[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]\r\n\
                            [[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]\r\n\
                            [[[[5,4],[7,7]],8],[[8,3],8]]\r\n[[9,3],[[9,9],[6,[4,9]]]]\r\n\
                            [[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]\r\n\
                            [[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]\r\n";

        // when
        let largest_magnitude_of_any_addition = find_largest_magnitude_of_any_addition(input);

        // then
        assert_eq!(largest_magnitude_of_any_addition, Ok(3993));
    }

    #[test]
    fn test_find_magnitude_of_added_snailfish_numbers() {
        // given
        let input = "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]\r\n\
                            [[[5,[2,8]],4],[5,[[9,9],0]]]\r\n[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]\r\n\
                            [[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]\r\n\
                            [[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]\r\n\
                            [[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]\r\n\
                            [[[[5,4],[7,7]],8],[[8,3],8]]\r\n[[9,3],[[9,9],[6,[4,9]]]]\r\n\
                            [[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]\r\n\
                            [[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]\r\n";

        // when
        let magnitude_of_added_snailfish_numbers = find_magnitude_of_added_snailfish_numbers(input);

        // then
        assert_eq!(magnitude_of_added_snailfish_numbers, Ok(4140));
    }

    #[test]
    fn test_inner_snailfish_number_add() {
        // given
        let input_1 =
            SnailfishNumber::from_str("[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]").unwrap();
        let input_2 = SnailfishNumber::from_str("[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]").unwrap();
        let input_3 =
            SnailfishNumber::from_str("[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]").unwrap();
        let input_4 =
            SnailfishNumber::from_str("[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]")
                .unwrap();
        let input_5 = SnailfishNumber::from_str("[7,[5,[[3,8],[1,4]]]]").unwrap();
        let input_6 = SnailfishNumber::from_str("[[2,[2,2]],[8,[8,1]]]").unwrap();
        let input_7 = SnailfishNumber::from_str("[2,9]").unwrap();
        let input_8 = SnailfishNumber::from_str("[1,[[[9,3],9],[[9,0],[0,7]]]]").unwrap();
        let input_9 = SnailfishNumber::from_str("[[[5,[7,4]],7],1]").unwrap();
        let input_10 = SnailfishNumber::from_str("[[[[4,2],2],6],[8,7]]").unwrap();

        // when
        let output_1 = input_1 + input_2;
        let output_2 = output_1.clone() + input_3;
        let output_3 = output_2.clone() + input_4;
        let output_4 = output_3.clone() + input_5;
        let output_5 = output_4.clone() + input_6;
        let output_6 = output_5.clone() + input_7;
        let output_7 = output_6.clone() + input_8;
        let output_8 = output_7.clone() + input_9;
        let output_9 = output_8.clone() + input_10;

        // then
        assert_eq!(
            output_1,
            SnailfishNumber::from_str("[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]")
                .unwrap()
        );
        assert_eq!(
            output_2,
            SnailfishNumber::from_str(
                "[[[[6,7],[6,7]],[[7,7],[0,7]]],[[[8,7],[7,7]],[[8,8],[8,0]]]]"
            )
            .unwrap()
        );
        assert_eq!(
            output_3,
            SnailfishNumber::from_str(
                "[[[[7,0],[7,7]],[[7,7],[7,8]]],[[[7,7],[8,8]],[[7,7],[8,7]]]]"
            )
            .unwrap()
        );
        assert_eq!(
            output_4,
            SnailfishNumber::from_str(
                "[[[[7,7],[7,8]],[[9,5],[8,7]]],[[[6,8],[0,8]],[[9,9],[9,0]]]]"
            )
            .unwrap()
        );
        assert_eq!(
            output_5,
            SnailfishNumber::from_str("[[[[6,6],[6,6]],[[6,0],[6,7]]],[[[7,7],[8,9]],[8,[8,1]]]]")
                .unwrap()
        );
        assert_eq!(
            output_6,
            SnailfishNumber::from_str("[[[[6,6],[7,7]],[[0,7],[7,7]]],[[[5,5],[5,6]],9]]").unwrap()
        );
        assert_eq!(
            output_7,
            SnailfishNumber::from_str(
                "[[[[7,8],[6,7]],[[6,8],[0,8]]],[[[7,7],[5,0]],[[5,5],[5,6]]]]"
            )
            .unwrap()
        );
        assert_eq!(
            output_8,
            SnailfishNumber::from_str("[[[[7,7],[7,7]],[[8,7],[8,7]]],[[[7,0],[7,7]],9]]").unwrap()
        );
        assert_eq!(
            output_9,
            SnailfishNumber::from_str("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]")
                .unwrap()
        );
    }

    #[test]
    fn inner_snailfish_number_explode() {
        // given
        let mut input_1 = InnerSnailfishNumber::from_str("[[[[[9,8],1],2],3],4]").unwrap();
        let mut input_2 = InnerSnailfishNumber::from_str("[7,[6,[5,[4,[3,2]]]]]").unwrap();
        let mut input_3 = InnerSnailfishNumber::from_str("[[6,[5,[4,[3,2]]]],1]").unwrap();
        let mut input_4 =
            InnerSnailfishNumber::from_str("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]").unwrap();
        let mut input_5 =
            InnerSnailfishNumber::from_str("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]").unwrap();

        // when
        input_1.explode();
        input_2.explode();
        input_3.explode();
        input_4.explode();
        input_5.explode();

        // then
        assert_eq!(
            input_1,
            InnerSnailfishNumber::from_str("[[[[0,9],2],3],4]").unwrap()
        );
        assert_eq!(
            input_2,
            InnerSnailfishNumber::from_str("[7,[6,[5,[7,0]]]]").unwrap()
        );
        assert_eq!(
            input_3,
            InnerSnailfishNumber::from_str("[[6,[5,[7,0]]],3]").unwrap()
        );
        assert_eq!(
            input_4,
            InnerSnailfishNumber::from_str("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]").unwrap()
        );
        assert_eq!(
            input_5,
            InnerSnailfishNumber::from_str("[[3,[2,[8,0]]],[9,[5,[7,0]]]]").unwrap()
        );
    }

    #[test]
    fn inner_snailfish_number_split() {
        // given
        let mut input_1 = InnerSnailfishNumber::from_str("10").unwrap();
        let mut input_2 = InnerSnailfishNumber::from_str("11").unwrap();
        let mut input_3 = InnerSnailfishNumber::from_str("12").unwrap();
        let mut input_4 =
            InnerSnailfishNumber::from_str("[[[[0,7],4],[15,[0,13]]],[1,1]]").unwrap();
        let mut input_5 =
            InnerSnailfishNumber::from_str("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]").unwrap();

        // when
        input_1.split();
        input_2.split();
        input_3.split();
        input_4.split();
        input_5.split();

        // then
        assert_eq!(input_1, InnerSnailfishNumber::from_str("[5,5]").unwrap());
        assert_eq!(input_2, InnerSnailfishNumber::from_str("[5,6]").unwrap());
        assert_eq!(input_3, InnerSnailfishNumber::from_str("[6,6]").unwrap());
        assert_eq!(
            input_4,
            InnerSnailfishNumber::from_str("[[[[0,7],4],[[7,8],[0,13]]],[1,1]]").unwrap()
        );
        assert_eq!(
            input_5,
            InnerSnailfishNumber::from_str("[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]").unwrap()
        );
    }
}
