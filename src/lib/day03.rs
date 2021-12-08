use std::collections::HashMap;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day03";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 3: Binary Diagnostic")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day03-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day03Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day03Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let life_support_rating = extract_life_support_rating(&file_contents)?;
            println!("Extracted {:?}.", life_support_rating);
        }
        _ => {
            let power_consumption = extract_power_consumption(&file_contents)?;
            println!("Extracted {:?}.", power_consumption);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day03Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not extract power consumption ({0})")]
    ExtractPowerConsumption(#[from] ExtractPowerConsumptionError),
    #[error("Could not extract life support rating ({0})")]
    ExtractLifeSupportRating(#[from] ExtractLifeSupportRatingError),
}

pub fn extract_power_consumption(
    diagnostic_report: &str,
) -> Result<PowerConsumption, ExtractPowerConsumptionError> {
    let buckets: HashMap<usize, (u128, u128)> = diagnostic_report
        .split(|c| c == '\r' || c == '\n')
        .filter(|line| !line.is_empty())
        .fold(HashMap::new(), |mut buckets, next| {
            next.chars().enumerate().for_each(|(index, c)| {
                if c == '0' {
                    buckets
                        .entry(index)
                        .and_modify(|count| count.0 += 1)
                        .or_insert((1, 0));
                } else if c == '1' {
                    buckets
                        .entry(index)
                        .and_modify(|count| count.1 += 1)
                        .or_insert((0, 1));
                }
            });
            buckets
        });
    if buckets.is_empty() {
        Err(ExtractPowerConsumptionError::EveryLineEmpty)
    } else {
        let mut gamma_rate = 0u16;
        let mut epsilon_rate = 0u16;
        let max_index = *buckets.keys().max().unwrap();
        for index in 0..=max_index {
            let counts = buckets.get(&index).unwrap_or(&(0, 0));
            if counts.0 == counts.1 {
                eprintln!("Warning: index {} has equal 0s and 1s", index);
            }
            gamma_rate <<= 1;
            epsilon_rate <<= 1;
            if counts.0 <= counts.1 {
                // bit for gamma is 1 and for epsilon is 0
                gamma_rate |= 1;
            } else {
                // bit for gamma is 0 and for epsilon is 1
                epsilon_rate |= 1;
            }
        }
        Ok(PowerConsumption::of(gamma_rate, epsilon_rate))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExtractPowerConsumptionError {
    #[error("Every line is empty")]
    EveryLineEmpty,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PowerConsumption {
    gamma_rate: u16,
    epsilon_rate: u16,
}

impl PowerConsumption {
    fn of(gamma_rate: u16, epsilon_rate: u16) -> Self {
        Self {
            gamma_rate,
            epsilon_rate,
        }
    }
}

pub fn extract_life_support_rating(
    diagnostic_report: &str,
) -> Result<LifeSupportRating, ExtractLifeSupportRatingError> {
    let oxgen_generator_rating = extract_rating(diagnostic_report, |current_bit_index, count| {
        move |line| {
            if count.0 > count.1 {
                line.chars().nth(current_bit_index).unwrap() == '0'
            } else {
                line.chars().nth(current_bit_index).unwrap() == '1'
            }
        }
    })
    .map_err(ExtractLifeSupportRatingError::ExtractOxygenGeneratorRating)?;
    let co2_scrubber_rating = extract_rating(diagnostic_report, |current_bit_index, count| {
        move |line| {
            if count.0 > count.1 {
                line.chars().nth(current_bit_index).unwrap() == '1'
            } else {
                line.chars().nth(current_bit_index).unwrap() == '0'
            }
        }
    })
    .map_err(ExtractLifeSupportRatingError::ExtractCo2ScrubberRating)?;
    Ok(LifeSupportRating::of(
        oxgen_generator_rating,
        co2_scrubber_rating,
    ))
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExtractLifeSupportRatingError {
    #[error("Extracting oxygen rating failed ({0})")]
    ExtractOxygenGeneratorRating(#[source] ExtractRatingError),
    #[error("Extracting CO2 scrubber rating failed ({0})")]
    ExtractCo2ScrubberRating(#[source] ExtractRatingError),
}

fn extract_rating<F: Fn(usize, (u16, u16)) -> G, G: Fn(&&str) -> bool>(
    diagnostic_report: &str,
    filter: F,
) -> Result<u16, ExtractRatingError> {
    let mut lines = diagnostic_report
        .split(|c| c == '\r' || c == '\n')
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>();
    let mut current_bit_index = 0;

    loop {
        let count = lines
            .iter()
            .map(|line| {
                line.chars()
                    .nth(current_bit_index)
                    .ok_or(ExtractRatingError::LineMissingNthChar(current_bit_index))
                    .and_then(|c| match c {
                        '0' => Ok((1, 0)),
                        '1' => Ok((0, 1)),
                        c => Err(ExtractRatingError::CharNotZeroOrOne(c)),
                    })
            })
            .collect::<Result<Vec<(u16, u16)>, ExtractRatingError>>()?
            .into_iter()
            .fold((0u16, 0u16), |count, next| {
                (count.0 + next.0, count.1 + next.1)
            });
        let new_lines = lines
            .into_iter()
            .filter(filter(current_bit_index, count))
            .collect::<Vec<&str>>();

        match new_lines.len() {
            0 => {
                break Err(ExtractRatingError::IterationLeftZeroElements(
                    current_bit_index,
                ));
            }
            1 => {
                break Ok(new_lines[0].chars().fold(0u16, |mut value, next| {
                    value <<= 1;
                    if next == '1' {
                        value |= 1;
                    }
                    value
                }));
            }
            _ => {
                lines = new_lines;
                current_bit_index += 1;
            }
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExtractRatingError {
    #[error("Line is missing char no. {0}")]
    LineMissingNthChar(usize),
    #[error("Char \"{0}\" is neither 0 nor 1")]
    CharNotZeroOrOne(char),
    #[error("Last iteration over bit index {0} left zero elements")]
    IterationLeftZeroElements(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub struct LifeSupportRating {
    oxygen_generator_rating: u16,
    co2_scrubber_rating: u16,
}

impl LifeSupportRating {
    fn of(oxygen_generator_rating: u16, co2_scrubber_rating: u16) -> Self {
        Self {
            oxygen_generator_rating,
            co2_scrubber_rating,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_consumption_of() {
        // when
        let power_consumption = PowerConsumption::of(10, 23);

        // then
        assert_eq!(power_consumption.gamma_rate, 10);
        assert_eq!(power_consumption.epsilon_rate, 23);
    }

    #[test]
    fn extract_power_consumption_should_return_0_1() {
        // given
        let input = "000\r\n000\r\n000";

        // when
        let power_consumption = extract_power_consumption(input);

        // then
        assert_eq!(power_consumption, Ok(PowerConsumption::of(0, 7)));
    }

    #[test]
    fn extract_power_consumption_should_return_22_9() {
        // given
        let input = "00100\r\n11110\r\n10110\r\n10111\r\n10101\r\n01111\r\n00111\r\n11100\r\n10000\r\n11001\r\n00010\r\n01010";

        // when
        let power_consumption = extract_power_consumption(input);

        // then
        assert_eq!(power_consumption, Ok(PowerConsumption::of(22, 9)));
    }

    #[test]
    fn life_support_rating_of() {
        // given
        let oxygen_generator_rating = rand::random();
        let co2_scrubber_rating = rand::random();

        // when
        let life_support_rating =
            LifeSupportRating::of(oxygen_generator_rating, co2_scrubber_rating);

        // then
        assert_eq!(
            life_support_rating.oxygen_generator_rating,
            oxygen_generator_rating
        );
        assert_eq!(life_support_rating.co2_scrubber_rating, co2_scrubber_rating);
    }

    #[test]
    fn extract_life_support_rating_should_return_2_5() {
        // given
        let input = "000\r\n010\r\n101";

        // when
        let life_support_rating = extract_life_support_rating(&input);

        // then
        assert_eq!(life_support_rating, Ok(LifeSupportRating::of(2, 5)));
    }

    #[test]
    fn extract_life_support_rating_should_return_23_10() {
        // given
        let input = "00100\r\n11110\r\n10110\r\n10111\r\n10101\r\n01111\r\n00111\r\n11100\r\n10000\r\n11001\r\n00010\r\n01010";

        // when
        let life_support_rating = extract_life_support_rating(&input);

        // then
        assert_eq!(life_support_rating, Ok(LifeSupportRating::of(23, 10)));
    }
}
