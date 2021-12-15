use std::collections::HashMap;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

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
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day14Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day14Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let step_count = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => 40,
        _ => 10,
    };
    let processed_polymer_character_count =
        process_polymer_pair_insertion_rules(&file_contents, step_count)?;
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
        "The answer to the puzzle with {} steps is {:?} - {:?} = {}",
        step_count,
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
) -> Result<HashMap<char, u128>, ProcessPolymerPairInsertionRulesError> {
    let polymer_instructions = PolymerInstructions::from_str(instructions)?;

    let (mut bucket_pair_counting_map, optional_last_character) =
        polymer_instructions.polymer_template.chars().fold(
            (HashMap::new(), None),
            |(mut counting_hash_map, optional_last_character): (
                HashMap<(char, char), u128>,
                Option<char>,
            ),
             next_character| {
                if let Some(last_character) = optional_last_character {
                    counting_hash_map
                        .entry((last_character, next_character))
                        .and_modify(|c| *c += 1)
                        .or_insert(1);
                }
                (counting_hash_map, Some(next_character))
            },
        );
    if let Some(last_character) = optional_last_character {
        bucket_pair_counting_map
            .entry((last_character, '\0'))
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    for _ in 0..step_count {
        let mut output = HashMap::new();
        for (pair, count) in bucket_pair_counting_map.clone().into_iter() {
            let pair_search_str = format!("{}{}", pair.0, pair.1);
            let mut last_first_pair_character = pair.0;
            for pair_insertion_rule in &polymer_instructions.pair_insertion_rules {
                if pair_insertion_rule.search_str == pair_search_str {
                    let insert_character = pair_insertion_rule
                        .insert_str
                        .chars()
                        .collect::<Vec<char>>()[0];
                    output
                        .entry((last_first_pair_character, insert_character))
                        .and_modify(|c| *c += count)
                        .or_insert(count);
                    last_first_pair_character = insert_character;
                }
            }
            output
                .entry((last_first_pair_character, pair.1))
                .and_modify(|c| *c += count)
                .or_insert(count);
        }
        bucket_pair_counting_map = output;
    }

    let mut output = HashMap::new();
    for ((character, _), counter) in bucket_pair_counting_map.into_iter() {
        output
            .entry(character)
            .and_modify(|c| *c += counter)
            .or_insert(counter);
    }

    Ok(output)
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

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_b_1749_c_298_h_161_n_865() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 10);

        // then
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'B'), Some(&1749));
        assert_eq!(processed_polymer.get(&'C'), Some(&298));
        assert_eq!(processed_polymer.get(&'H'), Some(&161));
        assert_eq!(processed_polymer.get(&'N'), Some(&865));
    }

    #[test]
    fn process_polymer_pair_insertion_rules_should_return_b_2192039569602_h_3849876073() {
        // given
        let input = "NNCB\r\n\r\nCH -> B\r\nHH -> N\r\nCB -> H\r\nNH -> C\r\nHB -> C\r\n\
                            HC -> B\r\nHN -> C\r\nNN -> C\r\nBH -> H\r\nNC -> B\r\nNB -> B\r\n\
                            BN -> B\r\nBB -> N\r\nBC -> B\r\nCC -> N\r\nCN -> C";

        // when
        let processed_polymer = process_polymer_pair_insertion_rules(input, 40);

        // then
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'B'), Some(&2192039569602));
        assert_eq!(processed_polymer.get(&'H'), Some(&3849876073));
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
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'N'), Some(&2));
        assert_eq!(processed_polymer.get(&'C'), Some(&2));
        assert_eq!(processed_polymer.get(&'B'), Some(&2));
        assert_eq!(processed_polymer.get(&'H'), Some(&1));
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
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'N'), Some(&2));
        assert_eq!(processed_polymer.get(&'C'), Some(&4));
        assert_eq!(processed_polymer.get(&'B'), Some(&6));
        assert_eq!(processed_polymer.get(&'H'), Some(&1));
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
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'N'), Some(&5));
        assert_eq!(processed_polymer.get(&'C'), Some(&5));
        assert_eq!(processed_polymer.get(&'B'), Some(&11));
        assert_eq!(processed_polymer.get(&'H'), Some(&4));
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
        let processed_polymer = processed_polymer.unwrap();
        assert_eq!(processed_polymer.get(&'N'), Some(&11));
        assert_eq!(processed_polymer.get(&'C'), Some(&10));
        assert_eq!(processed_polymer.get(&'B'), Some(&23));
        assert_eq!(processed_polymer.get(&'H'), Some(&5));
    }
}
