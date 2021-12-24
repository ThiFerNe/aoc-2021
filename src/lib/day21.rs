use std::num::ParseIntError;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day21";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 21: Dirac Dice")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day21-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day21Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day21Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let loosing_score_times_die_rolls =
        simulate_game_and_return_loosing_score_times_die_rolls(&file_contents)?;
    println!(
        "The loosing score multiplied by the die rolls is {}.",
        loosing_score_times_die_rolls
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day21Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not simulate game and return loosing score multiplied by die rolls ({0})")]
    SimulateGameAndReturnLoosingScoreTimesDieRolls(
        #[from] SimulateGameAndReturnLoosingScoreTimesDieRollsError,
    ),
}

pub fn simulate_game_and_return_loosing_score_times_die_rolls(
    starting_positions: &str,
) -> Result<u128, SimulateGameAndReturnLoosingScoreTimesDieRollsError> {
    let mut players = parse_players(starting_positions)?;
    let mut deterministic_die = DeterministicDie::with(1, 100);
    simulate_game(&mut players, &mut deterministic_die, 1000);
    let loosing_player = players
        .iter()
        .reduce(|a, b| if a.total_score < b.total_score { a } else { b })
        .unwrap();
    Ok(loosing_player.total_score as u128 * deterministic_die.roll_count)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SimulateGameAndReturnLoosingScoreTimesDieRollsError {
    #[error("Could not parse players ({0})")]
    ParsePlayers(#[from] ParsePlayersError),
}

fn parse_players(starting_positions: &str) -> Result<Vec<Player>, ParsePlayersError> {
    let mut players = starting_positions
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            Player::from_str(line)
                .map_err(|error| ParsePlayersError::PlayerFromStr(line.to_string(), error))
        })
        .collect::<Result<Vec<Player>, ParsePlayersError>>()?;
    players.sort_unstable_by_key(|s| s.id);
    Ok(players)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParsePlayersError {
    #[error("Could not parse player from string \"{0}\" ({1})")]
    PlayerFromStr(String, #[source] PlayerFromStrError),
}

fn simulate_game<D: Die>(players: &mut [Player], die: &mut D, target_score: u16) {
    let mut current_player = 0;
    while !players
        .iter()
        .any(|player| player.total_score >= target_score)
    {
        let die_roll = die.roll() + die.roll() + die.roll();
        players[current_player].move_by(die_roll);
        current_player = (current_player + 1) % players.len();
    }
}

trait Die {
    fn roll(&mut self) -> u16;
}

#[derive(Debug, Copy, Clone)]
struct DeterministicDie {
    next_number: u16,
    min_number: u16,
    max_number: u16,
    roll_count: u128,
}

impl DeterministicDie {
    fn with(min_inclusive: u16, max_inclusive: u16) -> Self {
        Self {
            next_number: min_inclusive,
            min_number: min_inclusive,
            max_number: max_inclusive,
            roll_count: 0,
        }
    }
}

impl Die for DeterministicDie {
    fn roll(&mut self) -> u16 {
        self.roll_count += 1;
        let output = self.next_number;
        self.next_number += 1;
        if self.next_number > self.max_number {
            self.next_number = self.min_number;
        }
        output
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Player {
    id: u8,
    position: u8,
    total_score: u16,
}

impl Player {
    fn move_by(&mut self, value: u16) {
        let mut new_position = self.position as u16 + value;
        while new_position > 10 {
            new_position -= 10;
        }
        self.position = new_position as u8;
        self.total_score += self.position as u16;
    }
}

impl FromStr for Player {
    type Err = PlayerFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("Player ")
            .ok_or(PlayerFromStrError::InvalidPrefix)
            .map(str::split_whitespace)
            .and_then(|mut suffix_parts| {
                suffix_parts
                    .next()
                    .ok_or(PlayerFromStrError::MissingId)
                    .and_then(|id| {
                        id.parse::<u8>()
                            .map_err(|error| PlayerFromStrError::ParseId(id.to_string(), error))
                    })
                    .and_then(|id| {
                        suffix_parts
                            .last()
                            .ok_or(PlayerFromStrError::MissingStartingPosition)
                            .and_then(|starting_position| {
                                starting_position
                                    .parse::<u8>()
                                    .map_err(|error| {
                                        PlayerFromStrError::ParseStartingPosition(
                                            starting_position.to_string(),
                                            error,
                                        )
                                    })
                                    .and_then(|starting_position| {
                                        if !(1..=10).contains(&starting_position) {
                                            Err(PlayerFromStrError::InvalidStartingPosition(
                                                starting_position,
                                            ))
                                        } else {
                                            Ok(starting_position)
                                        }
                                    })
                            })
                            .map(|position| Player {
                                id,
                                position,
                                total_score: 0,
                            })
                    })
            })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PlayerFromStrError {
    #[error("Starts with invalid prefix, needed \"Player \"")]
    InvalidPrefix,
    #[error("Missing player id")]
    MissingId,
    #[error("Could not parse player id from \"{0}\" ({1})")]
    ParseId(String, #[source] ParseIntError),
    #[error("Missing player starting position")]
    MissingStartingPosition,
    #[error("Could not parse player starting position from \"{0}\" ({1})")]
    ParseStartingPosition(String, #[source] ParseIntError),
    #[error("Starting position '{0}' is invalid, needs to be within 1 to 10")]
    InvalidStartingPosition(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_game_and_return_losing_score_times_dice_rolls() {
        // given
        let input = "Player 1 starting position: 4\r\nPlayer 2 starting position: 8\r\n";

        // when
        let losing_score_times_dice_rolls =
            simulate_game_and_return_loosing_score_times_die_rolls(input);

        // then
        assert_eq!(losing_score_times_dice_rolls, Ok(739785));
    }
}
