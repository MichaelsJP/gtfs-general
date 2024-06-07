extern crate utilities;

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use ::zip::ZipArchive;
use log::{debug, error, info};
use polars::prelude::*;

use utilities::common::filter_module::filter_by::{filter_file_by_dates, filter_file_by_values};
use utilities::common::filter_module::filter_column::{get_column, get_columns};
use utilities::common::zip_module::unzip_file;

pub struct ServiceRange {
    pub start_date: String,
    pub latest_start_date: String,
    pub end_date: String,
}

impl fmt::Debug for ServiceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ start_date: {}, latest_start_date: {}, end_date: {} }}",
            self.start_date, self.latest_start_date, self.end_date
        )
    }
}

pub struct Metadata {
    pub service_range: ServiceRange,
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the service range and add a line break
        write!(f, "Metadata:\nService Range |{:?}", self.service_range)
    }
}

pub struct GTFS {
    pub file_location: PathBuf,
    pub working_directory: PathBuf,
}

impl fmt::Debug for GTFS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GTFS:\nfile_location: {:?}\nworking_directory: {:?}",
            self.file_location, self.working_directory
        )
    }
}

impl GTFS {
    // Constructor

    /// Create a new GTFS object
    ///
    /// # Arguments
    ///
    /// * `file_location`: PathBuf - Path to the GTFS file or folder
    /// * `working_directory`: PathBuf - Path to the working directory.
    /// If the GTFS file is a zip file, the files will be extracted to this directory.
    /// If the GTFS file is a folder, this parameter is overwritten with file_location.
    ///
    ///
    /// returns: Result<GTFS, Box<dyn Error>>
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use tempfile::tempdir;
    /// use gtfs_general::gtfs::gtfs::GTFS;
    /// use utilities::testing::environment_module::{get_gtfs_test_data_path, setup_temp_gtfs_data};
    ///
    /// let temp_folder = tempdir().expect("Failed to create temp folder");
    /// setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
    /// let gtfs = GTFS::new(temp_folder.path().to_path_buf(), temp_folder.path().to_path_buf());
    /// assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
    /// ```
    pub fn new(file_location: PathBuf, working_directory: PathBuf) -> Result<GTFS, Box<dyn Error>> {
        let mut gtfs = GTFS {
            file_location: file_location.clone(),
            working_directory: PathBuf::from(""),
        };
        // Check if the GTFS file is valid
        gtfs.is_valid()?;

        if file_location.is_file() {
            gtfs.working_directory = working_directory
                .clone()
                .join(file_location.file_stem().unwrap());
        } else if file_location.is_dir() {
            gtfs.working_directory = file_location.clone();
        } else {
            error!(
                "File is not pointing to an existing file or folder: {:?}",
                file_location
            );
        }

        // If th working directory does not exist, create it
        if !working_directory.exists() {
            fs::create_dir_all(&working_directory).unwrap_or_else(|err| {
                error!("Error creating temporary folder: {}", err);
            });
        }
        Ok(gtfs)
    }

    fn is_valid(&self) -> Result<bool, Box<dyn Error>> {
        // Get all file names
        let file_names = self.get_filenames()?;

        // Check if file names is empty
        if file_names.is_empty() {
            return Err(format!("No files found in folder {:?}", self.file_location))?;
        }

        // Create a vector of the required file names and check if they exist
        let required_files = vec![
            "agency.txt",
            "stops.txt",
            "routes.txt",
            "trips.txt",
            "stop_times.txt",
        ];
        for required_file in required_files {
            if !file_names.contains(&required_file.to_string()) {
                return Err(format!(
                    "Required file does not exist in GTFS data: {:?}",
                    required_file
                ))?;
            }
        }

        // Conditionally required files calendar and calendar_dates
        // Either calendar or calendar_dates must exist
        if !file_names.contains(&"calendar.txt".to_string())
            && !file_names.contains(&"calendar_dates.txt".to_string())
        {
            return Err(format!(
                "Either calendar.txt or calendar_dates.txt must exist in GTFS data: {:?}",
                self.file_location
            ))?;
        }
        // feed_info becomes required if translations doesn't exist
        if !file_names.contains(&"translations.txt".to_string())
            && !file_names.contains(&"feed_info.txt".to_string())
        {
            return Err(format!(
                "Either feed_info.txt or translations.txt must exist in GTFS data: {:?}",
                self.file_location
            ))?;
        }

        // Optional files fare_attributes, fare_rules, shapes, frequencies, transfers, pathways, levels, translations, attributions
        // Inform if optional files are missing
        let optional_files = vec![
            "fare_attributes.txt",
            "fare_rules.txt",
            "shapes.txt",
            "frequencies.txt",
            "transfers.txt",
            "pathways.txt",
            "levels.txt",
            "translations.txt",
            "attributions.txt",
        ];
        for optional_file in optional_files {
            if !file_names.contains(&optional_file.to_string()) {
                info!(
                    "Optional file does not exist in GTFS data: {:?}",
                    optional_file
                );
            }
        }

        // All valid
        Ok(true)
    }

