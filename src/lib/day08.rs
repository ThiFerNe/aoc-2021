use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

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
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day08Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day08Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let signals = decode_mixed_up_signals(&file_contents, DecodingPower::Full)?;
            println!(
                "The sum of all decoded digits is {}.",
                signals
                    .iter()
                    .map(Signal::as_number)
                    .map(|v| v as u128)
                    .sum::<u128>()
            );
        }
        _ => {
            let signals = decode_mixed_up_signals(&file_contents, DecodingPower::Half)?;
            println!(
                "The digits 1, 4, 7, 8 appear {} times.",
                signals.iter().map(Signal::count_decoded).sum::<usize>()
            );
        }
    };
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
    decoding_power: DecodingPower,
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

    fn map_line_a<'a>(
        line: ([&str; 10], [&'a str; 4]),
    ) -> Result<Signal<'a>, DecodeMixedUpSignalsError> {
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
    }

    fn map_line_b<'a>(
        line: ([&str; 10], [&'a str; 4]),
    ) -> Result<Signal<'a>, DecodeMixedUpSignalsError> {
        /*
          0000
         1    2
         1    2
          3333
         4    5
         4    5
          6666
        */
        let mut notes: [Vec<char>; 7] = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];

        // 1) remove noted characters from input and then note remaining input characters into the notes
        let indices_per_entry_length_1 = [
            (2, vec![2, 5]),
            (3, vec![0, 2, 5]),
            (4, vec![1, 2, 3, 5]),
            (7, vec![0, 1, 2, 3, 4, 5, 6]),
        ];
        for (entry_length, indices) in indices_per_entry_length_1 {
            let (retained_chars, removed_chars): (Vec<char>, Vec<char>) = line
                .0
                .iter()
                .find(|entry| entry.len() == entry_length)
                .ok_or(DecodeMixedUpSignalsError::MissingEntryWithLength(
                    entry_length,
                ))?
                .chars()
                .partition(|character| !notes.iter().any(|segment| segment.contains(character)));
            for index in indices {
                if !removed_chars
                    .iter()
                    .any(|character| notes[index].contains(character))
                {
                    notes[index].extend(&retained_chars);
                }
            }
        }

        // 2) remove from note segments chars which are not in the entries; if then segment only has one char, remove that from every other segment
        let indices_per_entry_length_2 = [(6, vec![0, 1, 5, 6]), (5, vec![0, 3, 6])];
        for (entry_length, indices) in indices_per_entry_length_2 {
            let entries_with_entry_length = line
                .0
                .iter()
                .filter(|entry| entry.len() == entry_length)
                .collect::<Vec<&&str>>();
            for entry in entries_with_entry_length {
                for index in &indices {
                    notes[*index].retain(|character| entry.contains(*character));
                    if notes[*index].len() == 1 {
                        let character_to_remove = notes[*index][0];
                        notes
                            .iter_mut()
                            .enumerate()
                            .filter(|(inner_index, _)| *index != *inner_index)
                            .for_each(|(_, segment)| {
                                segment.retain(|character| *character != character_to_remove)
                            });
                    }
                }
            }
        }

        // 3) only one character per segment should remain
        let notes: [char; 7] = notes
            .into_iter()
            .enumerate()
            .map(|(index, segment)| {
                if segment.len() == 1 {
                    Ok(segment[0])
                } else {
                    let mut line_str = line.0.join(" ");
                    line_str.push_str(" | ");
                    line_str.push_str(&line.1.join(" "));
                    Err(
                        DecodeMixedUpSignalsError::DeducedSegmentHasUnexpectedPossibilities(
                            line_str, index, segment,
                        ),
                    )
                }
            })
            .collect::<Result<Vec<char>, DecodeMixedUpSignalsError>>()?
            .try_into()
            .unwrap();

        // 4) convert second element
        line.1
            .iter()
            .map(|entry| -> Result<SignalNumber, DecodeMixedUpSignalsError> {
                let numbers_indices = [
                    vec![0, 1, 2, 4, 5, 6],
                    vec![2, 5],
                    vec![0, 2, 3, 4, 6],
                    vec![0, 2, 3, 5, 6],
                    vec![1, 2, 3, 5],
                    vec![0, 1, 3, 5, 6],
                    vec![0, 1, 3, 4, 5, 6],
                    vec![0, 2, 5],
                    vec![0, 1, 2, 3, 4, 5, 6],
                    vec![0, 1, 2, 3, 5, 6],
                ];
                for (index, number_indices) in numbers_indices.iter().enumerate() {
                    if number_indices
                        .iter()
                        .all(|internal_index| entry.contains(notes[*internal_index]))
                        && entry.len() == numbers_indices[index].len()
                    {
                        return Ok(SignalNumber::Decoded(index as u8));
                    }
                }
                Ok(SignalNumber::Coded(entry))
            })
            .collect::<Result<Vec<SignalNumber>, DecodeMixedUpSignalsError>>()?
            .into_iter()
            .fold(
                (None, None, None, None),
                |(mut a, mut b, mut c, mut s), next| {
                    match a {
                        None => a = Some(next),
                        Some(ua) => match b {
                            None => b = Some(next),
                            Some(ub) => match c {
                                None => c = Some(next),
                                Some(uc) => {
                                    s = Some(Signal {
                                        first: ua,
                                        second: ub,
                                        third: uc,
                                        fourth: next,
                                    })
                                }
                            },
                        },
                    }
                    (a, b, c, s)
                },
            )
            .3
            .ok_or_else(|| {
                let mut line_str = line.0.join(" ");
                line_str.push_str(" | ");
                line_str.push_str(&line.1.join(" "));
                DecodeMixedUpSignalsError::NotEnoughEntriesInSecondElement(line_str)
            })
    }

    lines
        .into_iter()
        .map(|line| match decoding_power {
            DecodingPower::Half => map_line_a(line),
            DecodingPower::Full => map_line_b(line),
        })
        .collect::<Result<Vec<Signal>, DecodeMixedUpSignalsError>>()
}

