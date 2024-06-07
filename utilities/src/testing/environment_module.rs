use std::path::PathBuf;

pub fn get_gtfs_test_data_path() -> Result<std::path::PathBuf, std::io::Error> {
    let mut gtfs_zip_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    gtfs_zip_path.push("../resources/test/ic_ice_gtfs_germany.zip");
    Ok(gtfs_zip_path)
}

pub fn setup_temp_gtfs_data(temporary_folder: &tempfile::TempDir) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    // Get the path of this test file
    let gtfs_zip_path = get_gtfs_test_data_path()?;
    let files: Vec<std::path::PathBuf> =
        crate::common::zip_module::unzip_files(&gtfs_zip_path, &temporary_folder.path().to_path_buf()).expect("Failed to unzip file");
    Ok(files)
}

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
