use std::error::Error;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use polars::datatypes::DataType;
use polars::export::chrono::NaiveDate;
use polars::prelude::{all, col, CsvWriterOptions, Expr, LazyCsvReader, LazyFileListReader, lit, SerializeOptions, Series, StrptimeOptions};

use crate::common::file_module;
use crate::common::file_module::ensure_header;

#[cfg(feature = "filter")]
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
    let mut csv_writer_options = CsvWriterOptions {
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