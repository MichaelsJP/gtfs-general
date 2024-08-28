#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_unzip_files() {
        // Create a temp folder
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a zip file in the temp folder with one test file
        let zip_file = temp_dir.path().join("test.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&zip_file).unwrap());
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("test1.txt", options).unwrap();
        zip.write_all(b"test1").unwrap();
        zip.start_file("test2.txt", options).unwrap();
        zip.write_all(b"test2").unwrap();
        zip.finish().unwrap();

        // Unzip the file
        let files = gtfs_general::common::unzip_module::unzip_files(zip_file, temp_dir.path()).unwrap();
        // Assert that the file was extracted
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].file_name().unwrap(), "test1.txt");
        assert_eq!(files[1].file_name().unwrap(), "test2.txt");
    }
}
