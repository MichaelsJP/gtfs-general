use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

use tempfile::TempDir;

use crate::common::unzip_module::unzip_files;

#[cfg(feature = "testing")]
pub fn setup_temp_gtfs_data(temporary_folder: &TempDir) -> Result<Vec<PathBuf>, Error> {
    let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
    let files: Vec<PathBuf> =
        unzip_files(gtfs_zip_path, temporary_folder.as_ref()).expect("Failed to unzip file");
    Ok(files)
}

#[cfg(feature = "testing")]
pub fn check_file_content(
    file: &PathBuf,
    expected_lines: HashMap<usize, &str>,
    expected_size: usize,
) {
    let file_content = fs::read_to_string(file).expect("Failed to read file");
    // Check if the bounds of the file content are the same s the expected lines
    assert_eq!(file_content.lines().count(), expected_size);

    // Do it reversely and iterate over the expected lines to check if it is in the file content
    for (line_number, expected_line) in expected_lines.iter() {
        match file_content.lines().nth(*line_number) {
            Some(line) => assert_eq!(line, *expected_line),
            None => panic!("Expected line not found: {}", expected_line),
        }
    }
}
