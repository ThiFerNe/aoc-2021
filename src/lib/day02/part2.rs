use std::num::ParseIntError;

use thiserror::Error;

#[derive(Default)]
pub struct Submarine {
    pub position: Position,
}

impl Submarine {
    pub fn drive(&mut self, course: &str) -> Result<(), SubmarineDriveError> {
        course
            .split(|c| c == '\r' || c == '\n')
            .filter(|line| !line.is_empty())
            .enumerate()
            .map(|(line_nr, line)| {
                let elements = line.split(' ').collect::<Vec<&str>>();
                if elements.len() == 2 {
                    match elements[1].parse::<u128>() {
                        Ok(distance) => Ok((line_nr, line, elements[0], distance)),
                        Err(error) => Err(SubmarineDriveError::LineParseNumber(
                            line_nr,
                            line.to_string(),
                            error,
                        )),
                    }
                } else {
                    Err(SubmarineDriveError::LineWrongElementsCount(
                        line_nr,
                        line.to_string(),
                        elements.len(),
                    ))
                }
            })
            .collect::<Result<Vec<(usize, &str, &str, u128)>, SubmarineDriveError>>()?
            .into_iter()
            .map(
                |(line_nr, line, direction, distance)| match direction.to_lowercase().as_str() {
                    "forward" => {
                        self.position.forward(distance);
                        Ok(())
                    }
                    "down" => {
                        self.position.down(distance);
                        Ok(())
                    }
                    "up" => {
                        self.position.up(distance);
                        Ok(())
                    }
                    _ => Err(SubmarineDriveError::UnknownCommand(
                        direction.to_string(),
                        line.to_string(),
                        line_nr,
                    )),
                },
            )
            .collect::<Result<Vec<()>, SubmarineDriveError>>()
            .map(|_| ())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SubmarineDriveError {
    #[error("unknown command \"{0}\" at line no. {2} \"{1}\"")]
    UnknownCommand(String, String, usize),
    #[error("could not parse line no. {0} \"{1}\" ({2})")]
    LineParseNumber(usize, String, ParseIntError),
    #[error("line no. {0} \"{1}\" has wrong ({2}) count of elements")]
    LineWrongElementsCount(usize, String, usize),
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Position {
    aim: i128,
    horizontal: u128,
    depth: u128,
}

impl Position {
    // Allowing dead code for the test cases to work
    #[allow(dead_code)]
    fn new(aim: i128, horizontal: u128, depth: u128) -> Self {
        Self {
            aim,
            horizontal,
            depth,
        }
    }

    fn forward(&mut self, units: u128) {
        self.horizontal = self
            .horizontal
            .checked_add(units)
            .unwrap_or(self.horizontal);
        self.depth = (self.depth as i128)
            .checked_add(self.aim * units as i128)
            .unwrap_or(self.depth as i128) as u128;
    }

    fn down(&mut self, units: u128) {
        self.aim = self.aim.checked_add(units as i128).unwrap_or(self.aim);
    }

    fn up(&mut self, units: u128) {
        self.aim = self.aim.checked_sub(units as i128).unwrap_or(self.aim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submarine_default() {
        // when
        let submarine = Submarine::default();

        // then
        assert_eq!(submarine.position, Position::default());
    }

    #[test]
    fn test_submarine_drive_forward() {
        // given
        let mut submarine = Submarine::default();

        // when
        let drive = submarine.drive("forward 5");

        // then
        assert_eq!(drive, Ok(()));
        assert_eq!(submarine.position, Position::new(0, 5, 0));
    }

    #[test]
    fn test_submarine_drive_down() {
        // given
        let mut submarine = Submarine::default();

        // when
        let drive = submarine.drive("down 4");

        // then
        assert_eq!(drive, Ok(()));
        assert_eq!(submarine.position, Position::new(4, 0, 0));
    }

    #[test]
    fn test_submarine_drive_up() {
        // given
        let mut submarine = Submarine::default();
        assert_eq!(submarine.drive("down 2"), Ok(()));

        // when
        let drive = submarine.drive("up 1");

        // then
        assert_eq!(drive, Ok(()));
        assert_eq!(submarine.position, Position::new(1, 0, 0));
    }

    #[test]
    fn test_submarine_drive_example() {
        // given
        let course = "forward 5\r\ndown 5\r\nforward 8\r\nup 3\r\ndown 8\r\nforward 2";
        let mut submarine = Submarine::default();

        // when
        let drive = submarine.drive(&course);

        // then
        assert_eq!(drive, Ok(()));
        assert_eq!(submarine.position, Position::new(10, 15, 60));
    }

    #[test]
    fn test_submarine_drive_random() {
        // given
        let random_course = (1..(rand::random::<f64>() * 16f64) as u8)
            .into_iter()
            .map(|_| {
                let direction: (&str, fn(&mut Position, u128)) =
                    match (rand::random::<f64>() * 3f64) as u8 {
                        0 => ("forward", Position::forward),
                        1 => ("down", Position::down),
                        _ => ("up", Position::up),
                    };
                let distance = rand::random::<u8>() as u128;
                (direction.0, distance, direction.1)
            })
            .fold(
                (String::new(), Position::default()),
                |(mut output, mut position),
                 (next_direction, next_distance, next_direction_function)| {
                    next_direction_function(&mut position, next_distance);
                    if !output.is_empty() {
                        output.push_str("\r\n");
                    }
                    output.push_str(&format!("{} {}", next_direction, next_distance));
                    (output, position)
                },
            );
        let mut submarine = Submarine::default();

        // when
        let drive = submarine.drive(&random_course.0);

        // then
        assert_eq!(drive, Ok(()));
        assert_eq!(submarine.position, random_course.1);
    }

    #[test]
    fn test_position_default() {
        // when
        let position = Position::default();

        // then
        assert_eq!(position.aim, 0);
        assert_eq!(position.horizontal, 0);
        assert_eq!(position.depth, 0);
    }

    #[test]
    fn test_position_default_forward() {
        // given
        let mut position = Position::default();

        // when
        position.forward(1);

        // then
        assert_eq!(position.aim, 0);
        assert_eq!(position.horizontal, 1);
        assert_eq!(position.depth, 0);
    }

    #[test]
    fn test_position_default_down() {
        // given
        let mut position = Position::default();

        // when
        position.down(1);

        // then
        assert_eq!(position.aim, 1);
        assert_eq!(position.horizontal, 0);
        assert_eq!(position.depth, 0);
    }

    #[test]
    fn test_position_default_up() {
        // given
        let mut position = Position::default();

        // when
        position.up(1);

        // then
        assert_eq!(position.aim, -1);
        assert_eq!(position.horizontal, 0);
        assert_eq!(position.depth, 0);
    }

    #[test]
    fn test_position_new() {
        // when
        let position = Position::new(3, 1, 5);

        // then
        assert_eq!(position.aim, 3);
        assert_eq!(position.horizontal, 1);
        assert_eq!(position.depth, 5);
    }

    #[test]
    fn test_position_new_forward() {
        // given
        let mut position = Position::new(3, 1, 5);

        // when
        position.forward(1);

        // then
        assert_eq!(position.aim, 3);
        assert_eq!(position.horizontal, 2);
        assert_eq!(position.depth, 8);
    }

    #[test]
    fn test_position_new_down() {
        // given
        let mut position = Position::new(3, 1, 5);

        // when
        position.down(1);

        // then
        assert_eq!(position.aim, 4);
        assert_eq!(position.horizontal, 1);
        assert_eq!(position.depth, 5);
    }

    #[test]
    fn test_position_new_up() {
        // given
        let mut position = Position::new(3, 1, 5);

        // when
        position.up(1);

        // then
        assert_eq!(position.aim, 2);
        assert_eq!(position.horizontal, 1);
        assert_eq!(position.depth, 5);
    }
}
