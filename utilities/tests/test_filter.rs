#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use utilities::common::filter_module::filter_file_by_dates;
    use utilities::testing::environment_module::setup_temp_gtfs_data;

    #[test]
    fn test_filter_file_by_different_date_columns() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let test_files = setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");

        let calendar_file = test_files.iter().find(|file| file.file_name().unwrap() == "calendar.txt").expect("Failed to find file");

        let result = filter_file_by_dates(
            &calendar_file,
            &temp_working_directory.path().to_path_buf().clone(),
            "2022-10-02",
            "2022-10-03",
            "start_date",
            "end_date",
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let file_content = fs::read_to_string(result).expect("Failed to read file");
        assert_eq!(file_content.lines().count(), 2);
        // Check that the file contains the expected lines
        assert!(file_content.contains("monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,service_id"));
        assert!(file_content.contains("1,0,0,0,0,0,1,20221002,20221003,46"));
    }

    #[test]
    fn test_filter_file_by_one_date_column() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let test_files = setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let calender_dates_file = test_files.iter().find(|file| file.file_name().unwrap() == "calendar_dates.txt").expect("Failed to find file");

        // Act
        let result = filter_file_by_dates(
            &calender_dates_file,
            &temp_working_directory.path().to_path_buf().clone(),
            "2022-10-02",
            "2022-10-03",
            "date",
            "date",
        );
        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let file_content = fs::read_to_string(result).expect("Failed to read file");
        assert_eq!(file_content.lines().count(), 3);
        // Check that the file contains the expected lines
        assert!(file_content.contains("service_id,exception_type,date"));
        assert!(file_content.contains("55,1,20221003"));
        assert!(file_content.contains("57,1,20221002"));
    }
}