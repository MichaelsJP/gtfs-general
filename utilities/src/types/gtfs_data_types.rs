use std::sync::Arc;

use polars::enable_string_cache;
use polars::prelude::{DataType, Field, PolarsResult, Schema};

// Define the GtfsDtypes struct which will contain static methods for each GTFS schema
pub struct GTFSDataTypes;


impl GTFSDataTypes {
    pub fn modify_dtype(original_schema: &Schema, reference_schema: Arc<Schema>) -> PolarsResult<Schema> {
        let mut modified_schema = original_schema.clone();
        for field in original_schema.iter_fields() {
            if reference_schema.get(field.name()).is_some() {
                modified_schema.set_dtype(field.name(), reference_schema.get(field.name()).unwrap().clone());
            }
        }
        Ok(modified_schema)
    }
    /// Schema for GTFS Agency data
    pub fn agency() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("agency_id", DataType::String),
            Field::new("agency_name", DataType::String),
            Field::new("agency_url", DataType::String),
            Field::new("agency_timezone", DataType::String),
            Field::new("agency_lang", DataType::String),
            Field::new("agency_phone", DataType::String),
            Field::new("agency_fare_url", DataType::String),
            Field::new("agency_email", DataType::String),
        ])
    }

    /// Schema for GTFS Calendar Dates data
    pub fn calendar_dates() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("service_id", DataType::String),
            Field::new("date", DataType::String),
            Field::new("exception_type", DataType::Int64),
        ])
    }

    /// Schema for GTFS Calendar data
    pub fn calendar() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("monday", DataType::Int64),
            Field::new("tuesday", DataType::Int64),
            Field::new("wednesday", DataType::Int64),
            Field::new("thursday", DataType::Int64),
            Field::new("friday", DataType::Int64),
            Field::new("saturday", DataType::Int64),
            Field::new("sunday", DataType::Int64),
            Field::new("start_date", DataType::String),
            Field::new("end_date", DataType::String),
            Field::new("service_id", DataType::String),
        ])
    }

    /// Schema for GTFS Feed Info data
    pub fn feed_info() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("feed_publisher_name", DataType::String),
            Field::new("feed_publisher_url", DataType::String),
            Field::new("feed_lang", DataType::String),
            Field::new("default_lang", DataType::String),
            Field::new("feed_start_date", DataType::String),
            Field::new("feed_end_date", DataType::String),
            Field::new("feed_version", DataType::String),
            Field::new("feed_contact_email", DataType::String),
            Field::new("feed_contact_url", DataType::String),
        ])
    }

    /// Schema for GTFS Routes data
    pub fn routes() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("route_id", DataType::String),
            Field::new("agency_id", DataType::String),
            Field::new("route_short_name", DataType::String),
            Field::new("route_long_name", DataType::String),
            Field::new("route_desc", DataType::String),
            Field::new("route_type", DataType::Int64),
            Field::new("route_url", DataType::String),
            Field::new("route_color", DataType::String),
            Field::new("route_text_color", DataType::String),
            Field::new("route_sort_order", DataType::Int64),
            Field::new("continuous_pickup", DataType::Int64),
            Field::new("continuous_drop_off", DataType::Int64),
        ])
    }

    /// Schema for GTFS Stops data
    pub fn stops() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("stop_id", DataType::String),
            Field::new("stop_code", DataType::String),
            Field::new("stop_name", DataType::String),
            Field::new("stop_desc", DataType::String),
            Field::new("stop_lat", DataType::Float64),
            Field::new("stop_lon", DataType::Float64),
            Field::new("zone_id", DataType::String),
            Field::new("stop_url", DataType::String),
            Field::new("location_type", DataType::Int64),
            Field::new("parent_station", DataType::String),
            Field::new("stop_timezone", DataType::String),
            Field::new("wheelchair_boarding", DataType::Int64),
            Field::new("level_id", DataType::String),
            Field::new("platform_code", DataType::String),
        ])
    }

    /// Schema for GTFS Trips data
    pub fn trips() -> Schema {
        Schema::from_iter(vec![
            Field::new("route_id", DataType::String),
            Field::new("service_id", DataType::String),
            Field::new("trip_id", DataType::String),
            Field::new("trip_headsign", DataType::String),
            Field::new("trip_short_name", DataType::String),
            Field::new("direction_id", DataType::Int64),
            Field::new("block_id", DataType::String),
            Field::new("shape_id", DataType::String),
            Field::new("wheelchair_accessible", DataType::Int64),
            Field::new("bikes_allowed", DataType::Int64),
        ])
    }

    /// Schema for GTFS Stop Times data
    pub fn stop_times() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("trip_id", DataType::String),
            Field::new("arrival_time", DataType::String),
            Field::new("departure_time", DataType::String),
            Field::new("stop_id", DataType::String),
            Field::new("stop_sequence", DataType::Int64),
            Field::new("stop_headsign", DataType::String),
            Field::new("pickup_type", DataType::Int64),
            Field::new("drop_off_type", DataType::Int64),
            Field::new("continuous_pickup", DataType::Int64),
            Field::new("continuous_drop_off", DataType::Int64),
            Field::new("shape_dist_traveled", DataType::Float64),
            Field::new("timepoint", DataType::Int64),
        ])
    }

    /// Schema for GTFS Shapes data
    pub fn shapes() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("shape_id", DataType::String),
            Field::new("shape_pt_sequence", DataType::Int64),
            Field::new("shape_pt_lat", DataType::Float64),
            Field::new("shape_pt_lon", DataType::Float64),
            Field::new("shape_dist_traveled", DataType::Float64),
        ])
    }

    /// Schema for GTFS Frequencies data
    pub fn frequencies() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("trip_id", DataType::String),
            Field::new("start_time", DataType::String),
            Field::new("end_time", DataType::String),
            Field::new("headway_secs", DataType::Int64),
            Field::new("exact_times", DataType::Int64),
        ])
    }

    /// Schema for GTFS Transfers data
    pub fn transfers() -> Schema {
        // Set a global string cache
        enable_string_cache();
        Schema::from_iter(vec![
            Field::new("from_stop_id", DataType::String),
            Field::new("to_stop_id", DataType::String),
            Field::new("transfer_type", DataType::Int64),
            Field::new("min_transfer_time", DataType::Int64),
        ])
    }
}
