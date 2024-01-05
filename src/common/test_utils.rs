use std::io::Error;
use std::path::PathBuf;
use tempfile::TempDir;
use crate::common::unzip_module::unzip_files;

pub fn setup_temp_gtfs_data(temporary_folder: &TempDir) -> Result<Vec<PathBuf>, Error> {
    let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
    let files: Vec<PathBuf> = unzip_files(gtfs_zip_path, temporary_folder.as_ref()).expect("Failed to unzip file");
    Ok(files)
}