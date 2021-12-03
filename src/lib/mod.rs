use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;

use thiserror::Error;

pub mod day01;
pub mod day02;

fn read_file_contents(file_path: Option<&str>) -> Result<String, ReadFileContentsError> {
    let mut content = String::new();
    File::open(file_path.ok_or(ReadFileContentsError::MissingFilePath)?)
        .map_err(ReadFileContentsError::OpeningFile)?
        .read_to_string(&mut content)
        .map_err(ReadFileContentsError::ReadingFile)?;
    Ok(content)
}

#[derive(Debug, Error)]
pub enum ReadFileContentsError {
    #[error("Missing file path")]
    MissingFilePath,
    #[error("Failed opening file ({0})")]
    OpeningFile(#[source] IoError),
    #[error("Failed reading file ({0})")]
    ReadingFile(#[source] IoError),
}
