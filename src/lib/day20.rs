use std::fmt::{Display, Formatter};
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day20";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 20: Trench Map")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day20-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day20Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day20Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let count_of_enhancements = match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => 50,
        _ => 2,
    };
    let count_of_lit_pixels =
        count_lit_pixels_after_enhancement(&file_contents, count_of_enhancements)?;
    println!(
        "The count of lit pixels after {} enhancements is {}.",
        count_of_enhancements, count_of_lit_pixels
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day20Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count lit pixels after enhancement ({0})")]
    CountLitPixelsAfterEnhancement(#[from] CountLitPixelsAfterEnhancementError),
}

pub fn count_lit_pixels_after_enhancement(
    enhancement_algorithm_and_image: &str,
    count_of_enhancements: u128,
) -> Result<u128, CountLitPixelsAfterEnhancementError> {
    let (image_enhancement_algorithm, mut brightness_image) =
        parse_image_enhancement_and_image(enhancement_algorithm_and_image)?;

    for _ in 0..count_of_enhancements {
        brightness_image = enhance_image(&brightness_image, &image_enhancement_algorithm);
    }

    if brightness_image.background == PixelBrightness::Light {
        Err(CountLitPixelsAfterEnhancementError::InfiniteLitPixels)
    } else {
        Ok(brightness_image
            .data
            .iter()
            .flatten()
            .fold(0, |counter, next| {
                if matches!(next, PixelBrightness::Light) {
                    counter + 1
                } else {
                    counter
                }
            }))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountLitPixelsAfterEnhancementError {
    #[error("Could not parse image enhancement and image ({0})")]
    ParseImageEnhancementAndImage(#[from] ParseImageEnhancementAndImageError),
    #[error("After enhancement there are now infinite lit pixels")]
    InfiniteLitPixels,
}

fn enhance_image(
    brightness_image: &BrightnessImage,
    image_enhancement_algorithm: &ImageEnhancementAlgorithm,
) -> BrightnessImage {
    let brightness_area = |x: isize, y: isize| -> [PixelBrightness; 9] {
        let get_brightness_of = |x: isize, y: isize| -> PixelBrightness {
            if y < 0 || x < 0 {
                brightness_image.background
            } else {
                brightness_image
                    .data
                    .get(y as usize)
                    .map(|v: &Vec<PixelBrightness>| v.get(x as usize))
                    .flatten()
                    .copied()
                    .unwrap_or(brightness_image.background)
            }
        };
        [
            get_brightness_of(x - 1, y - 1),
            get_brightness_of(x, y - 1),
            get_brightness_of(x + 1, y - 1),
            get_brightness_of(x - 1, y),
            get_brightness_of(x, y),
            get_brightness_of(x + 1, y),
            get_brightness_of(x - 1, y + 1),
            get_brightness_of(x, y + 1),
            get_brightness_of(x + 1, y + 1),
        ]
    };

    BrightnessImage {
        data: (-1isize..(brightness_image.height as isize + 1))
            .map(|y| {
                (-1isize..(brightness_image.width as isize + 1))
                    .map(|x| {
                        image_enhancement_algorithm.0[brightness_area(x, y).into_iter().fold(
                            0usize,
                            |mut binary_number, next| {
                                binary_number <<= 1;
                                if next == PixelBrightness::Light {
                                    binary_number |= 1;
                                }
                                binary_number
                            },
                        )]
                    })
                    .collect::<Vec<PixelBrightness>>()
            })
            .collect::<Vec<Vec<PixelBrightness>>>(),
        background: match brightness_image.background {
            PixelBrightness::Light => image_enhancement_algorithm.0[511],
            PixelBrightness::Dark => image_enhancement_algorithm.0[0],
        },
        width: brightness_image.width + 2,
        height: brightness_image.height + 2,
    }
}

fn parse_image_enhancement_and_image(
    enhancement_algorithm_and_image: &str,
) -> Result<(ImageEnhancementAlgorithm, BrightnessImage), ParseImageEnhancementAndImageError> {
    let mut lines = enhancement_algorithm_and_image.lines();
    let image_enhancement_algorithm = ImageEnhancementAlgorithm::from_str(
        lines
            .next()
            .ok_or(ParseImageEnhancementAndImageError::MissingFirstLine)?,
    )?;
    let brightness_image = BrightnessImage::from_str(
        &lines
            .map(|line| format!("{}\r\n", line))
            .collect::<String>(),
    )?;
    Ok((image_enhancement_algorithm, brightness_image))
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseImageEnhancementAndImageError {
    #[error("Missing first line")]
    MissingFirstLine,
    #[error("Could not parse image enhancement algorithm from string ({0})")]
    ImageEnhancementAlgorithmFromStr(#[from] ImageEnhancementAlgorithmFromStrError),
    #[error("Could not parse input image from string ({0})")]
    BrightnessImageFromStr(#[from] BrightnessImageFromStrError),
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct ImageEnhancementAlgorithm(Vec<PixelBrightness>);

impl Display for ImageEnhancementAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for p in &self.0 {
            write!(f, "{}", p)?;
        }
        Ok(())
    }
}

impl FromStr for ImageEnhancementAlgorithm {
    type Err = ImageEnhancementAlgorithmFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pixel_brightness_vector = s
            .chars()
            .map(PixelBrightness::try_from)
            .collect::<Result<Vec<PixelBrightness>, PixelBrightnessTryFromCharError>>()?;
        if pixel_brightness_vector.len() != 512 {
            Err(ImageEnhancementAlgorithmFromStrError::LengthIsInvalid(
                pixel_brightness_vector.len(),
            ))
        } else {
            Ok(ImageEnhancementAlgorithm(pixel_brightness_vector))
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ImageEnhancementAlgorithmFromStrError {
    #[error("Could not parse pixel brightness ({0})")]
    PixelBrightnessTryFromChar(#[from] PixelBrightnessTryFromCharError),
    #[error("Parsed length of {0} is invalid, expected 512")]
    LengthIsInvalid(usize),
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct BrightnessImage {
    data: Vec<Vec<PixelBrightness>>,
    background: PixelBrightness,
    width: usize,
    height: usize,
}

impl Display for BrightnessImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..(self.width + 2) {
            write!(f, "{}", self.background)?;
        }
        writeln!(f)?;
        for y in 0..self.data.len() {
            write!(f, "{}", self.background)?;
            for x in 0..self.data[y].len() {
                write!(f, "{}", self.data[y][x])?;
            }
            writeln!(f, "{}", self.background)?;
        }
        for _ in 0..(self.width + 2) {
            write!(f, "{}", self.background)?;
        }
        writeln!(f)
    }
}

impl FromStr for BrightnessImage {
    type Err = BrightnessImageFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.chars()
                    .map(PixelBrightness::try_from)
                    .collect::<Result<Vec<PixelBrightness>, PixelBrightnessTryFromCharError>>()
            })
            .collect::<Result<Vec<Vec<PixelBrightness>>, PixelBrightnessTryFromCharError>>()?;
        let general_line_len = data[0].len();
        if data.iter().any(|line| line.len() != general_line_len) {
            Err(BrightnessImageFromStrError::UnequalDimensions(
                general_line_len,
            ))
        } else {
            let height = data.len();
            Ok(BrightnessImage {
                data,
                background: PixelBrightness::Dark,
                width: general_line_len,
                height,
            })
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BrightnessImageFromStrError {
    #[error("Could not parse pixel brightness ({0})")]
    PixelBrightnessTryFromChar(#[from] PixelBrightnessTryFromCharError),
    #[error("Parsed brightness image has unequal dimensions (not all lines have width {0})")]
    UnequalDimensions(usize),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum PixelBrightness {
    Light,
    Dark,
}

impl Display for PixelBrightness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelBrightness::Light => write!(f, "#"),
            PixelBrightness::Dark => write!(f, "."),
        }
    }
}

impl TryFrom<char> for PixelBrightness {
    type Error = PixelBrightnessTryFromCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Dark),
            '#' => Ok(Self::Light),
            _ => Err(PixelBrightnessTryFromCharError::InvalidCharacter(value)),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PixelBrightnessTryFromCharError {
    #[error("Got invalid character '{0}'")]
    InvalidCharacter(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_lit_pixels_after_enhancement_two_times() {
        // given
        let input = "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##.\
                            .###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#...\
                            ...#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##.\
                            .....#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.\
                            #...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......\
                            #.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##.\
                            .#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..###\
                            ##........#..####......#..#\r\n\r\n#..#.\r\n#....\r\n##..#\r\n..#..\r\n\
                            ..###";

        // when
        let count_of_lit_pixels = count_lit_pixels_after_enhancement(input, 2);

        // then
        assert_eq!(count_of_lit_pixels, Ok(35));
    }

    #[test]
    fn test_count_lit_pixels_after_enhancement_fifty_times() {
        // given
        let input = "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##.\
                            .###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#...\
                            ...#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##.\
                            .....#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.\
                            #...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......\
                            #.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##.\
                            .#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..###\
                            ##........#..####......#..#\r\n\r\n#..#.\r\n#....\r\n##..#\r\n..#..\r\n\
                            ..###";

        // when
        let count_of_lit_pixels = count_lit_pixels_after_enhancement(input, 50);

        // then
        assert_eq!(count_of_lit_pixels, Ok(3351));
    }
}
