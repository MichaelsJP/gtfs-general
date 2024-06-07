#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use polars::prelude::DataType::Int32;
    use polars::prelude::Series;
    use tempfile::tempdir;

    use utilities::common::filter_module::filter_by::{filter_file_by_dates, filter_file_by_values};
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

    #[test]
    fn test_filter_file_by_values_when_the_file_doesnt_exist() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");

        // Act
        let allowed: Series = [1, 2].iter().collect();
        // Create face pathbuf
        let fake_path = PathBuf::from("fake_path");
        let result = filter_file_by_values(
            &fake_path,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id"],
            vec![Int32],
            &allowed,
        );
        // Assert error
        assert!(result.is_err(), "Expected Err, got Ok");
    }

    #[test]
    fn test_filter_file_by_values_with_single_column_name() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let test_files = setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let trips_file = test_files.iter().find(|file| file.file_name().unwrap() == "trips.txt").expect("Failed to find file");
        // Act
        let allowed: Series = [68, 76].iter().collect();
        let result = filter_file_by_values(
            &trips_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id"],
            vec![Int32],
            &allowed,
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let file_content = fs::read_to_string(result).expect("Failed to read file");
        // Assert length
        assert_eq!(file_content.lines().count(), 35);
        assert!(file_content.contains("route_id,service_id,direction_id,trip_id,shape_id"));
        assert!(file_content.contains("2,76,0,2564,"));
        assert!(file_content.contains("29,68,0,1980,"));
    }

    #[test]
    fn test_filter_file_by_values_with_multiple_column_name() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let test_files = setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let trips_file = test_files.iter().find(|file| file.file_name().unwrap() == "trips.txt").expect("Failed to find file");
        // Act
        let allowed: Series = [9, 68].iter().collect();
        let result = filter_file_by_values(
            &trips_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id", "route_id"],
            vec![Int32, Int32],
            &allowed,
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let file_content = fs::read_to_string(result).expect("Failed to read file");
        // Check trips.txt
        let max_line_number = file_content.lines().count();
        for line_number in 0..max_line_number {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            match line_number {
                0 => assert_eq!(line, "route_id,service_id,direction_id,trip_id,shape_id"),
                1 => assert_eq!(line, "9,68,0,1136,"),
                2 => assert_eq!(line, "9,68,0,114,"),
                3 => assert_eq!(line, "9,68,0,1855,"),
                4 => assert_eq!(line, "9,68,0,2539,"),
                _ => panic!("Unexpected line: {}", line),
            }
        }
    }
}