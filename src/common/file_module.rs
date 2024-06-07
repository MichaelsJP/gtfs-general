use std::fs::File;
use std::path::{Path, PathBuf};
use polars::prelude::{CsvWriter, LazyCsvReader, LazyFileListReader, SerWriter};

// Write a function that takes an input file and output file path as an input and returns a Result<()>.
pub fn ensure_header(original_file: &PathBuf, output_file: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check if file is empty
    if output_file.metadata()?.len() == 0 {
        // Get number of rows from file
        let mut df = LazyCsvReader::new(original_file)
            .with_has_header(true)
            .with_n_rows(Some(0))
            .finish()?
            .collect()?;
        // Write the headers to file
        let mut output_file_ensured = File::create(&output_file)?;
        CsvWriter::new(&mut output_file_ensured)
            .include_header(true)
            .finish(&mut df)
            .unwrap();
    } 
    Ok(output_file.to_path_buf())
}