use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day10";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 10: Syntax Scoring")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day10-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day10Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day10Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let total_syntax_error_score = calculate_total_syntax_error_score(&file_contents)?;
    println!(
        "The total syntax error socre is: {}.",
        total_syntax_error_score
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day10Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate total syntax error score ({0})")]
    CalculateTotalSyntaxErrorScore(#[from] CalculateTotalSyntaxErrorScoreError),
}

pub fn calculate_total_syntax_error_score(
    navigation_subsystem: &str,
) -> Result<u128, CalculateTotalSyntaxErrorScoreError> {
    Ok(navigation_subsystem
        .lines()
        .enumerate()
        .map(|(lines_index, line)| {
            if line.trim().is_empty() {
                Err(CalculateTotalSyntaxErrorScoreError::LineIsEmpty(
                    lines_index,
                ))
            } else {
                let mut chunk_stack = Vec::new();
                for (line_index, current_character) in line.chars().enumerate() {
                    let current_symbol = SyntaxSymbol::try_from(current_character)?;
                    if current_symbol.is_opening() {
                        chunk_stack.push(current_symbol);
                    } else if let Some(opening_symbol) = chunk_stack.pop() {
                        let expected_closing_character = opening_symbol.closing_variation();
                        if expected_closing_character != current_symbol {
                            return Err(CalculateTotalSyntaxErrorScoreError::ExpectedButFound(
                                expected_closing_character,
                                current_symbol,
                                lines_index,
                                line_index,
                            ));
                        }
                    } else {
                        return Err(CalculateTotalSyntaxErrorScoreError::NotExpectedButFound(
                            current_symbol,
                            lines_index,
                            line_index,
                        ));
                    }
                }
                Ok(())
            }
        })
        .filter(Result::is_err)
        .map(Result::unwrap_err)
        .map(|error: CalculateTotalSyntaxErrorScoreError| match error {
            CalculateTotalSyntaxErrorScoreError::ExpectedButFound(_, found_symbol, _, _) => {
                Ok(match found_symbol {
                    SyntaxSymbol::Parentheses(_) => 3,
                    SyntaxSymbol::Brackets(_) => 57,
                    SyntaxSymbol::Braces(_) => 1197,
                    SyntaxSymbol::AngleBrackets(_) => 25137,
                })
            }
            _ => Err(error),
        })
        .collect::<Result<Vec<u128>, CalculateTotalSyntaxErrorScoreError>>()?
        .into_iter()
        .sum::<u128>())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateTotalSyntaxErrorScoreError {
    #[error("Line no. {0} is empty")]
    LineIsEmpty(usize),
    #[error("Could not parse a character ({0})")]
    SyntaxSymbolTryFrom(#[from] SyntaxSymbolTryFromError),
    #[error("Expected {0:?}, but found {1:?} instead (at line {2} index {3})")]
    ExpectedButFound(SyntaxSymbol, SyntaxSymbol, usize, usize),
    #[error("Expected nothing, but found {0:?} instead (at line {1} index {2})")]
    NotExpectedButFound(SyntaxSymbol, usize, usize),
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SyntaxSymbol {
    Parentheses(SymbolVariation),   // (
    Brackets(SymbolVariation),      // [
    Braces(SymbolVariation),        // {
    AngleBrackets(SymbolVariation), // <
}

impl SyntaxSymbol {
    fn is_opening(&self) -> bool {
        match self {
            SyntaxSymbol::Parentheses(opening) => matches!(opening, SymbolVariation::Opening),
            SyntaxSymbol::Brackets(opening) => matches!(opening, SymbolVariation::Opening),
            SyntaxSymbol::Braces(opening) => matches!(opening, SymbolVariation::Opening),
            SyntaxSymbol::AngleBrackets(opening) => matches!(opening, SymbolVariation::Opening),
        }
    }

    fn closing_variation(&self) -> SyntaxSymbol {
        match self {
            SyntaxSymbol::Parentheses(_) => SyntaxSymbol::Parentheses(SymbolVariation::Closing),
            SyntaxSymbol::Brackets(_) => SyntaxSymbol::Brackets(SymbolVariation::Closing),
            SyntaxSymbol::Braces(_) => SyntaxSymbol::Braces(SymbolVariation::Closing),
            SyntaxSymbol::AngleBrackets(_) => SyntaxSymbol::AngleBrackets(SymbolVariation::Closing),
        }
    }
}

impl TryFrom<char> for SyntaxSymbol {
    type Error = SyntaxSymbolTryFromError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '(' => Ok(Self::Parentheses(SymbolVariation::Opening)),
            ')' => Ok(Self::Parentheses(SymbolVariation::Closing)),
            '[' => Ok(Self::Brackets(SymbolVariation::Opening)),
            ']' => Ok(Self::Brackets(SymbolVariation::Closing)),
            '{' => Ok(Self::Braces(SymbolVariation::Opening)),
            '}' => Ok(Self::Braces(SymbolVariation::Closing)),
            '<' => Ok(Self::AngleBrackets(SymbolVariation::Opening)),
            '>' => Ok(Self::AngleBrackets(SymbolVariation::Closing)),
            _ => Err(SyntaxSymbolTryFromError(value)),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
#[error("Expected some of '(', ')', '[', ']', '{{', '}}', '<' or '>', but got {0}.")]
pub struct SyntaxSymbolTryFromError(char);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SymbolVariation {
    Opening,
    Closing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_total_syntax_error_score_should_return_26397() {
        // given
        let input = "[({(<(())[]>[[{[]{<()<>>\r\n[(()[<>])]({[<{<<[]>>(\r\n\
                            {([(<{}[<>[]}>{[]{[(<()>\r\n(((({<>}<{<{<>}{[]{[]{}\r\n\
                            [[<[([]))<([[{}[[()]]]\r\n[{[{({}]{}}([{[{{{}}([]\r\n\
                            {<[[]]>}<{[{[{[]{()[[[]\r\n[<(<(<(<{}))><([]([]()\r\n\
                            <{([([[(<>()){}]>(<<{{\r\n<{([{{}}[<[[[<>{}]]]>[]]";

        // when
        let total_syntax_error_score = calculate_total_syntax_error_score(input);

        // then
        assert_eq!(total_syntax_error_score, Ok(26397));
    }
}
