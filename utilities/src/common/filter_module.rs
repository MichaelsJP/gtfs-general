#[cfg(feature = "filter")]
pub mod filter_by {
    use std::error::Error;
    use std::path::PathBuf;

    use log::debug;
    use polars::datatypes::DataType;
    use polars::datatypes::DataType::Categorical;
    use polars::enable_string_cache;
    use polars::export::chrono::NaiveDate;
    use polars::export::rayon::join;
    use polars::io::RowIndex;
    use polars::prelude::{all, col, CsvWriterOptions, Expr, LazyCsvReader, LazyFileListReader, lit, Schema, SerializeOptions, Series, StrptimeOptions};
    use polars::prelude::*;
    use polars::prelude::Expr::Exclude;
    use smartstring::SmartString;

    use crate::common::file_module;
    use crate::types::gtfs_data_types::GTFSDataTypes;

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
        expected_file_schema: Schema,
    ) -> Result<PathBuf, Box<dyn Error>> {
        // Set a global string cache
        enable_string_cache();


        // Ensure the file exists before proceeding
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file).into());
        }

        // Prepare the output file path
        let output_file = output_folder.join(file.file_name().ok_or("Invalid file name")?);

        // Cast allowed values to strings for filtering
        let mut allowed = allowed_values.cast(&DataType::String)?;
        allowed = allowed.cast(&Categorical(Default::default(), Default::default()))?;
        let mut columns = Vec::new();
        // Create empty filter expression
        let mut filter_expr: Expr = Default::default();

        // Iterate through the column names and data types and create a vector of expressions and add it to columns filter
        for (column_name, data_type) in column_names.iter().zip(data_types.iter()) {
            let column = col(column_name).cast(data_type.clone());
            columns.push(column.clone());
            if column_name != &column_names[0] {
                filter_expr = filter_expr.and(column.is_in(lit(allowed.clone())));
            } else {
                filter_expr = column.is_in(lit(allowed.clone()));
            };
        }


        // Prepare a Schema reference from the provided expected schema
        let schema_ref = Arc::new(expected_file_schema);

        // Print info with the input file, output file and the function name
        debug!("Filtering file: {:?} to {:?} in 'filter_file_by_values'", file, output_file);

        // Process the CSV file
        LazyCsvReader::new(file)
            .with_has_header(true)
            .with_low_memory(false)
            .with_schema_modify(|file_schema: Schema| GTFSDataTypes::modify_dtype(&file_schema, schema_ref.clone()))?
            .finish()?
            .filter(filter_expr)
            .with_streaming(false).collect()?;

        // Return the output file path, ensuring the header is included
        Ok(output_file.to_path_buf())
    }

    pub fn filter_file_by_values_df(
        file: &PathBuf,
        output_folder: &PathBuf,
        join_columns: Vec<&str>,
        mut join_df: DataFrame,
        expected_file_schema: Schema,
    ) -> Result<PathBuf, Box<dyn Error>> {
        enable_string_cache();
        // Ensure the file exists before proceeding
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file).into());
        }

        // Prepare the output file path
        let output_file = output_folder.join(file.file_name().ok_or("Invalid file name")?);


        let mut filter_expr: Expr = Default::default();
        let mut columns = Vec::new();

        // Iterate through the column names and data types and create a vector of expressions and add it to columns filter
        for i in 0..join_columns.len() {
            let column_name = join_columns[i];
            let column_data_type = expected_file_schema.get(column_name).expect("Column not found in schema");

            let column = col(column_name).cast(column_data_type.clone());
            columns.push(column.clone());
            if i != 0 {
                filter_expr = filter_expr.and(column.is_in(lit(join_df.column(column_name).unwrap().clone())));
            } else {
                filter_expr = column.is_in(lit(join_df.column(column_name).unwrap().clone()));
            };
        }

        // Create a series with boolean type and height of the allowed series
        // join_df.with_column(Series::new("allowed", vec![true; join_df.height()]))?;


        // Make sure the schema is correct
        let schema_ref = Arc::new(expected_file_schema);
        // Make a right join of the allowed_df and the file
        let mut ldf_input = LazyCsvReader::new(file)
            .with_has_header(true)
            .with_low_memory(false)
            .with_schema_modify(|file_schema: Schema| GTFSDataTypes::modify_dtype(&file_schema, schema_ref.clone()))?
            .finish()?
            .filter(filter_expr)
            .with_streaming(false)
            .sink_csv(output_file.clone(), CsvWriterOptions::default())?;

        // Write to file
        // let mut file = std::fs::File::create(&output_file).unwrap();
        // CsvWriter::new(&mut file).finish(&mut ldf_input).unwrap();

        Ok(output_file.to_path_buf())
    }


    pub fn filter_file_by_values_join(
        file: &PathBuf,
        output_folder: &PathBuf,
        join_columns: Vec<&str>,
        mut join_df: DataFrame,
        expected_file_schema: Schema,
    ) -> Result<PathBuf, Box<dyn Error>> {
        enable_string_cache();
        // Ensure the file exists before proceeding
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file).into());
        }

        // Prepare the output file path
        let output_file = output_folder.join(file.file_name().ok_or("Invalid file name")?);

        let mut columns = Vec::new();
        let mut all_columns: Vec<Expr> = Vec::new();

        // Iterate through the column names and data types and create a vector of expressions and add it to columns filter
        for column_name in join_columns.iter() {
            let column = col(column_name);
            columns.push(column.clone());
            all_columns.push(column.clone());
        }

        // Create a series with boolean type and height of the allowed series
        join_df.with_column(Series::new("allowed", vec![true; join_df.height()]))?;


        let row_index = RowIndex {
            name: Arc::from("id"),
            offset: 0,  // Start the row index from 10
        };

        // expected_file_schema.new_inserting_at_index(0, SmartString::from("id"), DataType::UInt32);

        // Make sure the schema is correct
        let schema_ref = Arc::new(expected_file_schema.clone());

        // Negate columns. Get all columns from expected_file_schema that are not in join_columns
        let mut negated_columns: Vec<&str> = Default::default();
        for field in expected_file_schema.get_names() {
            if !join_columns.contains(&field) {
                negated_columns.push(field.clone());
                all_columns.push(col(field).clone());
            }
        }
        // Add all negated strings and join_columns to columns as type col
        for column_name in negated_columns.iter() {
            let column = col(column_name);
            all_columns.push(column.clone());
        }
        for column_name in join_columns.iter() {
            let column = col(column_name);
            all_columns.push(column.clone());
        }

        // Make a right join of the allowed_df and the file
        let ldf_input = LazyCsvReader::new(file)
            .with_has_header(true)
            .with_low_memory(false)
            .with_schema_modify(|file_schema: Schema| GTFSDataTypes::modify_dtype(&file_schema, schema_ref.clone()))?
            .with_row_index(Some(row_index))
            .finish()?;
        let mut df = ldf_input.clone()
            .select(&[all().exclude(negated_columns)])
            .with_row_estimate(true)
            // .with_row_estimate(true)
            .join(join_df.lazy(), &columns, &columns, JoinArgs::from(JoinType::Left))
            .filter(col("allowed"))
            .select(&[col("id"), col("allowed")])
            .with_row_estimate(true)
            // Keep all rows from ldf_input that are in the first_filter column id
            .left_join(ldf_input, col("id"), col("id"))
            .filter(col("allowed"))
            .select(&[all().exclude(&["id", "allowed"])]).collect()?;
        // Write the output file
        let mut file = std::fs::File::create(&output_file).unwrap();
        CsvWriter::new(&mut file).finish(&mut df).unwrap();
        Ok(output_file.to_path_buf())
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
        expected_file_schema: Schema,
    ) -> Result<PathBuf, Box<dyn Error>> {
        // Derive the output file path
        let output_file = output_folder.join(file_name.file_name().ok_or("Invalid file name")?);

        // Parse the start and end dates
        let start_date_converted = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;
        let end_date_converted = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?;

        // Prepare a Schema reference from the provided expected schema
        let schema_ref = Arc::new(expected_file_schema);

        // Define date parsing options for the columns
        let strptime_options = StrptimeOptions {
            format: Some("%Y%m%d".to_string()),
            ..Default::default()
        };

        // Create date format expressions for start and end dates
        let start_date_expr = col(start_date_column)
            .cast(DataType::String)
            .str()
            .to_date(strptime_options.clone())
            .dt()
            .date();

        // Conditionally add end date column if it's different from the start date column
        let date_columns = if start_date_column != end_date_column {
            vec![
                start_date_expr.clone(),
                col(end_date_column)
                    .cast(DataType::String)
                    .str()
                    .to_date(strptime_options.clone())
                    .dt()
                    .date(),
            ]
        } else {
            vec![start_date_expr.clone()]
        };

        // CSV writer options
        let csv_writer_options = CsvWriterOptions {
            maintain_order: true,
            serialize_options: SerializeOptions {
                date_format: Some("%Y%m%d".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        // Log debug information
        debug!("Filtering file: {:?} to {:?} in 'filter_file_by_dates'", file_name, output_file);
        // Create a lazy CSV reader, apply schema modification, filter, and write to CSV
        LazyCsvReader::new(file_name)
            .with_has_header(true)
            .with_low_memory(false)
            .with_schema_modify(|file_schema| GTFSDataTypes::modify_dtype(&file_schema, schema_ref.clone()))?
            .finish()?
            .select(&[all()]) // Ensure all columns are selected
            .with_columns(date_columns)
            .filter(
                col(start_date_column)
                    .gt_eq(lit(start_date_converted))
                    .and(col(end_date_column).lt_eq(lit(end_date_converted))),
            )
            .with_streaming(false)
            .sink_csv(output_file.clone(), csv_writer_options)?;

        // Ensure header is included in the output file
        file_module::ensure_header(file_name, &output_file)?;

        Ok(output_file)
    }
}

#[cfg(feature = "filter")]
pub mod filter_column {
    use std::error::Error;
    use std::path::PathBuf;
    use std::sync::Arc;

    use log::debug;
    use polars::datatypes::DataType;
    use polars::enable_string_cache;
    use polars::frame::DataFrame;
    use polars::prelude::{col, LazyCsvReader, LazyFileListReader, Schema, Series};

    use crate::types::gtfs_data_types::GTFSDataTypes;

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
        expected_file_schema: Schema,
    ) -> Result<Series, Box<dyn Error>> {
        // Calls get_column with the file_name and column_name and data_type
        let df = get_columns(file_path, vec![column_name], vec![data_type], expected_file_schema)?;
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
        expected_file_schema: Schema,
    ) -> Result<DataFrame, Box<dyn Error>> {
        // Set a global string cache
        let mut columns = Vec::new();
        // Iterate through the column names and data types and create a vector of expressions and add it to columns
        for column_name in column_names.iter() {
            let schema = expected_file_schema.get(column_name).unwrap();
            columns.push(col(column_name).cast(schema.clone()));
        }
        let schema_ref = Arc::new(expected_file_schema);

        // Print info with the input file, output file and the function name
        debug!("Getting columns: {:?} from file: {:?}", column_names, file_path);

        // Create a lazy csv reader
        let df = LazyCsvReader::new(file_path.clone())
            .with_low_memory(false)
            .with_has_header(true)
            .with_schema_modify(|file_schema: Schema| GTFSDataTypes::modify_dtype(&file_schema, schema_ref.clone()))?
            .finish()?
            .select(&columns)
            .with_streaming(false)
            .collect()?;
        // TODO sink to temp csv file or return lazy
        // Return column
        Ok(df)
    }
}
