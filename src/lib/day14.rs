use std::collections::HashMap;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day14";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 14: ?Extended Polymerization")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day14-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day14Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day14Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let processed_polymer_character_count =
        process_polymer_pair_insertion_rules(&file_contents, 10)?
            .chars()
            .fold(HashMap::new(), |mut map, character| {
                map.entry(character).and_modify(|v| *v += 1).or_insert(1);
                map
            });
    let (most_common, least_common) = processed_polymer_character_count
        .into_iter()
        .fold(None, |output, next| match output {
            None => Some((next, next)),
            Some((most_common, least_common)) => Some((
                if next.1 > most_common.1 {
                    next
                } else {
                    most_common
                },
                if next.1 < least_common.1 {
                    next
                } else {
                    least_common
                },
            )),
        })
        .unwrap();
    println!(
        "The answer to the puzzle is {:?} - {:?} = {}",
        most_common,
        least_common,
        most_common.1 - least_common.1
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day14Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not process polymer pair insertions rules ({0})")]
    ProcessPolymerPairInsertionRules(#[from] ProcessPolymerPairInsertionRulesError),
}

pub fn process_polymer_pair_insertion_rules(
    instructions: &str,
    step_count: u128,
) -> Result<String, ProcessPolymerPairInsertionRulesError> {
    fn process(
        polymer_instructions: PolymerInstructions,
    ) -> Result<String, ProcessPolymerPairInsertionRulesError> {
        let mut found = Vec::new();
        for index in 0..polymer_instructions.polymer_template.len() {
            for pair_insertion_rule in &polymer_instructions.pair_insertion_rules {
                if pair_insertion_rule.search_str
                    == polymer_instructions.polymer_template[index
                        ..(index + pair_insertion_rule.search_str.len())
                            .min(polymer_instructions.polymer_template.len())]
                {
                    found.push((index + 1, pair_insertion_rule.clone()));
                }
            }
        }
        Ok(polymer_instructions
            .polymer_template
            .chars()
            .enumerate()
            .fold(String::new(), |mut output, (index, character)| {
                let (current, remaining) = found
                    .clone()
                    .into_iter()
                    .partition(|(found_index, _)| index == *found_index);
                found = remaining;
                current.into_iter().for_each(|(_, rule)| {
                    output.push_str(&rule.insert_str);
                });
                output.push(character);
                output
            }))
    }
    let mut polymer_instructions = PolymerInstructions::from_str(instructions)?;
    for _ in 0..step_count {
        polymer_instructions = PolymerInstructions {
            polymer_template: process(polymer_instructions.clone())?,
            pair_insertion_rules: polymer_instructions.pair_insertion_rules,
        };
    }
    Ok(polymer_instructions.polymer_template)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProcessPolymerPairInsertionRulesError {
    #[error("Could not parse polymer instructions ({0})")]
    PolymerInstructionsFromStr(#[from] PolymerInstructionsFromStrError),
}

#[derive(Clone)]
struct PolymerInstructions {
    polymer_template: String,
    pair_insertion_rules: Vec<PairInsertionRule>,
}

impl FromStr for PolymerInstructions {
    type Err = PolymerInstructionsFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (optional_first_line, _, pair_insertions) = s.lines().fold(
            (None, false, Vec::new()),
            |(mut optional_first_line, mut divider_line, mut pair_insertions), next| {
                if optional_first_line.is_some() {
                    if divider_line {
                        pair_insertions.push(PairInsertionRule::from_str(next).map_err(|error| {
                            PolymerInstructionsFromStrError::PairInsertionRuleFromStr(
                                next.to_string(),
                                error,
                            )
                        }));
                    } else {
                        divider_line = true;
                    }
                } else {
                    optional_first_line = Some(next.to_string());
                }
                (optional_first_line, divider_line, pair_insertions)
            },
        );
        Ok(Self {
            polymer_template: optional_first_line
                .ok_or(PolymerInstructionsFromStrError::NoLinesInInput)?,
            pair_insertion_rules: pair_insertions
                .into_iter()
                .collect::<Result<Vec<PairInsertionRule>, PolymerInstructionsFromStrError>>()?,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PolymerInstructionsFromStrError {
    #[error("There were no lines to parse")]
    NoLinesInInput,
    #[error("Could not parse pair insertion rule from str \"{0}\" ({1})")]
    PairInsertionRuleFromStr(String, #[source] PairInsertionRuleFromStrError),
}

#[derive(Clone)]
struct PairInsertionRule {
    search_str: String,
    insert_str: String,
}

impl FromStr for PairInsertionRule {
    type Err = PairInsertionRuleFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let k: [&str; 3] =
            s.split(' ')
                .collect::<Vec<&str>>()
                .try_into()
                .map_err(|_: Vec<&str>| {
                    PairInsertionRuleFromStrError::NotThreeElements(s.to_string())
                })?;
        Ok(Self {
            search_str: k[0].to_string(),
            insert_str: k[2].to_string(),
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PairInsertionRuleFromStrError {
    #[error("Pair insertion rule does not have three elements \"{0}\"")]
    NotThreeElements(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_b_1749_c_298_h_161_n_865() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 10);

        // then
        println!("{:?}", processed_polymer);
        assert!(processed_polymer.is_ok());
        let processed_polymer = processed_polymer.unwrap();
        let map = processed_polymer
            .chars()
            .fold(HashMap::new(), |mut map, character| {
                map.entry(character).and_modify(|v| *v += 1).or_insert(1);
                map
            });
        assert_eq!(map.get(&'B'), Some(&1749));
        assert_eq!(map.get(&'C'), Some(&298));
        assert_eq!(map.get(&'H'), Some(&161));
        assert_eq!(map.get(&'N'), Some(&865));
    }

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_ncnbchb() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 1);

        // then
        assert_eq!(processed_polymer, Ok("NCNBCHB".to_string()));
    }

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_nbccnbbbcbhcb() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 2);

        // then
        assert_eq!(processed_polymer, Ok("NBCCNBBBCBHCB".to_string()));
    }

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_nbbbcnccnbbnbnbbchbhhbchb() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 3);

        // then
        assert_eq!(
            processed_polymer,
            Ok("NBBBCNCCNBBNBNBBCHBHHBCHB".to_string())
        );
    }

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_nbbnbnbbccnbcnccnbbnbbnbbbnbbnbbcbhcbhhbhcbbcbhcb(
    ) {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 4);

        // then
        assert_eq!(
            processed_polymer,
            Ok("NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB".to_string())
        );
    }
}
