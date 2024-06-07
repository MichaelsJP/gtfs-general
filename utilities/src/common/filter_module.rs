#[cfg(feature = "filter")]
pub mod filter_by {
    use std::error::Error;
    use std::num::NonZeroUsize;
    use std::path::PathBuf;

    use polars::datatypes::DataType;
    use polars::export::chrono::NaiveDate;
    use polars::prelude::{all, col, CsvWriterOptions, Expr, LazyCsvReader, LazyFileListReader, lit, SerializeOptions, Series, StrptimeOptions};

    use crate::common::file_module;
    use crate::common::file_module::ensure_header;

    /// Filter a file by values
    ///
    /// # Arguments
    ///
    /// * `file`: &PathBuf - File to filter
    /// * `output_folder`: &PathBuf - Folder to save the filtered file
    /// * `column_names`: Vec<&str> - Column names to filter
    /// * `data_types`: Vec<DataType> - Data types of the columns to convert to
    /// * `allowed_values`: &Series - Allowed values to filter by
    ///
    /// returns: Result<PathBuf, Box<dyn Error, Global>> - The path to the output file with the header

    pub fn filter_file_by_values(
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
        let csv_writer_options = CsvWriterOptions {
            include_bom: false,
            include_header: true,
            batch_size: NonZeroUsize::new(10000).unwrap(),
            maintain_order: true,
            ..Default::default()
        };
        LazyCsvReader::new(file)
            .with_has_header(true)
            .finish()?
            .filter(filter)
            .with_streaming(true)
            .sink_csv(output_file.clone(), csv_writer_options.clone())?;
        // Return path
        Ok(ensure_header(&file, &output_file)?)
    }

    /// Filter a file by dates
    ///
    /// # Arguments
    ///
    /// * `file_name`: &PathBuf - The file to filter
    /// * `output_folder`: &PathBuf - The folder to save the filtered file
    /// * `start_date`: &str - The start date to filter by
    /// * `end_date`: &str - The end date to filter by
    /// * `start_date_column`: &str - The column to filter by start date
    /// * `end_date_column`: &str - The column to filter by end date
    ///
    /// returns: Result<PathBuf, Box<dyn Error, Global>>
    ///
    pub fn filter_file_by_dates(
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
            batch_size: NonZeroUsize::new(10000).unwrap(),
            maintain_order: true,
            serialize_options,
        };

        // Create a lazy csv reader select start and end date and filter the minimum start date by using a boolean expression
        LazyCsvReader::new(file_name)
            .with_has_header(true)
            .with_low_memory(true)
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
        file_module::ensure_header(&file_name, &output_file)?;
        Ok(output_file)
    }
}

#[cfg(feature = "filter")]
pub mod filter_column {
    use std::error::Error;
    use std::path::PathBuf;

    use polars::datatypes::DataType;
    use polars::frame::DataFrame;
    use polars::prelude::{col, LazyCsvReader, LazyFileListReader, Series};

    /// Get a column from a csv file and format it to a definable type
    ///
    /// # Arguments
    ///
    /// * `file_path`: PathBuf - The file to get the column from
    /// * `column_name`: &str - The column to get
    /// * `data_type`: DataType - The data type to convert the column to
    ///
    /// returns: Result<Series, Box<dyn Error, Global>> - The column from the csv file
    ///
    // Write function to get a column from a csv file and format it to a definable type
    pub fn get_column(
        file_path: PathBuf,
        column_name: &str,
        data_type: DataType,
    ) -> Result<Series, Box<dyn Error>> {
        // Calls get_column with the file_name and column_name and data_type
        let df = get_columns(file_path, vec![column_name], vec![data_type])?;
        // Return column
        Ok(df.column(column_name).unwrap().clone())
    }

    /// Get multiple columns from a csv file and format them to definable types
    ///
    /// # Arguments
    ///
    /// * `file_path`: PathBuf - The file to get the columns from
    /// * `column_names`: Vec<&str> - The columns to get
    /// * `data_types`: Vec<DataType> - The data types to convert the columns to
    ///
    /// returns: Result<DataFrame, Box<dyn Error, Global>> - The columns from the csv file
    ///
    pub fn get_columns(
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
        let df = LazyCsvReader::new(file_path.clone())
            .with_low_memory(true)
            .with_has_header(true)
            .finish()?
            .select(columns)
            .with_streaming(true)
            .collect()?;
        // TODO sink to temp csv file
        // Return column
        Ok(df)
    }
}
