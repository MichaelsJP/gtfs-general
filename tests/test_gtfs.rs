#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use polars::prelude::DataType::Int32;
    use polars::series::Series;
    use tempfile::{tempdir};
    use gtfs_general::gtfs::gtfs::{GTFS, ServiceRange};
    use pretty_assertions::{assert_eq, assert_ne};
    use gtfs_general::common::test_utils::setup_temp_gtfs_data;

    #[test]
    fn test_get_filenames_success_folder_and_no_working_directory() {
        let temp_folder_valid = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        // Create a sub folder
        let non_existent_subfolder = temp_working_directory.path().join("sub_folder");

        setup_temp_gtfs_data(&temp_folder_valid).expect("Failed to setup temp gtfs data");

        // Create Gtfs instance with new
        let gtfs = GTFS::new(temp_folder_valid.path().to_path_buf().clone(), non_existent_subfolder.clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Check file location is the same as the temp_folder_valid
        assert_eq!(gtfs.file_location, temp_folder_valid.path().to_path_buf());
        // Check working directory is the same as the temp_working_directory
        assert_eq!(gtfs.working_directory, non_existent_subfolder);

        let result = gtfs.get_filenames();

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let filenames = result.unwrap();
        assert_eq!(filenames.len(), 9);
        assert!(filenames.contains(&String::from("calendar_dates.txt")));
        assert!(filenames.contains(&String::from("feed_info.txt")));
        assert!(filenames.contains(&String::from("shapes.txt")));
        assert!(filenames.contains(&String::from("agency.txt")));
        assert!(filenames.contains(&String::from("routes.txt")));
        assert!(filenames.contains(&String::from("calendar.txt")));
        assert!(filenames.contains(&String::from("trips.txt")));
        assert!(filenames.contains(&String::from("stops.txt")));
        assert!(filenames.contains(&String::from("stop_times.txt")));
    }

    #[test]
    fn test_get_filenames_success_zip() {
        let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        // Create a sub folder
        let non_existent_subfolder = temp_working_directory.path().join("sub_folder");

        // Assert folder doesnt exist
        assert!(!non_existent_subfolder.is_dir());

        // Create Gtfs instance
        let gtfs = GTFS::new(gtfs_zip_path.clone(), non_existent_subfolder.clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Check temp_location is not the same as the gtfs_zip_path but an existing folder
        assert_ne!(gtfs.working_directory, gtfs_zip_path);
        assert_eq!(gtfs.working_directory, non_existent_subfolder.clone().join("ic_ice_gtfs_germany"));

        // Act
        let result = gtfs.get_filenames();

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let filenames = result.unwrap();
        assert_eq!(filenames.len(), 9);
        assert!(filenames.contains(&String::from("calendar_dates.txt")));
        assert!(filenames.contains(&String::from("feed_info.txt")));
        assert!(filenames.contains(&String::from("shapes.txt")));
        assert!(filenames.contains(&String::from("agency.txt")));
        assert!(filenames.contains(&String::from("routes.txt")));
        assert!(filenames.contains(&String::from("calendar.txt")));
        assert!(filenames.contains(&String::from("trips.txt")));
        assert!(filenames.contains(&String::from("stops.txt")));
        assert!(filenames.contains(&String::from("stop_times.txt")));
    }

    #[test]
    fn test_get_filenames_failure_nonexistent() {
        // Arrange
        let nonexistent_path = PathBuf::from("/nonexistent/path");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");

        // Create Gtfs instance and pass nonexistent path as a clone
        let gtfs = GTFS::new(nonexistent_path.clone(), temp_working_directory.path().to_path_buf().clone());

        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!("File or folder does not exist: {:?}", nonexistent_path)));
    }

    #[test]
    fn test_get_filenames_failure_invalid_zip() {
        // Create temp folder
        let temp_folder = tempdir().expect("Failed to create temp folder");
        // Create fake tar file
        let invalid_zip_path = temp_folder.path().join("invalid.tar");
        File::create(invalid_zip_path.clone()).expect("Failed to create file");

        // Create Gtfs instance
        let gtfs = GTFS::new(invalid_zip_path.clone(), temp_folder.path().to_path_buf().clone());
        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!("File is not a valid zip file or folder: {:?}", invalid_zip_path)));
    }

    #[test]
    fn test_get_filenames_failure_empty_folder() {
        // Create temp folder
        let temp_folder = tempdir().expect("Failed to create temp folder");

        // Create Gtfs instance
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_folder.path().to_path_buf().clone());

        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!("No files found in folder {:?}", temp_folder.path().to_path_buf())));
    }

    #[test]
    fn test_working_folder_creation() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
        // Create a sub folder
        let non_existent_subfolder = temp_working_directory.path().join("sub_folder");

        // Assert folder doesnt exist
        assert!(!non_existent_subfolder.is_dir());

        // Create Gtfs instance
        let gtfs = GTFS::new(gtfs_zip_path, non_existent_subfolder.clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Assert
        assert!(non_existent_subfolder.is_dir());
        assert!(temp_working_directory.path().is_dir());
        assert_eq!(gtfs.working_directory, non_existent_subfolder.join("ic_ice_gtfs_germany"));
    }

    #[test]
    fn test_get_file_from_folder() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");

        // Create Gtfs instance
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let result = gtfs.get_file("stops.txt");

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let file = result.unwrap();
        assert_eq!(file.file_name().unwrap().to_str().unwrap(), "stops.txt");
    }

    #[test]
    fn test_get_file_from_zip() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");

        // Create Gtfs instance
        let gtfs = GTFS::new(gtfs_zip_path, temp_working_directory.path().to_path_buf());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Check expected file doesnt exist yet
        assert!(!temp_working_directory.path().join("stops.txt").is_file());

        // Act
        let result = gtfs.get_file("stops.txt");

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let file = result.unwrap();
        assert_eq!(file.file_name().unwrap().to_str().unwrap(), "stops.txt");
        assert!(file.is_file());
    }

    #[test]
    fn test_get_nonexistent_file_from_zip() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let gtfs_zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");

        // Create Gtfs instance
        let gtfs = GTFS::new(gtfs_zip_path, temp_working_directory.path().to_path_buf());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        let non_existent_file = gtfs.get_file("foo.txt");
        assert!(non_existent_file.is_err(), "Expected Err, got Ok");
        let error_message = non_existent_file.err().unwrap();
        assert!(error_message.to_string().contains(&"File does not exist in GTFS data: \"tests/files/ic_ice_gtfs_germany.zip\"".to_string()))
    }

    #[test]
    fn test_get_nonexistent_file_from_folder() {
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        let non_existent_file = gtfs.get_file("foo.txt");
        assert!(non_existent_file.is_err(), "Expected Err, got Ok");
    }

    #[test]
    fn test_get_nonexistent_file_corrupt_zip() {
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let corrupt_zip_path = temp_folder.path().join("corrupt.zip");
        File::create(corrupt_zip_path.clone()).expect("Failed to create file");
        let gtfs = GTFS::new(corrupt_zip_path, temp_working_directory.path().to_path_buf().clone());

        // Assert assert gtfs is_err
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!("Error reading zip file content | invalid Zip archive: Invalid zip header")));
    }

    #[test]
    fn test_get_file_write_permission_denied() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
        // Remove write permission from the temp_working_directory
        let mut perms = fs::metadata(temp_working_directory.path()).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(temp_working_directory.path(), perms).unwrap();
        // Create Gtfs instance with healthy gtfs data but ask for non existent file
        let gtfs = GTFS::new(zip_path, temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        let non_existent_file = gtfs.get_file("stops.txt");
        assert!(non_existent_file.is_err(), "Expected Err, got Ok");
        let error_message = non_existent_file.err().unwrap();
        assert!(error_message.to_string().contains(&format!("Error extracting file from zip file: Permission denied (os error 13)")));
    }

    #[test]
    fn test_get_service_range() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let result = GTFS::service_date_range(&gtfs);

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let service_range: ServiceRange = result.unwrap();
        assert_eq!(service_range.start_date, "2022-10-02");
        assert_eq!(service_range.latest_start_date, "2022-10-08");
        assert_eq!(service_range.end_date, "2022-10-09");
    }

    #[test]
    fn test_filter_file_by_different_date_columns() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();
        // Act
        let result = gtfs.filter_file_by_dates(
            "calendar.txt",
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
        // Check that the file contains the expected lines
        assert!(file_content.contains("monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,service_id"));
        assert!(file_content.contains("1,0,0,0,0,0,1,20221002,20221003,46"));
    }

    #[test]
    fn test_filter_file_by_one_date_column() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let result = gtfs.filter_file_by_dates(
            "calendar_dates.txt",
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
        // Check that the file contains the expected lines
        assert!(file_content.contains("service_id,exception_type,date"));
        assert!(file_content.contains("55,1,20221003"));
        assert!(file_content.contains("57,1,20221002"));
    }

    #[test]
    fn test_get_column() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let calendar_file = gtfs.get_file("calendar.txt").expect("Failed to get file");
        let result = gtfs.get_column(calendar_file, "service_id", Int32);

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        assert_eq!(result.len(), 84);
        assert_eq!(result.get(0).unwrap().to_string(), "68");
        assert_eq!(result.get(1).unwrap().to_string(), "76");
        assert_eq!(result.iter().last().unwrap().to_string(), "86");
    }

    #[test]
    fn test_get_columns() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let calendar_file = gtfs.get_file("calendar.txt").expect("Failed to get file");
        let result = gtfs.get_columns(calendar_file, vec!["service_id", "start_date"], vec![Int32, Int32]);

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        assert_eq!(result[0].len(), 84);
        assert_eq!(result[1].len(), 84);
        assert_eq!(result[0].get(0).unwrap().to_string(), "68");
        assert_eq!(result[0].get(1).unwrap().to_string(), "76");
        assert_eq!(result[0].iter().last().unwrap().to_string(), "86");
        assert_eq!(result[1].get(0).unwrap().to_string(), "20221002");
        assert_eq!(result[1].get(1).unwrap().to_string(), "20221002");
        assert_eq!(result[1].iter().last().unwrap().to_string(), "20221003");
    }

    #[test]
    fn test_filter_file_by_values() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(temp_folder.path().to_path_buf().clone(), temp_working_directory.path().to_path_buf().clone());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let allowed: Series = [68, 76].iter().collect();
        let result = gtfs.filter_file_by_values("trips.txt", &temp_working_directory.path().to_path_buf(), "service_id", allowed);

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let file_content = fs::read_to_string(result).expect("Failed to read file");
        assert!(file_content.contains("route_id,service_id,direction_id,trip_id,shape_id"));
        assert!(file_content.contains("2,76,0,2564,"));
        assert!(file_content.contains("29,68,0,1980,"));
    }
}
