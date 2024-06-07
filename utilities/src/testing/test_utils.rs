#[cfg(feature = "testing")]
#[allow(unused_imports)]
pub fn setup_temp_gtfs_data(temporary_folder: &tempfile::TempDir) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let gtfs_zip_path = std::path::PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
    let files: Vec<std::path::PathBuf> =
        crate::common::unzip_module::unzip_files(gtfs_zip_path, temporary_folder.as_ref()).expect("Failed to unzip file");
    Ok(files)
}

#[cfg(feature = "testing")]
#[allow(unused_imports)]
pub fn check_file_content(
    file: &std::path::PathBuf,
    expected_lines: std::collections::HashMap<usize, &str>,
    expected_size: usize,
) {
    let file_content = std::fs::read_to_string(file).expect("Failed to read file");
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
