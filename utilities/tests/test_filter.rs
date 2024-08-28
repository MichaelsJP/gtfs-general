#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Instant;

    use polars::enable_string_cache;
    use polars::frame::DataFrame;
    use polars::prelude::{NamedFrom, Series};
    use polars::prelude::DataType::String;
    use tempfile::tempdir;

    use utilities::common::filter_module::filter_by::{filter_file_by_dates, filter_file_by_values_df, filter_file_by_values_join};
    use utilities::testing::environment_module::{check_file_content, setup_temp_gtfs_data};
    use utilities::types::gtfs_data_types::GTFSDataTypes;

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
            GTFSDataTypes::calendar(),
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
            GTFSDataTypes::calendar_dates(),
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
        let service_ids_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("service_id", &[10, 29, 29, 46, 26421]).cast(&String).expect(""),
        ]).expect("Failed to create dataframe");
        let result = filter_file_by_values_df(
            &fake_path,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id"],
            service_ids_to_keep,
            GTFSDataTypes::trips(),
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
        let route_trip_shape_ids_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("service_id", &[68, 76]).cast(&String).expect(""),
        ]).expect("Failed to create dataframe");
        let result = filter_file_by_values_df(
            &trips_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id"],
            route_trip_shape_ids_to_keep.clone(),
            GTFSDataTypes::trips(),
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let mut expected_lines = HashMap::new();
        expected_lines.insert(0, "route_id,service_id,direction_id,trip_id,shape_id");
        expected_lines.insert(1, "2,76,0,2564,");
        expected_lines.insert(2, "2,76,0,322,");
        expected_lines.insert(35, "91,76,0,834,");

        check_file_content(&result, expected_lines, 36);
    }

    fn test_filter_file_by_values_benchmark() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let test_files = setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let trips_file = test_files.iter().find(|file| file.file_name().unwrap() == "trips.txt").expect("Failed to find file");
        // pathbuf from /home/jules/HeiGIT/repos/gtfs-general/resources/220506_GTFS-Brosi/trips.txt
        let pathbuf = PathBuf::from("/home/jules/HeiGIT/repos/gtfs-general/resources/220506_GTFS-Brosi/trips.txt");
        // Act
        let allowed: Series = [9, 68].iter().collect();
        enable_string_cache();

        let mut times = Vec::new();
        // Benchmark filter_file_by_values_join
        println!("Benchmarking filter_file_by_values_join");

        let route_trip_shape_ids_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("route_id", &[1, 1, 1, 10, 86]).cast(&String).expect(""),
            Series::new("service_id", &[10, 29, 29, 46, 26421]).cast(&String).expect(""),
        ]).expect("Failed to create dataframe");
        // print dataframe
        // println!("{:?}", route_trip_shape_ids_to_keep);
        for _ in 0..1 {
            let iterator_time = Instant::now();
            let result = filter_file_by_values_join(
                &pathbuf,
                &temp_working_directory.path().to_path_buf(),
                vec!["service_id", "route_id"],
                route_trip_shape_ids_to_keep.clone(),
                GTFSDataTypes::trips(),
            );
            assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
            let iterator_end_time = iterator_time.elapsed();
            times.push(iterator_end_time);
        }

        // Get the total and average time in seconds
        let total_time: u128 = times.iter().map(|time| time.as_nanos()).sum();
        let average_time = total_time / times.len() as u128;
        // Print the total time by dividing with 1e+9
        println!("Total time: {:?} seconds", total_time as f64 / 1e+9);
        println!("Average time: {:?} seconds", average_time as f64 / 1e+9);


        // Benchmark filter_file_by_values_df
        times = Vec::new();
        println!("Benchmarking filter_file_by_values_df");

        for _ in 0..1 {
            let iterator_time = Instant::now();
            let result = filter_file_by_values_df(
                &pathbuf,
                &temp_working_directory.path().to_path_buf(),
                vec!["service_id", "route_id"],
                route_trip_shape_ids_to_keep.clone(),
                GTFSDataTypes::trips(),
            );
            assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
            let iterator_end_time = iterator_time.elapsed();
            times.push(iterator_end_time);
        }

        // Get the total and average time in seconds
        let total_time: u128 = times.iter().map(|time| time.as_nanos()).sum();
        let average_time = total_time / times.len() as u128;
        // Print the total time by dividing with 1e+9
        println!("Total time: {:?} seconds", total_time as f64 / 1e+9);
        println!("Average time: {:?} seconds", average_time as f64 / 1e+9);
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
        let service_and_route_id_to_keep: DataFrame = DataFrame::new(vec![
            Series::new("service_id", &[9, 68]).cast(&String).expect(""),
            Series::new("route_id", &[9, 68]).cast(&String).expect(""),
        ]).expect("Failed to create dataframe");

        let result = filter_file_by_values_df(
            &trips_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["service_id", "route_id"],
            service_and_route_id_to_keep.clone(),
            GTFSDataTypes::trips(),
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        // Check if the file exists
        assert!(result.is_file());

        let mut expected_lines = HashMap::new();
        expected_lines.insert(0, "route_id,service_id,direction_id,trip_id,shape_id");
        expected_lines.insert(1, "9,68,0,1136,");
        expected_lines.insert(2, "9,68,0,114,");
        expected_lines.insert(3, "9,68,0,1855,");
        expected_lines.insert(4, "9,68,0,2539,");
        expected_lines.insert(5, "9,68,0,2539_1,");

        check_file_content(&result, expected_lines, 6);
    }
}
