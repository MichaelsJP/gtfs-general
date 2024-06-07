#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;

    use utilities::common::file_module;
    use utilities::common::unzip_module;

    #[test]
    fn test_unzip_files() {
        // Create a temp folder
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a zip file in the temp folder with one test file
        let zip_file = temp_dir.path().join("test.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&zip_file).unwrap());
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("test1.txt", options).unwrap();
        zip.write_all(b"test1").unwrap();
        zip.start_file("test2.txt", options).unwrap();
        zip.write_all(b"test2").unwrap();
        zip.finish().unwrap();

        // Unzip the file
        let files = unzip_module::unzip_files(zip_file, temp_dir.path()).unwrap();
        // Assert that the file was extracted
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].file_name().unwrap(), "test1.txt");
        assert_eq!(files[1].file_name().unwrap(), "test2.txt");
    }

    #[test]
    fn test_ensure_header_when_output_file_empty() {
        // Create a temp folder
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a test file in the temp folder
        let original_file = temp_dir.path().join("test.csv");
        let mut file = std::fs::File::create(&original_file).unwrap();
        file.write_all(b"column1,column2\n").unwrap();
        // Assert "column1,column2" was written to the file
        let file_content = std::fs::read_to_string(&original_file).unwrap();
        assert_eq!(file_content, "column1,column2\n");

        // Create an empty file
        let output_file = temp_dir.path().join("empty.csv");
        std::fs::File::create(&output_file).unwrap();

        // Assert output file is empty
        assert_eq!(output_file.metadata().unwrap().len(), 0);

        // Ensure the header of the file
        let output_file_ensure_header =
            file_module::ensure_header(&original_file, &output_file).unwrap();
        // Assert that the output_file_ensure_header is the same as the output_file
        assert_eq!(output_file_ensure_header, output_file);
        // Assert that the output_file_ensure_header contains the header
        let file_content_ensure_header =
            std::fs::read_to_string(&output_file_ensure_header).unwrap();
        assert_eq!(file_content_ensure_header, "column1,column2\n");
    }

    #[test]
    fn test_ensure_header_when_output_file_not_empty() {
        // Create a temp folder
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a test file in the temp folder
        let original_file = temp_dir.path().join("test.csv");
        let mut file = std::fs::File::create(&original_file).unwrap();
        file.write_all(b"column1,column2\n").unwrap();
        // Assert "column1,column2" was written to the file
        let file_content = std::fs::read_to_string(&original_file).unwrap();
        assert_eq!(file_content, "column1,column2\n");

        // Create a file with a header
        let output_file = temp_dir.path().join("header.csv");
        let mut file = std::fs::File::create(&output_file).unwrap();
        file.write_all(b"header2,header3\n").unwrap();

        // Ensure the header of the file
        let output_file_ensure_header =
            file_module::ensure_header(&original_file, &output_file).unwrap();
        // Assert that the output_file_ensure_header is the same as the output_file
        assert_eq!(output_file_ensure_header, output_file);
        // Assert that the output_file_ensure_header contains the unchanged header
        let file_content_ensure_header =
            std::fs::read_to_string(&output_file_ensure_header).unwrap();
        assert_eq!(file_content_ensure_header, "header2,header3\n");
    }
}
