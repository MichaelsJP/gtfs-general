use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use log::{error, info};
use serde::Serialize;
use Command::Metadata;
use crate::gtfs::gtfs::GTFS;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// Set the log level
    #[arg(long, value_enum, global = true, default_value_t = LogLevel::Info)]
    pub level: LogLevel,

    /// Define the gtfs data location as a folder or zip file
    #[arg(short, long)]
    pub input_data: PathBuf,

    /// Define a custom working directory or use the default one
    /// The default working directory is named gtfs_general and is located in the current working directory
    #[arg(long, short, default_value = "./gtfs_general")]
    pub working_directory: PathBuf,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Command {
    /// Extract the metadata from a GTFS file. The subcommand can be called with `gtfs metadata` or `gtfs m`.
    Metadata {},
    /// Cut the gtfs data by a bounding box.
    ExtractBbox {
        /// Define the bounding box to extract the data from
        #[arg(short, long, num_args = 4, value_names = & ["minx", "miny", "maxx", "maxy"])]
        bbox: Vec<f64>,
    },
    // Hide the not implemented command
    #[command(hide = true)]
    NotImplemented {},
}

#[derive(
clap::ValueEnum, Clone, Default, Debug, Serialize, PartialEq
)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

pub struct GtfsGeneralResult {
    pub success: bool,
    pub message: String,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl fmt::Display for GlobalOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "level: {}, input_data: {:?}, working_directory: {:?}", self.level, self.input_data, self.working_directory)
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "global_opts: {:?} | command: {:?}", self.global_opts, self.command)
    }
}

impl App {
    pub fn exec(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Metadata {} => {
                info!("{} {} {}", "#".repeat(2), "Metadata".to_string(), "#".repeat(2));
                // Create a new GTFS object
                let gtfs = GTFS::new(self.global_opts.input_data.clone(), self.global_opts.working_directory.clone());
                assert!(gtfs.is_ok(), "Expected Ok, got Err: {:?}", gtfs);
                let gtfs = gtfs.unwrap();

                // Get the metadata
                let metadata = gtfs.get_metadata()?;
                // Print Metadata Results with info and # in front and back
                info!("Service Range: {:?}", metadata.service_range);
                // Print Metadata End
                info!("{} {} {}", "#".repeat(2), "Metadata End".to_string(), "#".repeat(2));
            }
            _ => {
                error!("Command not implemented yet");
                return Err(Box::from("Command not implemented yet"));
            }
        }
        Ok(())
    }
}