    fn get_filenames(&self) -> Result<Vec<String>, String> {
        // Check if doesnt exist or isnt a folder or zip file
        if !self.file_location.exists() {
            Err(format!(
                "File or folder does not exist: {:?}",
                self.file_location
            ))?;
        } else if !self.file_location.is_dir()
            && self.file_location.extension().unwrap_or_default() != "zip"
        {
            Err(format!(
                "File is not a valid zip file or folder: {:?}",
                self.file_location
            ))?;
        }
        // Create file_names vector
        let file_names: Vec<String>;
        if self.file_location.is_dir() {
            debug!("Reading folder content: {:?}", self.file_location);
            // Iterate over the files in the folder and return the file names as a vector of strings.
            file_names = fs::read_dir(&self.file_location)
                .map_err(|err| format!("Error reading folder content: {}", err))?
                .map(|entry| entry.map_err(|err| format!("Error reading folder content: {}", err)))
                .filter(|entry| entry.as_ref().unwrap().path().is_file())
                .map(|entry| {
                    entry
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
                .collect();
        } else {
            debug!("Reading zip file content: {:?}", self.file_location);
            let file = fs::File::open(&self.file_location)
                .map_err(|err| format!("Error opening zip file: {}", err))?;

            let mut zip_file = ZipArchive::new(file)
                .map_err(|err| format!("Error reading zip file content | {}", err))?;

            // Read all files in the zip file and return the file names as a vector of strings
            file_names = (0..zip_file.len())
                .map(|i| zip_file.by_index(i).map(|file| file.name().to_string()))
                .collect::<Result<Vec<String>, _>>()
                .map_err(|err| format!("Error reading zip file content | {}", err))?;
            // debug number of found files
            debug!("Found {} files in zip file", file_names.len());
        }
        // Return error if no files were found
        if file_names.is_empty() {
            Err(format!("No files found in folder {:?}", self.file_location))?;
        }
        // Return the file names
        Ok(file_names)
    }

    fn get_file(&self, file_name: &str) -> Result<PathBuf, Box<dyn Error>> {
        // Check if file name is in get_filenames
        if !self.get_filenames()?.contains(&file_name.to_string()) {
            Err(format!(
                "File does not exist in GTFS data: {:?}",
                self.file_location
            ))?;
        }

        if self.file_location.join(file_name).exists() {
            return Ok(self.file_location.join(file_name));
        } else if self.working_directory.join(file_name).exists() {
            // Check if the file exists in temporary folder and is already extracted
            return Ok(self.working_directory.join(file_name));
        }

        // Extract from zip file
        let opened_zip_file = fs::File::open(&self.file_location)
            .map_err(|err| format!("Error opening zip file: {}", err))?;

        // Create zip file
        let mut opened_zip_file = ZipArchive::new(opened_zip_file)
            .map_err(|err| format!("Error reading zip file content | {}", err))?;

        // Get file pointer inside zip file
        let file_in_zip = opened_zip_file
            .by_name(file_name)
            .map_err(|err| format!("Error reading file from zip file: {}", err))?;

        let unzipped_file_path = unzip_file(file_in_zip, &self.working_directory)
            .map_err(|err| format!("Error extracting file from zip file: {}", err))?;

        // Return path to extracted file
        Ok(unzipped_file_path)
    }

    fn service_date_range(&self) -> Result<ServiceRange, Box<dyn Error>> {
        let calendar_file = self.get_file("calendar.txt");
        if calendar_file.is_err() {
            return Err(format!(
                "Error reading calendar file: {}",
                calendar_file.unwrap_err()
            ))?;
        }
        let start_date = col("start_date")
            .cast(DataType::String)
            .str()
            .to_date(StrptimeOptions {
                format: Some("%Y%m%d".to_string()),
                ..Default::default()
            })
            .dt()
            .date();
        let latest_start_date = col("start_date")
            .alias("latest_start_date")
            .cast(DataType::String)
            .str()
            .to_date(StrptimeOptions {
                format: Some("%Y%m%d".to_string()),
                ..Default::default()
            })
            .dt()
            .date();
        let end_date = col("end_date")
            .cast(DataType::String)
            .str()
            .to_date(StrptimeOptions {
                format: Some("%Y%m%d".to_string()),
                ..Default::default()
            })
            .dt()
            .date();
        // Create a lazy csv reader select start and end date and filter the minimum start date by using a boolean expression
        let lf = LazyCsvReader::new(calendar_file?)
            .with_has_header(true)
            .finish()?
            .select(&[
                start_date.clone().min(),
                latest_start_date.max(),
                end_date.max(),
            ]);

        let df = lf.with_streaming(true).collect()?;
        let start_date: String = df.column("start_date").unwrap().get(0).unwrap().to_string();
        let latest_start_date: String = df
            .column("latest_start_date")
            .unwrap()
            .get(0)
            .unwrap()
            .to_string();
        let end_date: String = df.column("end_date").unwrap().get(0).unwrap().to_string();
        Ok(ServiceRange {
            start_date,
            latest_start_date,
            end_date,
        })
    }

    fn process_common_files(
        &self,
        output_folder: &PathBuf,
        route_trip_shape_ids_to_keep: &DataFrame,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        // Required files
        let routes_file = self.get_file("routes.txt")?;
        let agency_file = self.get_file("agency.txt")?;
        let stop_times_file = self.get_file("stop_times.txt")?;
        let stops_file = self.get_file("stops.txt")?;

        // Optional files
        let shapes_file = self.get_file("shapes.txt").unwrap_or_default();
        let frequencies_file = self.get_file("frequencies.txt").unwrap_or_default();
        let transfers_file = self.get_file("transfers.txt").unwrap_or_default();
        let feed_info_file = self.get_file("feed_info.txt").unwrap_or_default();

        // Return vector with all file paths
        let mut file_paths: Vec<PathBuf> = vec![];

        // Filter files
        let filtered_routes_file = filter_file_by_values(
            &routes_file,
            output_folder,
            vec!["route_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("route_id")?,
        )?;
        file_paths.push(filtered_routes_file.clone());
        let filtered_shapes_file = filter_file_by_values(
            &shapes_file,
            output_folder,
            vec!["shape_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("shape_id")?,
        )?;
        file_paths.push(filtered_shapes_file);
        let filtered_stop_times_file = filter_file_by_values(
            &stop_times_file,
            output_folder,
            vec!["trip_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("trip_id")?,
        )?;
        file_paths.push(filtered_stop_times_file.clone());

        let agency_ids_to_keep =
            get_column(filtered_routes_file.clone(), "agency_id", DataType::Int64)?;
        let filtered_agency_file = filter_file_by_values(
            &agency_file,
            output_folder,
            vec!["agency_id"],
            vec![DataType::Int64],
            &agency_ids_to_keep,
        )?;
        file_paths.push(filtered_agency_file);

        // Filter stops file by stop_ids_to_keep
        let stop_ids_to_keep = get_column(filtered_stop_times_file, "stop_id", DataType::Int64)?;
        let filtered_stops_file = filter_file_by_values(
            &stops_file,
            output_folder,
            vec!["stop_id"],
            vec![DataType::Int64],
            &stop_ids_to_keep,
        )?;
        file_paths.push(filtered_stops_file);

        // Filter conditional files
        if frequencies_file.exists() {
            // Frequencies is an optional file
            filter_file_by_values(
                &frequencies_file,
                output_folder,
                vec!["trip_id"],
                vec![DataType::Int64],
                route_trip_shape_ids_to_keep.column("trip_id")?,
            )?;
            file_paths.push(frequencies_file);
        } else {
            info!("Frequencies file not found, skipping");
        }
        // Filter transfers file by stop_ids_to_keep the file is optional
        if transfers_file.exists() {
            filter_file_by_values(
                &transfers_file,
                output_folder,
                vec!["from_stop_id", "to_stop_id"],
                vec![DataType::Int64, DataType::Int64],
                &stop_ids_to_keep,
            )?;
            file_paths.push(transfers_file);
        } else {
            info!("Transfers file not found, skipping");
        }
        // Copy feed_info file to output folder
        if feed_info_file.exists() {
            fs::copy(
                feed_info_file.clone(),
                output_folder.join(feed_info_file.file_name().unwrap()),
            )?;
            file_paths.push(feed_info_file);
        } else {
            info!("Feed Info file not found, skipping");
        }
        Ok(file_paths)
    }

    /// Get metadata from the GTFS data
    ///
    /// returns: Result<Metadata, Box<dyn Error>>
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use tempfile::tempdir;
    /// use gtfs_general::gtfs::gtfs::GTFS;
    /// use utilities::testing::environment_module::{get_gtfs_test_data_path, setup_temp_gtfs_data};
    ///
    /// let temp_folder = tempdir().expect("Failed to create temp folder");
    /// setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
    /// let gtfs = GTFS::new(temp_folder.path().to_path_buf(), temp_folder.path().to_path_buf()).expect("Failed to create GTFS object");
    /// let metadata = gtfs.get_metadata().expect("Failed to get metadata");
    /// assert_eq!(metadata.service_range.start_date, "2022-10-02");
    /// assert_eq!(metadata.service_range.latest_start_date, "2022-10-08");
    /// assert_eq!(metadata.service_range.end_date, "2022-10-09");
    /// ```
    pub fn get_metadata(&self) -> Result<Metadata, Box<dyn Error>> {
        let service_range = self.service_date_range()?;
        Ok(Metadata { service_range })
    }

    /// Extract GTFS data by date
    ///
    /// # Arguments
    ///
    /// * `start_date`: &str - Start date in the format "YYYY-MM-DD"
    /// * `end_date`: &str - End date in the format "YYYY-MM-DD"
    /// * `output_folder`: &PathBuf - Path to the output folder
    ///
    /// returns: Result<Vec<PathBuf>, Box<dyn Error>>
    ///
    pub fn extract_by_date(
        &self,
        start_date: &str,
        end_date: &str,
        output_folder: &PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut processed_files: Vec<PathBuf> = vec![];

        // Get calendar file
        let calendar_file = self.get_file("calendar.txt")?;
        let calendar_dates_file = self.get_file("calendar_dates.txt")?;
        let trips_file = self.get_file("trips.txt")?;

        let filtered_calendar_file = filter_file_by_dates(
            &calendar_file,
            output_folder,
            start_date,
            end_date,
            "start_date",
            "end_date",
        )?;
        let filtered_calendar_dates_file = filter_file_by_dates(
            &calendar_dates_file,
            output_folder,
            start_date,
            end_date,
            "date",
            "date",
        )?;
        let mut service_ids_to_keep = get_column(
            filtered_calendar_file.clone(),
            "service_id",
            DataType::Int64,
        )?;
        let service_ids_to_keep = service_ids_to_keep.append(&get_column(
            filtered_calendar_dates_file.clone(),
            "service_id",
            DataType::Int64,
        )?)?;
        let filtered_trips_file = filter_file_by_values(
            &trips_file,
            output_folder,
            vec!["service_id"],
            vec![DataType::Int64],
            service_ids_to_keep,
        )?;
        let route_trip_shape_ids_to_keep = get_columns(
            filtered_trips_file.clone(),
            vec!["route_id", "trip_id", "shape_id"],
            vec![DataType::Int64, DataType::Int64, DataType::Int64],
        )?;
        let mut processed_common_files =
            self.process_common_files(output_folder, &route_trip_shape_ids_to_keep)?;
        processed_files.push(filtered_calendar_file);
        processed_files.push(filtered_calendar_dates_file);
        processed_files.push(filtered_trips_file);
        processed_files.append(&mut processed_common_files);
        // Return the file paths
        Ok(processed_files)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;

    use polars::datatypes::DataType;
    use polars::datatypes::DataType::Int32;
    use polars::frame::DataFrame;
    use polars::prelude::{NamedFrom, Series};
    use tempfile::tempdir;

    use utilities::common::filter_module::filter_by::filter_file_by_values;
    use utilities::common::filter_module::filter_column::{get_column, get_columns};
    use utilities::testing::environment_module::{
        check_file_content, get_gtfs_test_data_path, setup_temp_gtfs_data,
    };

    use crate::gtfs::gtfs::{GTFS, ServiceRange};

    #[test]
    fn test_is_valid() {
        let temp_folder = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf(),
            temp_folder.path().to_path_buf(),
        )
            .expect("Failed to create GTFS object");
        gtfs.is_valid().expect("Failed to validate GTFS data");
    }

    #[test]
    fn test_get_filenames() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf(),
            temp_folder.path().to_path_buf(),
        )
            .expect("Failed to create GTFS object");

        // Act
        let result = gtfs.get_filenames();

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let file_names = result.unwrap();
        assert_eq!(file_names.len(), 9);
        assert!(file_names.contains(&"agency.txt".to_string()));
        assert!(file_names.contains(&"calendar.txt".to_string()));
        assert!(file_names.contains(&"calendar_dates.txt".to_string()));
        assert!(file_names.contains(&"feed_info.txt".to_string()));
        assert!(file_names.contains(&"routes.txt".to_string()));
        assert!(file_names.contains(&"shapes.txt".to_string()));
        assert!(file_names.contains(&"stop_times.txt".to_string()));
        assert!(file_names.contains(&"stops.txt".to_string()));
        assert!(file_names.contains(&"trips.txt".to_string()));
    }

    #[test]
    fn test_get_file() {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");
        let gtfs = GTFS::new(
            temp_folder.path().to_path_buf(),
            temp_folder.path().to_path_buf(),
        )
            .expect("Failed to create GTFS object");

        // Act
        let result = gtfs.get_file("agency.txt");

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let file = result.unwrap();
        assert_eq!(file.file_name().unwrap(), "agency.txt");
        assert!(file.exists());
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
        pretty_assertions::assert_eq!(service_range.start_date, "2022-10-02");
        pretty_assertions::assert_eq!(service_range.latest_start_date, "2022-10-08");
        pretty_assertions::assert_eq!(service_range.end_date, "2022-10-09");
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
        pretty_assertions::assert_eq!(result.len(), 6); // should be 9 TODO
        for file in result.iter() {
            assert!(file.is_file());
        }
        // check routes.txt
        let routes_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "routes.txt")
            .expect("Failed to find routes.txt");
        let mut expected_lines = HashMap::new();
        expected_lines.insert(0, "route_long_name,route_short_name,agency_id,route_type,route_id");
        expected_lines.insert(1, "Intercity-Express,ICE 79,6,2,9");
        check_file_content(routes_file, expected_lines, 2);


        // check agency.txt
        let agency_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "agency.txt")
            .expect("Failed to find agency.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "agency_id,agency_name,agency_url,agency_timezone,agency_lang");
        expected_lines.insert(1, "6,DB Fernverkehr AG,https://www.bahn.de,Europe/Berlin,de");

        check_file_content(agency_file, expected_lines, 2);

        // check feed_info.txt
        let feed_info_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "feed_info.txt")
            .expect("Failed to find feed_info.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version,feed_contact_email,feed_contact_url");
        expected_lines.insert(1, "\"gtfs.de - GTFS für Deutschland, Daten bereitgestellt von DELFI e.V.\",http://gtfs.de,de,20211213,20221210,light-2022-10-02,info@gtfs.de,http://gtfs.de/de/feeds");
        check_file_content(feed_info_file, expected_lines, 2);

        // check stops.txt
        let stops_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "stops.txt")
            .expect("Failed to find stops.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "stop_name,stop_id,stop_lat,stop_lon");
        expected_lines.insert(1, "Aachen Hbf,318,50.7678,6.091499");
        expected_lines.insert(14, "Liège-Guillemins,915,50.62436,5.566483");
        check_file_content(stops_file, expected_lines, 15);


        // Check shapes.txt
        let shapes_file = result
            .iter()
            .find(|f| f.file_name().unwrap().to_str().unwrap() == "shapes.txt")
            .expect("Failed to find shapes.txt");
        expected_lines = HashMap::new();
        expected_lines.insert(0, "shape_id,shape_pt_sequence,shape_pt_lat,shape_pt_lon,shape_dist_traveled");
        expected_lines.insert(1, "10001,0,49.445,8.668,0.0");
        expected_lines.insert(2, "10001,1,49.445,8.668,54.527");
        expected_lines.insert(5, "10001,4,49.445,8.668,89.914");

        check_file_content(shapes_file, expected_lines, 6);
    }

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
        pretty_assertions::assert_eq!(gtfs.file_location, temp_folder_valid.path().to_path_buf());
        // Check working directory is the same as the temp_working_directory
        pretty_assertions::assert_eq!(
            gtfs.working_directory,
            temp_folder_valid.path().to_path_buf()
        );

        let result = gtfs.get_filenames();

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let filenames = result.unwrap();
        pretty_assertions::assert_eq!(filenames.len(), 9);
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
        let gtfs_zip_path = get_gtfs_test_data_path().expect("Failed to get gtfs test data path");
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
        pretty_assertions::assert_ne!(gtfs.working_directory, gtfs_zip_path);
        pretty_assertions::assert_eq!(
            gtfs.working_directory,
            non_existent_subfolder.clone().join("ic_ice_gtfs_germany")
        );

        // Act
        let result = gtfs.get_filenames();

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let filenames = result.unwrap();
        pretty_assertions::assert_eq!(filenames.len(), 9);
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
        let gtfs_zip_path = get_gtfs_test_data_path().expect("Failed to get gtfs test data path");
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
        pretty_assertions::assert_eq!(
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
        pretty_assertions::assert_eq!(file.file_name().unwrap().to_str().unwrap(), "stops.txt");
    }

    #[test]
    fn test_get_file_from_zip() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let gtfs_zip_path = get_gtfs_test_data_path().expect("Failed to get gtfs test data path");

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
        pretty_assertions::assert_eq!(file.file_name().unwrap().to_str().unwrap(), "stops.txt");
        assert!(file.is_file());
    }

