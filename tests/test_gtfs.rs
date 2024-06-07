#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;

    use polars::datatypes::DataType;
    use polars::frame::DataFrame;
    use polars::prelude::DataType::Int32;
    use polars::prelude::NamedFrom;
    use polars::series::Series;
    use pretty_assertions::{assert_eq, assert_ne};
    use tempfile::tempdir;

    use gtfs_general::gtfs::gtfs::{ServiceRange, GTFS};
    use utilities::testing::test_utils::{check_file_content, setup_temp_gtfs_data};

    #[test]
    fn test_get_filenames_success_folder_and_no_working_directory() {
        let temp_folder_valid = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        // Create a sub folder
        let non_existent_subfolder = temp_working_directory.path().join("sub_folder");

        setup_temp_gtfs_data(&temp_folder_valid).expect("Failed to setup temp gtfs data");

        // Create Gtfs instance with new
        let gtfs = GTFS::new(
            temp_folder_valid.path().to_path_buf().clone(),
            non_existent_subfolder.clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Check file location is the same as the temp_folder_valid
        assert_eq!(gtfs.file_location, temp_folder_valid.path().to_path_buf());
        // Check working directory is the same as the temp_working_directory
        assert_eq!(
            gtfs.working_directory,
            temp_folder_valid.path().to_path_buf()
        );

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
        assert_eq!(
            gtfs.working_directory,
            non_existent_subfolder.clone().join("ic_ice_gtfs_germany")
        );

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
        let gtfs = GTFS::new(
            nonexistent_path.clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );

        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!(
            "File or folder does not exist: {:?}",
            nonexistent_path
        )));
    }

    #[test]
    fn test_get_filenames_failure_invalid_zip() {
        // Create temp folder
        let temp_folder = tempdir().expect("Failed to create temp folder");
        // Create fake tar file
        let invalid_zip_path = temp_folder.path().join("invalid.tar");
        File::create(invalid_zip_path.clone()).expect("Failed to create file");

        // Create Gtfs instance
        let gtfs = GTFS::new(
            invalid_zip_path.clone(),
            temp_folder.path().to_path_buf().clone(),
        );
        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!(
            "File is not a valid zip file or folder: {:?}",
            invalid_zip_path
        )));
    }

    #[test]
    fn test_get_filenames_failure_empty_folder() {
        // Create temp folder
        let temp_folder = tempdir().expect("Failed to create temp folder");

        // Create Gtfs instance
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_folder.path().to_path_buf().clone(),
        );

        // Assert
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!(
            "No files found in folder {:?}",
            temp_folder.path().to_path_buf()
        )));
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
        assert_eq!(
            gtfs.working_directory,
            non_existent_subfolder.join("ic_ice_gtfs_germany")
        );
    }

    #[test]
    fn test_get_file_from_folder() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");

        // Create Gtfs instance
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
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
        assert!(error_message.to_string().contains(
            &"File does not exist in GTFS data: \"tests/files/ic_ice_gtfs_germany.zip\""
                .to_string()
        ))
    }

    #[test]
    fn test_get_nonexistent_file_from_folder() {
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
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
        let gtfs = GTFS::new(
            corrupt_zip_path,
            temp_working_directory.path().to_path_buf().clone(),
        );

        // Assert assert gtfs is_err
        assert!(gtfs.is_err(), "Expected Err, got Ok");
        let error_message = gtfs.err().unwrap();
        assert!(error_message.to_string().contains(&format!(
            "Error reading zip file content | invalid Zip archive: Invalid zip header"
        )));
    }

    #[test]
    fn test_get_file_write_permission_denied() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let zip_path = PathBuf::from("tests/files/ic_ice_gtfs_germany.zip");
        // Remove write permission from the temp_working_directory
        let mut perms = fs::metadata(temp_working_directory.path())
            .unwrap()
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(temp_working_directory.path(), perms).unwrap();
        // Create Gtfs instance with healthy gtfs data but ask for non existent file
        let gtfs = GTFS::new(
            zip_path,
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        let non_existent_file = gtfs.get_file("stops.txt");
        assert!(non_existent_file.is_err(), "Expected Err, got Ok");
        let error_message = non_existent_file.err().unwrap();
        assert!(error_message.to_string().contains(&format!(
            "Error extracting file from zip file: Permission denied (os error 13)"
        )));
    }

    #[test]
    fn test_get_service_range() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
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
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();
        // Act
        let calendar_file = gtfs.get_file("calendar.txt").expect("Failed to get file");
        let result = gtfs.filter_file_by_dates(
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
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let calendar_dates_file = gtfs
            .get_file("calendar_dates.txt")
            .expect("Failed to get file");
        let result = gtfs.filter_file_by_dates(
            &calendar_dates_file,
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
    fn test_get_column() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
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
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let calendar_file = gtfs.get_file("calendar.txt").expect("Failed to get file");
        let result = gtfs.get_columns(
            calendar_file,
            vec!["service_id", "start_date"],
            vec![Int32, Int32],
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        assert_eq!(result.iter().len(), 2);
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
    fn test_filter_file_by_values_that_doesnt_exist() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let allowed: Series = [1, 2].iter().collect();
        // Create face pathbuf
        let fake_path = PathBuf::from("fake_path");
        let result = gtfs.filter_file_by_values(
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
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let allowed: Series = [68, 76].iter().collect();
        let trips_file = gtfs.get_file("trips.txt").expect("Failed to get file");
        let result = gtfs.filter_file_by_values(
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
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Act
        let allowed: Series = [9, 68].iter().collect();
        let trips_file = gtfs.get_file("trips.txt").expect("Failed to get file");
        let result = gtfs.filter_file_by_values(
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

    #[test]
    fn test_empty_file_header() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();
        let route_trip_shape_ids_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("route_id", [0]),
            Series::new("service_id", [0]),
            Series::new("direction_id", [0]),
            Series::new("trip_id", [0]),
            Series::new("shape_id", [0]), // this causes an empty shapefile. We want to test for the header.
        ])
        .expect("Failed to create dataframe");

        let routes_file = gtfs.get_file("routes.txt").expect("Failed to get file");
        let shapes_file = gtfs.get_file("shapes.txt").expect("Failed to get file");

        // Filter files
        let filtered_routes_file = gtfs.filter_file_by_values(
            &routes_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["route_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("route_id").unwrap(),
        );
        let file_content =
            fs::read_to_string(filtered_routes_file.unwrap()).expect("Failed to read file");
        for line_number in 0..file_content.lines().count() {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            match line_number {
                0 => assert_eq!(
                    line,
                    "route_long_name,route_short_name,agency_id,route_type,route_id"
                ),
                _ => panic!("Unexpected line: {}", line),
            }
        }
    }

    #[test]
    fn test_process_common_files() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        // Create dataframe with the below data
        // route_id,service_id,direction_id,trip_id,shape_id
        // 9,68,0,1136,
        // 9,68,0,114,
        // 9,68,0,1855,
        // 9,68,0,2539,
        let route_trip_shape_ids_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("route_id", [9, 9, 9, 9]),
            Series::new("service_id", [68, 68, 68, 68]),
            Series::new("direction_id", [0, 0, 0, 0]),
            Series::new("trip_id", [1136, 114, 1855, 2539]),
            Series::new("shape_id", ["10001", "10001", "", ""]),
        ])
        .expect("Failed to create dataframe");
        let result = gtfs
            .process_common_files(
                &temp_working_directory.path().to_path_buf(),
                &route_trip_shape_ids_to_keep,
            )
            .expect("Failed to process common files");

        // First assert all files exist in result
        // Second find the file routes.txt and check the content
        assert_eq!(result.len(), 6); // should be 9 TODO
        for file in result.iter() {
            assert!(file.is_file());
        }
        // check routes.txt
        let routes_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "routes.txt")
            .expect("Failed to find routes.txt");
        let file_content = fs::read_to_string(routes_file).expect("Failed to read file");
        for line_number in 0..file_content.lines().count() {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            match line_number {
                0 => assert_eq!(
                    line,
                    "route_long_name,route_short_name,agency_id,route_type,route_id"
                ),
                1 => assert_eq!(line, "Intercity-Express,ICE 79,6,2,9"),
                _ => panic!("Unexpected line: {}", line),
            }
        }
        // check agency.txt
        // agency_id,agency_name,agency_url,agency_timezone,agency_lang
        // 6,DB Fernverkehr AG,https://www.bahn.de,Europe/Berlin,de
        let agency_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "agency.txt")
            .expect("Failed to find agency.txt");
        let file_content = fs::read_to_string(agency_file).expect("Failed to read file");
        for line_number in 0..file_content.lines().count() {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            match line_number {
                0 => assert_eq!(
                    line,
                    "agency_id,agency_name,agency_url,agency_timezone,agency_lang"
                ),
                1 => assert_eq!(
                    line,
                    "6,DB Fernverkehr AG,https://www.bahn.de,Europe/Berlin,de"
                ),
                _ => panic!("Unexpected line: {}", line),
            }
        }
        // check feed_info.txt
        let feed_info_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "feed_info.txt")
            .expect("Failed to find feed_info.txt");
        let file_content = fs::read_to_string(feed_info_file).expect("Failed to read file");
        // feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version,feed_contact_email,feed_contact_url
        file_content.contains("feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version,feed_contact_email,feed_contact_url");
        // check line two contains "gtfs.de - GTFS für Deutschland, Daten bereitgestellt von DELFI e.V."
        file_content
            .contains("gtfs.de - GTFS für Deutschland, Daten bereitgestellt von DELFI e.V.");
        for line_number in 0..file_content.lines().count() {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            match line_number {
                0 => assert_eq!(line, "feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version,feed_contact_email,feed_contact_url"),
                1 => assert!(line.contains("gtfs.de - GTFS für Deutschland, Daten bereitgestellt von DELFI e.V.")),
                _ => panic!("Unexpected line: {}", line),
            }
        }
        // check stops.txt
        // stop_name,stop_id,stop_lat,stop_lon
        // Aachen Hbf,318,50.7678,6.091499
        // last line Liège-Guillemins,915,50.62436,5.566483
        let stops_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "stops.txt")
            .expect("Failed to find stops.txt");
        let file_content = fs::read_to_string(stops_file).expect("Failed to read file");
        // Get max line number
        let max_line_number = file_content.lines().count();
        for line_number in 0..max_line_number {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            if line_number == 0 {
                assert_eq!(line, "stop_name,stop_id,stop_lat,stop_lon");
            } else if line_number == 1 {
                assert_eq!(line, "Aachen Hbf,318,50.7678,6.091499");
            } else if line_number == max_line_number - 1 {
                assert_eq!(line, "Liège-Guillemins,915,50.62436,5.566483");
            }
        }
        // Check stop_times.txt
        // trip_id,arrival_time,departure_time,stop_id,stop_sequence,pickup_type,drop_off_type
        // 1136,18:25:00,18:25:00,1334,0,,
        // last line 2539,22:13:00,22:13:00,1059,5,,
        let stop_times_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "stop_times.txt")
            .expect("Failed to find stop_times.txt");
        let file_content = fs::read_to_string(stop_times_file).expect("Failed to read file");
        // Get max line number
        let max_line_number = file_content.lines().count();
        for line_number in 0..max_line_number {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            if line_number == 0 {
                assert_eq!(line, "trip_id,arrival_time,departure_time,stop_id,stop_sequence,pickup_type,drop_off_type");
            } else if line_number == 1 {
                assert_eq!(line, "1136,18:25:00,18:25:00,1334,0,,");
            } else if line_number == max_line_number - 1 {
                assert_eq!(line, "2539,22:13:00,22:13:00,1059,5,,");
            }
        }

        // Check shapes.txt
        // shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence
        let shapes_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "shapes.txt")
            .expect("Failed to find shapes.txt");
        let file_content = fs::read_to_string(shapes_file).expect("Failed to read file");
        // Get max line number
        let max_line_number = file_content.lines().count();
        for line_number in 0..max_line_number {
            let line = file_content
                .lines()
                .nth(line_number)
                .expect("Failed to get line");
            if line_number == 0 {
                assert_eq!(
                    line,
                    "shape_id,shape_pt_sequence,shape_pt_lat,shape_pt_lon,shape_dist_traveled"
                );
            } else if line_number == 1 {
                assert_eq!(line, "10001,0,49.445,8.668,0.0");
            } else if line_number == 2 {
                assert_eq!(line, "10001,1,49.445,8.668,54.527");
            } else if line_number == max_line_number - 1 {
                assert_eq!(line, "10001,4,49.445,8.668,89.914");
            }
        }
        // // check calendar.txt
        // let calendar_file = result.iter().find(|f| f.file_name().unwrap().to_str().unwrap() == "calendar.txt").expect("Failed to find calendar.txt");
        // let file_content = fs::read_to_string(calendar_file).expect("Failed to read file");
        // // Get max line number
        // let max_line_number = file_content.lines().count();
        // for line_number in 0..max_line_number {
        //     let line = file_content.lines().nth(line_number).expect("Failed to get line");
        //     if line_number == 0 {
        //         assert_eq!(line, "monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,service_id");
        //     }
        // }
        // // Check calendar_dates.txt
        // let calendar_dates_file = result.iter().find(|f| f.file_name().unwrap().to_str().unwrap() == "calendar_dates.txt").expect("Failed to find calendar_dates.txt");
        // let file_content = fs::read_to_string(calendar_dates_file).expect("Failed to read file");
        // // Get max line number
        // let max_line_number = file_content.lines().count();
        // for line_number in 0..max_line_number {
        //     let line = file_content.lines().nth(line_number).expect("Failed to get line");
        //     if line_number == 0 {
        //         assert_eq!(line, "service_id,exception_type,date");
        //     }
        // }
    }

    #[test]
    fn test_extract_by_date_range() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf().clone(),
            temp_working_directory.path().to_path_buf().clone(),
        );
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();
        let start_date = "2022-10-02";
        let end_date = "2022-10-03";

        // Act
        let result = gtfs.extract_by_date(
            &start_date,
            &end_date,
            &temp_working_directory.path().to_path_buf().clone(),
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();

        // Assert that calendar file exists and has the expected content
        let calendar_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "calendar.txt")
            .expect("Failed to find calendar.txt");
        let mut expected_lines = HashMap::new();
        expected_lines.insert(0, "monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,service_id");
        expected_lines.insert(1, "1,0,0,0,0,0,1,20221002,20221003,46");

        check_file_content(calendar_file, expected_lines, 2);

        // Check Calendar Dates file exists and has the expected content
        let calendar_dates_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "calendar_dates.txt")
            .expect("Failed to find calendar_dates.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "service_id,exception_type,date");
        expected_lines.insert(1, "55,1,20221003");
        expected_lines.insert(2, "57,1,20221002");

        check_file_content(calendar_dates_file, expected_lines, 3);

        // Check trips file exists and has the expected content
        let trips_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "trips.txt")
            .expect("Failed to find trips.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "route_id,service_id,direction_id,trip_id,shape_id");
        expected_lines.insert(1, "1,55,0,1581,");
        expected_lines.insert(2, "1,55,0,2217,");
        expected_lines.insert(539, "97,46,0,2690,");

        check_file_content(trips_file, expected_lines, 540);
    }
}
