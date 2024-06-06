use crate::common::unzip_module::unzip_file;
use ::zip::ZipArchive;
use log::{debug, error, info};
use polars::export::chrono::NaiveDate;
use polars::prelude::*;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

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
            error!("File is not pointing to an existing file or folder: {:?}", file_location);
        }

        // If th working directory does not exist, create it
        if !working_directory.exists() {
            fs::create_dir_all(&working_directory).unwrap_or_else(|err| {
                error!("Error creating temporary folder: {}", err);
            });
        }
        Ok(gtfs)
    }

    pub fn is_valid(&self) -> Result<bool, Box<dyn Error>> {
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

    // Get names of all files in the file or folder
    pub fn get_filenames(&self) -> Result<Vec<String>, String> {
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

    // If a file from the zip is requested, it will be extracted to the temporary folder and the path to the file will be returned
    pub fn get_file(&self, file_name: &str) -> Result<PathBuf, Box<dyn Error>> {
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
    pub fn service_date_range(&self) -> Result<ServiceRange, Box<dyn Error>> {
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
            .has_header(true)
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
    pub(crate) fn get_metadata(&self) -> Result<Metadata, Box<dyn Error>> {
        let service_range = self.service_date_range()?;
        Ok(Metadata { service_range })
    }

    pub fn filter_file_by_dates(
        &self,
        file_name: &PathBuf,
        output_folder: &PathBuf,
        start_date: &str,
        end_date: &str,
        start_date_column: &str,
        end_date_column: &str,
    ) -> Result<PathBuf, Box<dyn Error>> {
        // Get the file name and add it to the output folder
        let output_file = output_folder.join(file_name.file_name().unwrap());

        // Cast start_date to a date object
        let start_date_converted = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;
        let end_date_converted = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?;
        let strptime_options = StrptimeOptions {
            format: Some("%Y%m%d".to_string()),
            ..Default::default()
        };
        let start_date_format = col(start_date_column)
            .cast(DataType::String)
            .str()
            .to_date(strptime_options.clone())
            .dt()
            .date();
        let end_date_format: Expr;
        let date_format_vector: Vec<Expr>;
        if start_date_column != end_date_column {
            end_date_format = col(end_date_column)
                .cast(DataType::String)
                .str()
                .to_date(strptime_options.clone())
                .dt()
                .date();
            date_format_vector = vec![start_date_format.clone(), end_date_format.clone()];
        } else {
            // Only one format expression, else the filter will fail
            date_format_vector = vec![start_date_format.clone()];
        }
        let serialize_options = SerializeOptions {
            date_format: Some("%Y%m%d".to_string()),
            ..Default::default()
        };
        let csv_writer_options = CsvWriterOptions {
            include_bom: false,
            include_header: true,
            batch_size: 10000,
            maintain_order: true,
            serialize_options,
        };

        // Create a lazy csv reader select start and end date and filter the minimum start date by using a boolean expression
        LazyCsvReader::new(file_name)
            .has_header(true)
            .low_memory(true)
            .finish()?
            // Select all and cast the start date and end date to a date object
            .select(&[all()])
            .with_columns(date_format_vector)
            .filter(
                col(start_date_column)
                    .gt_eq(lit(start_date_converted))
                    .and(col(end_date_column).lt_eq(lit(end_date_converted))),
            )
            .with_streaming(true)
            .sink_csv(output_file.clone(), csv_writer_options)?;
        Ok(output_file)
    }

    // Write function to get a column from a csv file and format it to a definable type
    pub fn get_column(
        &self,
        file_path: PathBuf,
        column_name: &str,
        data_type: DataType,
    ) -> Result<Series, Box<dyn Error>> {
        // Calls get_column with the file_name and column_name and data_type
        let df = self.get_columns(file_path, vec![column_name], vec![data_type])?;
        // Return column
        Ok(df.column(column_name).unwrap().clone())
    }

    pub fn get_columns(
        &self,
        file_path: PathBuf,
        column_names: Vec<&str>,
        data_types: Vec<DataType>,
    ) -> Result<DataFrame, Box<dyn Error>> {
        let mut columns = Vec::new();
        // Iterate through the column names and data types and create a vector of expressions and add it to columns
        for (column_name, data_type) in column_names.iter().zip(data_types.iter()) {
            columns.push(col(column_name).cast(data_type.clone()));
        }
        // Create a lazy csv reader
        let df = LazyCsvReader::new(file_path)
            .low_memory(true)
            .has_header(true)
            .finish()?
            .select(columns)
            .with_streaming(true)
            .collect()?;
        // Return column
        Ok(df)
    }

    pub fn filter_file_by_values(
        &self,
        file: &PathBuf,
        output_folder: &PathBuf,
        column_names: Vec<&str>,
        data_types: Vec<DataType>,
        allowed_values: &Series,
    ) -> Result<PathBuf, Box<dyn Error>> {
        // If file doesn't exist return Err
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file))?;
        }
        let output_file = output_folder.join(file.file_name().unwrap());
        let allowed = allowed_values.cast(&DataType::Int64).unwrap();
        let mut columns = Vec::new();
        let mut filter: Expr = Default::default();

        // Iterate through the column names and data types and create a vector of expressions and add it to columns filter
        for (column_name, data_type) in column_names.iter().zip(data_types.iter()) {
            let column = col(column_name).cast(data_type.clone());
            columns.push(column.clone());
            if column_name != &column_names[0] {
                filter = filter.and(column.is_in(lit(allowed.clone())));
            } else {
                filter = column.is_in(lit(allowed.clone()));
            };
        }
        let mut csv_writer_options = CsvWriterOptions {
            include_bom: false,
            include_header: true,
            batch_size: 10000,
            maintain_order: true,
            ..Default::default()
        };

        LazyCsvReader::new(file)
            .has_header(true)
            .finish()?
            .with_columns(columns.clone())
            .filter(filter)
            .with_streaming(true)
            .sink_csv(output_file.clone(), csv_writer_options.clone())?;
        // Check if file is empty
        if output_file.metadata()?.len() == 0 {
            // Get number of rows from file
            let mut df = LazyCsvReader::new(file)
                .has_header(true)
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
        // Return path
        Ok(output_file)
    }

    pub fn process_common_files(
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
        let filtered_routes_file = self.filter_file_by_values(
            &routes_file,
            output_folder,
            vec!["route_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("route_id")?,
        )?;
        file_paths.push(filtered_routes_file.clone());
        let filtered_shapes_file = self.filter_file_by_values(
            &shapes_file,
            output_folder,
            vec!["shape_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("shape_id")?,
        )?;
        file_paths.push(filtered_shapes_file);
        let filtered_stop_times_file = self.filter_file_by_values(
            &stop_times_file,
            output_folder,
            vec!["trip_id"],
            vec![DataType::Int64],
            route_trip_shape_ids_to_keep.column("trip_id")?,
        )?;
        file_paths.push(filtered_stop_times_file.clone());

        let agency_ids_to_keep =
            self.get_column(filtered_routes_file.clone(), "agency_id", DataType::Int64)?;
        let filtered_agency_file = self.filter_file_by_values(
            &agency_file,
            output_folder,
            vec!["agency_id"],
            vec![DataType::Int64],
            &agency_ids_to_keep,
        )?;
        file_paths.push(filtered_agency_file);

        // Filter stops file by stop_ids_to_keep
        let stop_ids_to_keep =
            self.get_column(filtered_stop_times_file, "stop_id", DataType::Int64)?;
        let filtered_stops_file = self.filter_file_by_values(
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
            self.filter_file_by_values(
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
            self.filter_file_by_values(
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
}