    #[test]
    fn test_get_nonexistent_file_from_zip() {
        // Arrange
        let temp_working_directory = tempdir().expect("Failed to create temp folder");
        let gtfs_zip_path = get_gtfs_test_data_path().expect("Failed to get gtfs test data path");

        // Create Gtfs instance
        let gtfs = GTFS::new(gtfs_zip_path, temp_working_directory.path().to_path_buf());
        assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
        let gtfs = gtfs.unwrap();

        let non_existent_file = gtfs.get_file("foo.txt");
        assert!(non_existent_file.is_err(), "Expected Err, got Ok");
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
        let gtfs_zip_path = get_gtfs_test_data_path().expect("Failed to get gtfs test data path");
        // Remove write permission from the temp_working_directory
        let mut perms = fs::metadata(temp_working_directory.path())
            .unwrap()
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(temp_working_directory.path(), perms).unwrap();
        // Create Gtfs instance with healthy gtfs data but ask for non existent file
        let gtfs = GTFS::new(
            gtfs_zip_path,
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
        let result = get_column(calendar_file, "service_id", Int32);

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        pretty_assertions::assert_eq!(result.len(), 84);
        pretty_assertions::assert_eq!(result.get(0).unwrap().to_string(), "68");
        pretty_assertions::assert_eq!(result.get(1).unwrap().to_string(), "76");
        pretty_assertions::assert_eq!(result.iter().last().unwrap().to_string(), "86");
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
        let result = get_columns(
            calendar_file,
            vec!["service_id", "start_date"],
            vec![Int32, Int32],
        );

        // Assert
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        let result = result.unwrap();
        pretty_assertions::assert_eq!(result.iter().len(), 2);
        pretty_assertions::assert_eq!(result[0].len(), 84);
        pretty_assertions::assert_eq!(result[1].len(), 84);
        pretty_assertions::assert_eq!(result[0].get(0).unwrap().to_string(), "68");
        pretty_assertions::assert_eq!(result[0].get(1).unwrap().to_string(), "76");
        pretty_assertions::assert_eq!(result[0].iter().last().unwrap().to_string(), "86");
        pretty_assertions::assert_eq!(result[1].get(0).unwrap().to_string(), "20221002");
        pretty_assertions::assert_eq!(result[1].get(1).unwrap().to_string(), "20221002");
        pretty_assertions::assert_eq!(result[1].iter().last().unwrap().to_string(), "20221003");
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

        // Filter files
        let filtered_routes_file = filter_file_by_values(
            &routes_file,
            &temp_working_directory.path().to_path_buf(),
            vec!["route_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("route_id").unwrap(),
        );
        let mut expected_lines = HashMap::new();
        expected_lines.insert(0, "route_long_name,route_short_name,agency_id,route_type,route_id");
        check_file_content(&filtered_routes_file.unwrap(), expected_lines, 1);
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
