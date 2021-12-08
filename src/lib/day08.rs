use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day08";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 8: Seven Segment Search")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("day08-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day08Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day08Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let signals = decode_mixed_up_signals(&file_contents)?;
    println!(
        "The digits 1, 4, 7, 8 appear {} times.",
        signals.iter().map(Signal::count_decoded).sum::<usize>()
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day08Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not decode mixed up signals ({0})")]
    DecodeMixedUpSignals(#[from] DecodeMixedUpSignalsError),
}

pub fn decode_mixed_up_signals(
    signals_with_notes: &str,
) -> Result<Vec<Signal>, DecodeMixedUpSignalsError> {
    fn extract_entries<const C: usize>(
        element_entries: &str,
    ) -> Result<[&str; C], DecodeMixedUpSignalsError> {
        element_entries
            .split(' ')
            .filter(|entry| !entry.is_empty())
            .collect::<Vec<&str>>()
            .try_into()
            .map_err(|vec: Vec<&str>| {
                DecodeMixedUpSignalsError::ElementHasUnexpectedCountOfEntries(
                    element_entries.to_string(),
                    vec.len(),
                )
            })
    }

    let lines = signals_with_notes
        .lines()
        .map(|line| {
            line.split('|')
                .collect::<Vec<&str>>()
                .try_into()
                .map_err(|vec: Vec<&str>| {
                    DecodeMixedUpSignalsError::LineHasUnexpectedCountOfVerticalBars(
                        line.to_string(),
                        vec.len(),
                    )
                })
                .map(|elements: [&str; 2]| {
                    extract_entries(elements[0])
                        .and_then(|a| extract_entries(elements[1]).map(|b| (a, b)))
                })
        })
        .collect::<Result<
            Vec<Result<([&str; 10], [&str; 4]), DecodeMixedUpSignalsError>>,
            DecodeMixedUpSignalsError,
        >>()?
        .into_iter()
        .collect::<Result<Vec<([&str; 10], [&str; 4])>, DecodeMixedUpSignalsError>>()?;

    lines
        .into_iter()
        .map(|line| {
            line.1
                .iter()
                .map(|entry| match entry.len() {
                    2 => Ok(SignalNumber::Decoded(1)),
                    3 => Ok(SignalNumber::Decoded(7)),
                    4 => Ok(SignalNumber::Decoded(4)),
                    5 => Ok(SignalNumber::Coded(entry)), // 2 & 3 & 5
                    6 => Ok(SignalNumber::Coded(entry)), // 0 & 6 & 9
                    7 => Ok(SignalNumber::Decoded(8)),
                    a => Err(DecodeMixedUpSignalsError::EntryHasUnexpectedLength(
                        entry.to_string(),
                        a,
                    )),
                })
                .collect::<Result<Vec<SignalNumber>, DecodeMixedUpSignalsError>>()
                .map(|signal_numbers| Signal {
                    first: signal_numbers[0],
                    second: signal_numbers[1],
                    third: signal_numbers[2],
                    fourth: signal_numbers[3],
                })
        })
        .collect::<Result<Vec<Signal>, DecodeMixedUpSignalsError>>()
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Signal<'a> {
    first: SignalNumber<'a>,
    second: SignalNumber<'a>,
    third: SignalNumber<'a>,
    fourth: SignalNumber<'a>,
}

impl<'a> Signal<'a> {
    pub fn count_decoded(&self) -> usize {
        let mut output = 0;
        if matches!(self.first, SignalNumber::Decoded(_)) {
            output += 1;
        }
        if matches!(self.second, SignalNumber::Decoded(_)) {
            output += 1;
        }
        if matches!(self.third, SignalNumber::Decoded(_)) {
            output += 1;
        }
        if matches!(self.fourth, SignalNumber::Decoded(_)) {
            output += 1;
        }
        output
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum SignalNumber<'a> {
    Decoded(u8),
    Coded(&'a str),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DecodeMixedUpSignalsError {
    #[error("Line \"{0}\" has unexpected count of vertical bars of {1} (expected 2)")]
    LineHasUnexpectedCountOfVerticalBars(String, usize),
    #[error("Line element \"{0}\" has unexpected count of entries of {1} (expected 10)")]
    ElementHasUnexpectedCountOfEntries(String, usize),
    #[error(
        "Line element entry \"{0}\" has unexpected length of {1} (expected 2, 3, 4, 5, 6 or 7)"
    )]
    EntryHasUnexpectedLength(String, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_count_decoded() {
        // given
        let signal = Signal {
            first: SignalNumber::Decoded(5),
            second: SignalNumber::Coded("bgc"),
            third: SignalNumber::Coded("cg"),
            fourth: SignalNumber::Decoded(2),
        };

        // when
        let count_of = signal.count_decoded();

        // then
        assert_eq!(count_of, 2);
    }

    #[test]
    fn decode_mixed_up_signals_should_return_26_instances() {
        // given
        let input = "be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe \
                            cefdb cefbgd gcbe\r\nedbfga begcd cbg gc gcadebf fbgde acbgfd abcde \
                            gfcbed gfec | fcgedb cgb dgebacf gc\r\nfgaebd cg bdaec gdafb agbcfd \
                            gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg\r\nfbegcd cbd adcefb \
                            dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb\r\n\
                            aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf \
                            egdcabf bgf bfgea\r\nfgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg \
                            bafgc acf | gebdcfa ecba ca fadegcb\r\ndbcfg fgd bdegcaf fgec aegbdf \
                            ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe\r\nbdfegc \
                            cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba \
                            cbgef\r\negadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | \
                            gbdfcae bgc cg cgb\r\ngcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge \
                            fdbac fegbdc | fgae cfgab fg bagce";

        // when
        let signals = decode_mixed_up_signals(input);

        // then
        assert!(signals.is_ok());
        assert_eq!(
            signals
                .unwrap()
                .iter()
                .map(Signal::count_decoded)
                .sum::<usize>(),
            26
        );
    }
}
