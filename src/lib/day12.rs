use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use thiserror::Error;

use super::{read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day12";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 12: Passage Pathing")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day12-input"), // TODO: download
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day12Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day12Error::ReadFileContents(input_file.map(str::to_string), error))?;
    let paths_count = count_paths_in_specific_way(&file_contents)?;
    println!(
        "There are {} paths through this cave system that visit small caves at once.",
        paths_count
    );
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day12Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not count paths in specific way ({0})")]
    CountPathsInSpecificWay(#[from] CountPathsInSpecificWayError),
}

pub fn count_paths_in_specific_way(rough_map: &str) -> Result<u128, CountPathsInSpecificWayError> {
    Ok(find_paths_in_specific_way(&RoughMap::from_str(rough_map)?)?.len() as u128)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CountPathsInSpecificWayError {
    #[error("Could not parse rough map ({0})")]
    RoughMapFromStr(#[from] RoughMapFromStrError),
    #[error("Could not find paths in specific way ({0})")]
    FindPathsInSpecificWay(#[from] FindPathsInSpecificWayError),
}

fn find_paths_in_specific_way(
    rough_map: &RoughMap,
) -> Result<Vec<MapPath>, FindPathsInSpecificWayError> {
    if !rough_map.vertices.contains(&"start".to_string()) {
        return Err(FindPathsInSpecificWayError::MissingStartVertex);
    }
    if !rough_map
        .edges
        .iter()
        .any(|(a, _)| a == &"start".to_string())
    {
        return Err(FindPathsInSpecificWayError::NoEdgeFromStart);
    }
    if !rough_map.vertices.contains(&"end".to_string()) {
        return Err(FindPathsInSpecificWayError::MissingEndVertex);
    }
    if !rough_map.edges.iter().any(|(_, b)| b == &"end".to_string()) {
        return Err(FindPathsInSpecificWayError::NoEdgeToEnd);
    }

    fn calculate_edges_path(
        current_edge: &(String, String),
        never_target_vertex: &str,
        target_vertex: &str,
        initial_path: &[(String, String)],
        rough_map: &RoughMap,
    ) -> Vec<Vec<(String, String)>> {
        if initial_path.contains(current_edge)
            || (current_edge.1.is_lowercase()
                && initial_path
                    .iter()
                    .any(|edge| current_edge.1 == edge.1 || current_edge.1 == never_target_vertex))
        {
            Vec::new()
        } else {
            let mut new_path = initial_path.to_owned();
            new_path.push(current_edge.clone());
            if current_edge.1 == target_vertex {
                vec![new_path]
            } else {
                rough_map
                    .edges
                    .iter()
                    .filter(|(s, _)| *s == current_edge.1)
                    .flat_map(|next_edge| {
                        calculate_edges_path(
                            next_edge,
                            never_target_vertex,
                            target_vertex,
                            &new_path.clone(),
                            rough_map,
                        )
                    })
                    .collect()
            }
        }
    }

    Ok(rough_map
        .edges
        .iter()
        .filter(|(s, _)| s.as_str() == "start")
        .flat_map(|next_edge| {
            calculate_edges_path(next_edge, "start", "end", &Vec::new(), rough_map)
        })
        .map(MapPath::from)
        .collect())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FindPathsInSpecificWayError {
    #[error("Missing \"start\" vertex")]
    MissingStartVertex,
    #[error("No edge from \"start\"")]
    NoEdgeFromStart,
    #[error("Missing \"end\" vertex")]
    MissingEndVertex,
    #[error("No edge to \"end\"")]
    NoEdgeToEnd,
}

#[derive(Debug, Eq, PartialEq)]
struct RoughMap {
    vertices: Vec<String>,
    edges: Vec<(String, String)>,
}

impl FromStr for RoughMap {
    type Err = RoughMapFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (vertices, edges) = s
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
            .map(|(lines_index, line)| {
                line.split('-')
                    .map(str::to_string)
                    .collect::<Vec<String>>()
                    .try_into()
                    .map_err(|error: Vec<String>| {
                        RoughMapFromStrError::InvalidCountOfLinePair(error.len(), lines_index)
                    })
            })
            .collect::<Result<Vec<[String; 2]>, RoughMapFromStrError>>()?
            .iter()
            .fold(
                (Vec::new(), Vec::new()),
                |(mut vertices, mut edges), line| {
                    if !vertices.contains(&line[0]) {
                        vertices.push(line[0].clone());
                    }
                    if !vertices.contains(&line[1]) {
                        vertices.push(line[1].clone());
                    }
                    let edge_a = (line[0].clone(), line[1].clone());
                    if !edges.contains(&edge_a) {
                        edges.push(edge_a);
                    }
                    let edge_b = (line[1].clone(), line[0].clone());
                    if !edges.contains(&edge_b) {
                        edges.push(edge_b);
                    }
                    (vertices, edges)
                },
            );
        Ok(Self { vertices, edges })
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RoughMapFromStrError {
    #[error("Found invalid count of pairs ({0}) in line no. {1}")]
    InvalidCountOfLinePair(usize, usize),
}

#[derive(Debug, Eq, PartialEq)]
struct MapPath(Vec<String>);

impl From<Vec<(String, String)>> for MapPath {
    fn from(edge_pairs: Vec<(String, String)>) -> Self {
        Self(
            edge_pairs
                .into_iter()
                .fold(Vec::new(), |mut map_path, edge| {
                    if map_path.is_empty() {
                        map_path.push(edge.0);
                    }
                    map_path.push(edge.1);
                    map_path
                }),
        )
    }
}

trait IsLowercase {
    fn is_lowercase(&self) -> bool;
}

impl IsLowercase for &str {
    fn is_lowercase(&self) -> bool {
        self == &self.to_lowercase().as_str()
    }
}

impl IsLowercase for String {
    fn is_lowercase(&self) -> bool {
        self.as_str().is_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_paths_in_specific_way_should_return_10() {
        // given
        let input = "start-A\r\nstart-b\r\nA-c\r\nA-b\r\nb-d\r\nA-end\r\nb-end";

        // when
        let count_of_paths = count_paths_in_specific_way(input);

        // then
        assert_eq!(count_of_paths, Ok(10));
    }

    #[test]
    fn count_paths_in_specific_way_should_return_19() {
        // given
        let input = "dc-end\r\nHN-start\r\nstart-kj\r\ndc-start\r\ndc-HN\r\nLN-dc\r\n\
                            HN-end\r\nkj-sa\r\nkj-HN\r\nkj-dc";

        // when
        let count_of_paths = count_paths_in_specific_way(input);

        // then
        assert_eq!(count_of_paths, Ok(19));
    }

    #[test]
    fn count_paths_in_specific_way_should_return_226() {
        // given
        let input = "fs-end\r\nhe-DX\r\nfs-he\r\nstart-DX\r\npj-DX\r\nend-zg\r\nzg-sl\r\n\
                            zg-pj\r\npj-he\r\nRW-he\r\nfs-DX\r\npj-RW\r\nzg-RW\r\nstart-pj\r\n\
                            he-WI\r\nzg-he\r\npj-fs\r\nstart-RW";

        // when
        let count_of_paths = count_paths_in_specific_way(input);

        // then
        assert_eq!(count_of_paths, Ok(226));
    }

    #[test]
    fn rough_map_from_str() {
        // given
        let input = "start-A\r\nstart-b\r\nA-c\r\nA-b\r\nb-d\r\nA-end\r\nb-end";

        // when
        let rough_map = RoughMap::from_str(input);

        // then
        assert_eq!(
            rough_map,
            Ok(RoughMap {
                vertices: vec![
                    "start".to_string(),
                    "A".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "end".to_string()
                ],
                edges: vec![
                    ("start".to_string(), "A".to_string()),
                    ("A".to_string(), "start".to_string()),
                    ("start".to_string(), "b".to_string()),
                    ("b".to_string(), "start".to_string()),
                    ("A".to_string(), "c".to_string()),
                    ("c".to_string(), "A".to_string()),
                    ("A".to_string(), "b".to_string()),
                    ("b".to_string(), "A".to_string()),
                    ("b".to_string(), "d".to_string()),
                    ("d".to_string(), "b".to_string()),
                    ("A".to_string(), "end".to_string()),
                    ("end".to_string(), "A".to_string()),
                    ("b".to_string(), "end".to_string()),
                    ("end".to_string(), "b".to_string()),
                ]
            })
        );
    }

    #[test]
    fn test_find_paths_in_specific_way() {
        // given
        let input = "start-A\r\nstart-b\r\nA-c\r\nA-b\r\nb-d\r\nA-end\r\nb-end";
        let rough_map = RoughMap::from_str(input).unwrap();

        // when
        let map_paths = find_paths_in_specific_way(&rough_map);

        // then
        let map_paths = map_paths.unwrap();
        assert_eq!(map_paths.len(), 10);
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "b".to_string(),
            "A".to_string(),
            "c".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "b".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "b".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "c".to_string(),
            "A".to_string(),
            "b".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "c".to_string(),
            "A".to_string(),
            "b".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "c".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "b".to_string(),
            "A".to_string(),
            "c".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "b".to_string(),
            "A".to_string(),
            "end".to_string()
        ])));
        assert!(map_paths.contains(&MapPath(vec![
            "start".to_string(),
            "b".to_string(),
            "end".to_string()
        ])));
    }
}