pub enum DecodingPower {
    Half,
    Full,
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

    pub fn as_number(&self) -> u16 {
        (self.first.unwrap() as u16) * 1000
            + (self.second.unwrap() as u16) * 100
            + (self.third.unwrap() as u16) * 10
            + (self.fourth.unwrap() as u16)
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum SignalNumber<'a> {
    Decoded(u8),
    Coded(&'a str),
}

impl<'a> SignalNumber<'a> {
    pub fn unwrap(self) -> u8 {
        match self {
            SignalNumber::Decoded(number) => number,
            SignalNumber::Coded(code) => panic!("SignalNumber is Coded with \"{}\"", code),
        }
    }
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
    #[error("Missing line element entry with length {0}")]
    MissingEntryWithLength(usize),
    #[error("Deduced segment no. {1} of line \"{0}\" has unexpected possibilities ({1:?})")]
    DeducedSegmentHasUnexpectedPossibilities(String, usize, Vec<char>),
    #[error("Not enough entries in second element for line \"{0}\"")]
    NotEnoughEntriesInSecondElement(String),
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
    fn signal_as_number() {
        // given
        let signal = Signal {
            first: SignalNumber::Decoded(5),
            second: SignalNumber::Decoded(3),
            third: SignalNumber::Decoded(2),
            fourth: SignalNumber::Decoded(2),
        };

        // when
        let number = signal.as_number();

        // then
        assert_eq!(number, 5322);
    }

    #[test]
    #[should_panic]
    fn signal_as_number_panics() {
        // given
        let signal = Signal {
            first: SignalNumber::Decoded(5),
            second: SignalNumber::Coded("bgc"),
            third: SignalNumber::Coded("cg"),
            fourth: SignalNumber::Decoded(2),
        };

        // when + then
        signal.as_number();
    }

    #[test]
    fn signal_number_unwrap() {
        // given
        let signal_number = SignalNumber::Decoded(5);

        // when
        let number = signal_number.unwrap();

        // then
        assert_eq!(number, 5);
    }

    #[test]
    #[should_panic]
    fn signal_number_unwrap_panics() {
        // given
        let signal_number = SignalNumber::Coded("fcgedb");

        // when + then
        signal_number.unwrap();
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
        let signals = decode_mixed_up_signals(input, DecodingPower::Half);

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

    #[test]
    fn decode_mixed_up_signals_should_return_sum_of_61229() {
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
        let signals = decode_mixed_up_signals(input, DecodingPower::Full);

        // then
        assert!(signals.is_ok());
        let signals = signals.unwrap();
        assert_eq!(
            signals.iter().map(Signal::as_number).collect::<Vec<u16>>(),
            vec![8394, 9781, 1197, 9361, 4873, 8418, 4548, 1625, 8717, 4315]
        );
        assert_eq!(
            signals
                .iter()
                .map(Signal::as_number)
                .map(|v| v as u128)
                .sum::<u128>(),
            61229
        );
    }
}
