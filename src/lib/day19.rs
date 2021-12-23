use std::collections::HashMap;
use std::fmt::Display;
use std::num::ParseIntError;
use std::ops::{Add, Sub};
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day19";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 19: Beacon Scanner")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day19-input"),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day19Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day19Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let largest_manhattan_distance_between_any_two_scanners =
                find_largest_manhattan_distance_between_any_two_scanners(&file_contents)?;
            println!(
                "The largest Manhattan distance between any two scanners is {}.",
                largest_manhattan_distance_between_any_two_scanners
            );
        }
        _ => {
            let unique_detected_beacons = count_unique_detected_beacons(&file_contents)?;
            println!("There are {} beacons.", unique_detected_beacons);
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day19Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count unique detected beacons ({0})")]
    CountUniqueDetectedBeacons(#[from] CountUniqueDetectedBeaconsError),
    #[error("Could not find largest Manhattan distance between any two scanners ({0})")]
    FindLargestManhattanDistanceBetweenAnyTwoScanners(
        #[from] FindLargestManhattanDistanceBetweenAnyTwoScannersError,
    ),
}

pub fn count_unique_detected_beacons(
    relative_beacon_positions: &str,
) -> Result<u128, CountUniqueDetectedBeaconsError> {
    let scanner_reports = parse_scanner_reports(relative_beacon_positions)?;
    let positioned_scanners = position_scanners(scanner_reports)?;
    let all_absolute_beacon_positions = positioned_scanners
        .into_iter()
        .flat_map(|scanner| scanner.scanned_beacons)
        .fold(Vec::new(), |mut output, next| {
            if !output.contains(&next) {
                output.push(next);
            }
            output
        });
    Ok(all_absolute_beacon_positions.len() as u128)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountUniqueDetectedBeaconsError {
    #[error("Could not parse scanner reports from string ({0})")]
    ParseScannerReports(#[from] ParseScannerReportsError),
    #[error("Could not position scanners ({0})")]
    PositionScanners(#[from] PositionScannersError),
}

pub fn find_largest_manhattan_distance_between_any_two_scanners(
    relative_beacon_positions: &str,
) -> Result<u128, FindLargestManhattanDistanceBetweenAnyTwoScannersError> {
    let scanner_reports = parse_scanner_reports(relative_beacon_positions)?;
    let positioned_scanners = position_scanners(scanner_reports)?;
    positioned_scanners
        .iter()
        .flat_map(|scanner_a| {
            positioned_scanners
                .iter()
                .map(move |scanner_b| (scanner_a, scanner_b))
        })
        .map(|(scanner_a, scanner_b)| {
            let distance_vector = scanner_a.position.0 - scanner_b.position.0;
            (distance_vector.x.abs() + distance_vector.y.abs() + distance_vector.z.abs()) as u128
        })
        .max()
        .ok_or(FindLargestManhattanDistanceBetweenAnyTwoScannersError::MissingScanners)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindLargestManhattanDistanceBetweenAnyTwoScannersError {
    #[error("Could not parse scanner reports ({0})")]
    ParseScannerReports(#[from] ParseScannerReportsError),
    #[error("Could not position scanners ({0})")]
    PositionScanners(#[from] PositionScannersError),
    #[error("Missing scanners in input")]
    MissingScanners,
}

fn position_scanners(
    scanner_reports: Vec<ScannerReport>,
) -> Result<Vec<Scanner>, PositionScannersError> {
    let mut scanner_reports = scanner_reports
        .into_iter()
        .map(|scanner_report| (scanner_report.id, scanner_report))
        .collect::<HashMap<ScannerId, ScannerReport>>();

    let mut positioned_scanners = vec![scanner_reports
        .remove(&ScannerId(0))
        .map(|scanner_report| scanner_report.into_scanner(&Rototranslation3D::identity()))
        .ok_or(PositionScannersError::MissingInitialScanner)?];

    println!(
        "Going to position {} scanner reports...",
        scanner_reports.len()
    );
    while !scanner_reports.is_empty() {
        let mut found = false;
        let scanner_report_keys = scanner_reports.keys().copied().collect::<Vec<_>>();
        for scanner_id in scanner_report_keys {
            if let Some(rototranslation) = positioned_scanners
                .iter()
                .flat_map(|positioned_scanner| {
                    find_rototranslation_for_b_with_12_fitting_beacons(
                        &positioned_scanner.scanned_beacons,
                        &scanner_reports[&scanner_id].scanned_beacons,
                    )
                })
                .next()
            {
                println!("Found {}. rototranslation", positioned_scanners.len());
                let new_scanner = scanner_reports
                    .remove(&scanner_id)
                    .unwrap()
                    .into_scanner(&rototranslation);
                positioned_scanners.push(new_scanner);
                found = true;
            }
        }
        if !found {
            eprintln!("scanner_reports ({})", scanner_reports.len());
            eprintln!("{:?}", scanner_reports);
            eprintln!("{:?}", scanner_reports.keys());
            panic!("Could not fit any ScannerReport in the already positioned Scanners!");
        }
    }
    Ok(positioned_scanners)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PositionScannersError {
    #[error("Missing initial scanner (with number 0)")]
    MissingInitialScanner,
}

fn parse_scanner_reports(s: &str) -> Result<Vec<ScannerReport>, ParseScannerReportsError> {
    fn convert_and_store(
        optional_scanner_line_buffer: &mut Option<String>,
        output: &mut Vec<ScannerReport>,
    ) -> Result<(), ParseScannerReportsError> {
        if let Some(scanner_line_buffer) = optional_scanner_line_buffer.take() {
            output.push(
                ScannerReport::from_str(&scanner_line_buffer).map_err(|error| {
                    ParseScannerReportsError::ScannerReportFromStr(scanner_line_buffer, error)
                })?,
            );
        }
        Ok(())
    }

    let mut output = Vec::new();
    let mut optional_scanner_line_buffer: Option<String> = None;
    for line in s.lines() {
        if line.starts_with("--- scanner ") {
            convert_and_store(&mut optional_scanner_line_buffer, &mut output)?;
            optional_scanner_line_buffer = Some(line.to_string());
        } else if let Some(scanner_line_buffer) = &mut optional_scanner_line_buffer {
            scanner_line_buffer.push_str("\r\n");
            scanner_line_buffer.push_str(line);
        }
    }
    convert_and_store(&mut optional_scanner_line_buffer, &mut output)?;
    Ok(output)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseScannerReportsError {
    #[error("Could not parse scanner report from string \"{0}\" ({1})")]
    ScannerReportFromStr(String, #[source] ScannerReportFromStrError),
}

#[derive(Debug, Clone)]
struct Scanner {
    id: ScannerId,
    position: AbsoluteScannerPosition,
    scanned_beacons: Vec<AbsoluteBeaconPosition>,
}

#[derive(Debug, Clone)]
struct ScannerReport {
    id: ScannerId,
    scanned_beacons: Vec<RelativeBeaconPosition>,
}

impl ScannerReport {
    fn into_scanner(self, rototranslation: &Rototranslation3D) -> Scanner {
        Scanner {
            id: self.id,
            position: AbsoluteScannerPosition(rototranslation.transform_point(&Point3D::origin())),
            scanned_beacons: self
                .scanned_beacons
                .into_iter()
                .map(|relative_beacon_position| {
                    relative_beacon_position.into_absolute(rototranslation)
                })
                .collect(),
        }
    }
}

impl FromStr for ScannerReport {
    type Err = ScannerReportFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<&str>>();
        Ok(Self {
            id: lines
                .get(0)
                .ok_or(ScannerReportFromStrError::MissingHeader)?
                .strip_prefix("--- scanner ")
                .ok_or(ScannerReportFromStrError::MissingHeader)?
                .split_whitespace()
                .collect::<Vec<&str>>()
                .get(0)
                .map(|value| {
                    value.parse::<u128>().map_err(|error| {
                        ScannerReportFromStrError::ParseHeaderNumber(value.to_string(), error)
                    })
                })
                .ok_or(ScannerReportFromStrError::MissingHeader)?
                .map(ScannerId)?,
            scanned_beacons: lines
                .into_iter()
                .skip(1)
                .filter(|line| !line.is_empty())
                .map(|line| {
                    RelativeBeaconPosition::from_str(line).map_err(|error| {
                        ScannerReportFromStrError::RelativeBeaconPointFromStr(
                            line.to_string(),
                            error,
                        )
                    })
                })
                .collect::<Result<Vec<RelativeBeaconPosition>, ScannerReportFromStrError>>()?,
        })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ScannerReportFromStrError {
    #[error("Missing header \"--- scanner x ---\"")]
    MissingHeader,
    #[error("Could not parse header number \"{0}\" ({1})")]
    ParseHeaderNumber(String, #[source] ParseIntError),
    #[error("Could not parse relative beacon point from string \"{0}\" ({1})")]
    RelativeBeaconPointFromStr(String, #[source] RelativeBeaconPointFromStrError),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct ScannerId(u128);

#[derive(Debug, Clone)]
struct RelativeBeaconPosition(Point3D);

impl RelativeBeaconPosition {
    fn into_absolute(self, rototranslation: &Rototranslation3D) -> AbsoluteBeaconPosition {
        AbsoluteBeaconPosition(rototranslation.transform_point(&self.0))
    }
}

impl FromStr for RelativeBeaconPosition {
    type Err = RelativeBeaconPointFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: [i16; 3] = s
            .split(',')
            .map(|line| {
                line.parse().map_err(|error| {
                    RelativeBeaconPointFromStrError::ParseInt(line.to_string(), error)
                })
            })
            .collect::<Result<Vec<i16>, RelativeBeaconPointFromStrError>>()?
            .try_into()
            .map_err(|v: Vec<i16>| {
                RelativeBeaconPointFromStrError::UnexpectedCountOfElements(s.to_string(), v.len())
            })?;
        Ok(Self(Point3D {
            x: parsed[0],
            y: parsed[1],
            z: parsed[2],
        }))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RelativeBeaconPointFromStrError {
    #[error("Could not parse \"{0}\" to integer ({1})")]
    ParseInt(String, #[source] ParseIntError),
    #[error("Unexpected count (is {1}, expected 3) of elements encountered in \"{0}\"")]
    UnexpectedCountOfElements(String, usize),
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AbsoluteBeaconPosition(Point3D);

impl Display for AbsoluteBeaconPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.0.x, self.0.y, self.0.z)
    }
}

#[derive(Debug, Clone)]
struct AbsoluteScannerPosition(Point3D);

fn find_rototranslation_for_b_with_12_fitting_beacons(
    absolute_beacon_points_a: &[AbsoluteBeaconPosition],
    relative_beacon_points_b: &[RelativeBeaconPosition],
) -> Option<Rototranslation3D> {
    let points_a = absolute_beacon_points_a
        .iter()
        .map(|absolute_beacon_point_a| absolute_beacon_point_a.0)
        .collect::<Vec<Point3D>>();
    let points_b = relative_beacon_points_b
        .iter()
        .map(|relative_beacon_point_b| relative_beacon_point_b.0)
        .collect::<Vec<Point3D>>();

    let rotations: [Rotation3D; 24] = (0..=3)
        .flat_map(|yaw| (0..=3).flat_map(move |pitch| (0..=3).map(move |roll| (yaw, pitch, roll))))
        .map(|(yaw, pitch, roll)| Rotation3D::from_euler_angles_90_degree(yaw, pitch, roll))
        .fold(Vec::new(), |mut output, next| {
            if !output.contains(&next) {
                output.push(next);
            }
            output
        })
        .try_into()
        .unwrap();

    for rotation in rotations {
        let rotated_points_b = points_b
            .iter()
            .map(|point_b| rotation.transform_point(point_b))
            .collect::<Vec<Point3D>>();

        for point_a in points_a.clone() {
            for rotated_point_b in rotated_points_b.clone() {
                let translation = point_a - rotated_point_b;

                let rototranslated_points_b = rotated_points_b
                    .iter()
                    .map(|rotated_point_b| *rotated_point_b + translation)
                    .collect::<Vec<Point3D>>();

                let mut found = 1;
                let mut cloned_points_a = points_a.clone();
                for rototranslated_point_b in rototranslated_points_b {
                    let prev = cloned_points_a.len();
                    cloned_points_a.retain(|point: &Point3D| *point != rototranslated_point_b);
                    if cloned_points_a.len() < prev {
                        if (prev - cloned_points_a.len()) != 1 {
                            eprintln!(
                                "Retaining deleted {} elements, but 1 was expected!",
                                prev - cloned_points_a.len()
                            );
                        }
                        found += 1;
                        if found >= 12 {
                            return Some(Rototranslation3D {
                                rotation,
                                translation: Translation3D {
                                    vector: translation,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
    None
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Vector3D {
    x: i16,
    y: i16,
    z: i16,
}

impl Vector3D {
    fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Point3D {
    x: i16,
    y: i16,
    z: i16,
}

impl Point3D {
    fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl Add<Vector3D> for Point3D {
    type Output = Point3D;

    fn add(self, rhs: Vector3D) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Point3D {
    type Output = Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Rototranslation3D {
    rotation: Rotation3D,
    translation: Translation3D,
}

impl Rototranslation3D {
    fn identity() -> Self {
        Self {
            rotation: Rotation3D::identity(),
            translation: Translation3D::identity(),
        }
    }

    fn transform_point(&self, point: &Point3D) -> Point3D {
        self.translation
            .transform_point(&self.rotation.transform_point(point))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Rotation3D {
    matrix: [[i16; 3]; 3],
}

impl Rotation3D {
    fn identity() -> Self {
        Self {
            matrix: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
        }
    }

    fn inner_sin(ypr: i8) -> i8 {
        match ypr % 4 {
            -3 => 1,
            -2 => 0,
            -1 => -1,
            0 => 0,
            1 => 1,
            2 => 0,
            3 => -1,
            _ => panic!(),
        }
    }

    fn inner_cos(ypr: i8) -> i8 {
        match ypr % 4 {
            -3 => 0,
            -2 => 1,
            -1 => 0,
            0 => 1,
            1 => 0,
            2 => -1,
            3 => 0,
            _ => panic!(),
        }
    }

    fn from_euler_angles_90_degree(yaw: i8, pitch: i8, roll: i8) -> Self {
        let sin_yaw = Self::inner_sin(yaw) as i16;
        let sin_pitch = Self::inner_sin(pitch) as i16;
        let sin_roll = Self::inner_sin(roll) as i16;
        let cos_yaw = Self::inner_cos(yaw) as i16;
        let cos_pitch = Self::inner_cos(pitch) as i16;
        let cos_roll = Self::inner_cos(roll) as i16;
        Self {
            matrix: [
                [
                    cos_yaw * cos_pitch,
                    cos_yaw * sin_pitch * sin_roll - sin_yaw * cos_roll,
                    cos_yaw * sin_pitch * cos_roll + sin_yaw * sin_roll,
                ],
                [
                    sin_yaw * cos_pitch,
                    sin_yaw * sin_pitch * sin_roll + cos_yaw * cos_roll,
                    sin_yaw * sin_pitch * cos_roll - cos_yaw * sin_roll,
                ],
                [-sin_pitch, cos_pitch * sin_roll, cos_pitch * cos_roll],
            ],
        }
    }

    fn transform_point(&self, point: &Point3D) -> Point3D {
        Point3D {
            x: self.matrix[0][0] * point.x
                + self.matrix[0][1] * point.y
                + self.matrix[0][2] * point.z,
            y: self.matrix[1][0] * point.x
                + self.matrix[1][1] * point.y
                + self.matrix[1][2] * point.z,
            z: self.matrix[2][0] * point.x
                + self.matrix[2][1] * point.y
                + self.matrix[2][2] * point.z,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Translation3D {
    vector: Vector3D,
}

impl Translation3D {
    fn identity() -> Self {
        Self {
            vector: Vector3D::origin(),
        }
    }

    fn transform_point(&self, point: &Point3D) -> Point3D {
        *point + self.vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_unique_detected_beacons() {
        // given
        let input = "--- scanner 0 ---\r\n404,-588,-901\r\n528,-643,409\r\n-838,591,734\r\n\
                            390,-675,-793\r\n-537,-823,-458\r\n-485,-357,347\r\n-345,-311,381\r\n\
                            -661,-816,-575\r\n-876,649,763\r\n-618,-824,-621\r\n553,345,-567\r\n\
                            474,580,667\r\n-447,-329,318\r\n-584,868,-557\r\n544,-627,-890\r\n\
                            564,392,-477\r\n455,729,728\r\n-892,524,684\r\n-689,845,-530\r\n\
                            423,-701,434\r\n7,-33,-71\r\n630,319,-379\r\n443,580,662\r\n\
                            -789,900,-551\r\n459,-707,401\r\n\r\n--- scanner 1 ---\r\n\
                            686,422,578\r\n605,423,415\r\n515,917,-361\r\n-336,658,858\r\n\
                            95,138,22\r\n-476,619,847\r\n-340,-569,-846\r\n567,-361,727\r\n\
                            -460,603,-452\r\n669,-402,600\r\n729,430,532\r\n-500,-761,534\r\n\
                            -322,571,750\r\n-466,-666,-811\r\n-429,-592,574\r\n-355,545,-477\r\n\
                            703,-491,-529\r\n-328,-685,520\r\n413,935,-424\r\n-391,539,-444\r\n\
                            586,-435,557\r\n-364,-763,-893\r\n807,-499,-711\r\n755,-354,-619\r\n\
                            553,889,-390\r\n\r\n--- scanner 2 ---\r\n649,640,665\r\n\
                            682,-795,504\r\n-784,533,-524\r\n-644,584,-595\r\n-588,-843,648\r\n\
                            -30,6,44\r\n-674,560,763\r\n500,723,-460\r\n609,671,-379\r\n\
                            -555,-800,653\r\n-675,-892,-343\r\n697,-426,-610\r\n578,704,681\r\n\
                            493,664,-388\r\n-671,-858,530\r\n-667,343,800\r\n571,-461,-707\r\n\
                            -138,-166,112\r\n-889,563,-600\r\n646,-828,498\r\n640,759,510\r\n\
                            -630,509,768\r\n-681,-892,-333\r\n673,-379,-804\r\n-742,-814,-386\r\n\
                            577,-820,562\r\n\r\n--- scanner 3 ---\r\n-589,542,597\r\n\
                            605,-692,669\r\n-500,565,-823\r\n-660,373,557\r\n-458,-679,-417\r\n\
                            -488,449,543\r\n-626,468,-788\r\n338,-750,-386\r\n528,-832,-391\r\n\
                            562,-778,733\r\n-938,-730,414\r\n543,643,-506\r\n-524,371,-870\r\n\
                            407,773,750\r\n-104,29,83\r\n378,-903,-323\r\n-778,-728,485\r\n\
                            426,699,580\r\n-438,-605,-362\r\n-469,-447,-387\r\n509,732,623\r\n\
                            647,635,-688\r\n-868,-804,481\r\n614,-800,639\r\n595,780,-596\r\n\r\n\
                            --- scanner 4 ---\r\n727,592,562\r\n-293,-554,779\r\n441,611,-461\r\n\
                            -714,465,-776\r\n-743,427,-804\r\n-660,-479,-426\r\n832,-632,460\r\n\
                            927,-485,-438\r\n408,393,-506\r\n466,436,-512\r\n110,16,151\r\n\
                            -258,-428,682\r\n-393,719,612\r\n-211,-452,876\r\n808,-476,-593\r\n\
                            -575,615,604\r\n-485,667,467\r\n-680,325,-822\r\n-627,-443,-432\r\n\
                            872,-547,-609\r\n833,512,582\r\n807,604,487\r\n839,-516,451\r\n\
                            891,-625,532\r\n-652,-548,-490\r\n30,-46,-14\r\n";

        // when
        let unique_detected_beacons = count_unique_detected_beacons(input);

        // then
        assert_eq!(unique_detected_beacons, Ok(79));
    }

    #[test]
    fn test_find_largest_manhattan_distance_between_any_two_scanners() {
        // given
        let input = "--- scanner 0 ---\r\n404,-588,-901\r\n528,-643,409\r\n-838,591,734\r\n\
                            390,-675,-793\r\n-537,-823,-458\r\n-485,-357,347\r\n-345,-311,381\r\n\
                            -661,-816,-575\r\n-876,649,763\r\n-618,-824,-621\r\n553,345,-567\r\n\
                            474,580,667\r\n-447,-329,318\r\n-584,868,-557\r\n544,-627,-890\r\n\
                            564,392,-477\r\n455,729,728\r\n-892,524,684\r\n-689,845,-530\r\n\
                            423,-701,434\r\n7,-33,-71\r\n630,319,-379\r\n443,580,662\r\n\
                            -789,900,-551\r\n459,-707,401\r\n\r\n--- scanner 1 ---\r\n\
                            686,422,578\r\n605,423,415\r\n515,917,-361\r\n-336,658,858\r\n\
                            95,138,22\r\n-476,619,847\r\n-340,-569,-846\r\n567,-361,727\r\n\
                            -460,603,-452\r\n669,-402,600\r\n729,430,532\r\n-500,-761,534\r\n\
                            -322,571,750\r\n-466,-666,-811\r\n-429,-592,574\r\n-355,545,-477\r\n\
                            703,-491,-529\r\n-328,-685,520\r\n413,935,-424\r\n-391,539,-444\r\n\
                            586,-435,557\r\n-364,-763,-893\r\n807,-499,-711\r\n755,-354,-619\r\n\
                            553,889,-390\r\n\r\n--- scanner 2 ---\r\n649,640,665\r\n\
                            682,-795,504\r\n-784,533,-524\r\n-644,584,-595\r\n-588,-843,648\r\n\
                            -30,6,44\r\n-674,560,763\r\n500,723,-460\r\n609,671,-379\r\n\
                            -555,-800,653\r\n-675,-892,-343\r\n697,-426,-610\r\n578,704,681\r\n\
                            493,664,-388\r\n-671,-858,530\r\n-667,343,800\r\n571,-461,-707\r\n\
                            -138,-166,112\r\n-889,563,-600\r\n646,-828,498\r\n640,759,510\r\n\
                            -630,509,768\r\n-681,-892,-333\r\n673,-379,-804\r\n-742,-814,-386\r\n\
                            577,-820,562\r\n\r\n--- scanner 3 ---\r\n-589,542,597\r\n\
                            605,-692,669\r\n-500,565,-823\r\n-660,373,557\r\n-458,-679,-417\r\n\
                            -488,449,543\r\n-626,468,-788\r\n338,-750,-386\r\n528,-832,-391\r\n\
                            562,-778,733\r\n-938,-730,414\r\n543,643,-506\r\n-524,371,-870\r\n\
                            407,773,750\r\n-104,29,83\r\n378,-903,-323\r\n-778,-728,485\r\n\
                            426,699,580\r\n-438,-605,-362\r\n-469,-447,-387\r\n509,732,623\r\n\
                            647,635,-688\r\n-868,-804,481\r\n614,-800,639\r\n595,780,-596\r\n\r\n\
                            --- scanner 4 ---\r\n727,592,562\r\n-293,-554,779\r\n441,611,-461\r\n\
                            -714,465,-776\r\n-743,427,-804\r\n-660,-479,-426\r\n832,-632,460\r\n\
                            927,-485,-438\r\n408,393,-506\r\n466,436,-512\r\n110,16,151\r\n\
                            -258,-428,682\r\n-393,719,612\r\n-211,-452,876\r\n808,-476,-593\r\n\
                            -575,615,604\r\n-485,667,467\r\n-680,325,-822\r\n-627,-443,-432\r\n\
                            872,-547,-609\r\n833,512,582\r\n807,604,487\r\n839,-516,451\r\n\
                            891,-625,532\r\n-652,-548,-490\r\n30,-46,-14\r\n";

        // when
        let largest_manhattan_distance_between_any_two_scanners =
            find_largest_manhattan_distance_between_any_two_scanners(input);

        // then
        assert_eq!(
            largest_manhattan_distance_between_any_two_scanners,
            Ok(3621)
        );
    }

    #[test]
    fn rotation3d_inner_sin_cos() {
        assert_eq!(Rotation3D::inner_sin(-3), 1);
        assert_eq!(Rotation3D::inner_sin(-2), 0);
        assert_eq!(Rotation3D::inner_sin(-1), -1);
        assert_eq!(Rotation3D::inner_sin(0), 0);
        assert_eq!(Rotation3D::inner_sin(1), 1);
        assert_eq!(Rotation3D::inner_sin(2), 0);
        assert_eq!(Rotation3D::inner_sin(3), -1);

        assert_eq!(Rotation3D::inner_cos(-3), 0);
        assert_eq!(Rotation3D::inner_cos(-2), 1);
        assert_eq!(Rotation3D::inner_cos(-1), 0);
        assert_eq!(Rotation3D::inner_cos(0), 1);
        assert_eq!(Rotation3D::inner_cos(1), 0);
        assert_eq!(Rotation3D::inner_cos(2), -1);
        assert_eq!(Rotation3D::inner_cos(3), 0);
    }

    #[test]
    fn rotation3d_from_euler_angles_90_degree() {
        assert_eq!(
            Rotation3D::from_euler_angles_90_degree(0, 0, 0).matrix,
            [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
        );
        assert_eq!(
            Rotation3D::from_euler_angles_90_degree(1, 0, 0).matrix,
            [[0, -1, 0], [1, 0, 0], [0, 0, 1]]
        );
    }

    #[test]
    fn rotation3d_transform_point() {
        assert_eq!(
            Rotation3D::from_euler_angles_90_degree(1, 0, 0).transform_point(&Point3D {
                x: 1,
                y: 0,
                z: 0
            }),
            Point3D { x: 0, y: 1, z: 0 }
        );
    }
}
