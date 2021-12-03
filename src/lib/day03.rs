use clap::{App, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

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
                .default_value("day03-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day03Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day03Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let power_consumption = extract_power_consumption(&file_contents)?;
    println!("Extracted {:?}.", power_consumption);
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day03Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not extract power consumption ({0})")]
    ExtractPowerConsumption(#[from] ExtractPowerConsumptionError),
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
}
