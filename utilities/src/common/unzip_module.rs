use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::{Path, PathBuf};

use log::warn;
use zip::read::{ZipArchive, ZipFile};

#[cfg(feature = "default")]
pub fn unzip_files(zip_path: PathBuf, extract_to: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let zip_file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;
    let mut files: Vec<PathBuf> = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_name = unzip_file(file, extract_to);
        if let Ok(file_name) = file_name {
            files.push(file_name);
        } else {
            warn!("Error extracting file: {}", file_name.unwrap_err());
        }
    }
    Ok(files)
}

#[cfg(feature = "default")]
pub fn unzip_file(mut file: ZipFile, extract_to: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = file.name().to_string();
    let extract_path = extract_to.join(&file_name);
    let extract_path = Path::new(&extract_path);

    // Ensure the directory structure is created
    if let Some(parent) = extract_path.parent() {
        create_dir_all(parent)?;
    }

    // Extract the file
    let mut extracted_file = File::create(extract_path)?;
    copy(&mut file, &mut extracted_file)?;
    // Extract the complete file path
    Ok(extract_path.to_path_buf())
}
