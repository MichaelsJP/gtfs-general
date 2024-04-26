use std::time::Instant;
use clap::Parser;
use log::{error, info};
use gtfs_general::command::{App, LogLevel};


fn init_logger(log_level: LogLevel) {
    env_logger::builder()
        .filter_level(match log_level {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warning => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
        })
        .init();
    // Print last message with println for the chosen log level in case its switched off or only errors are shown
    println!("Logger initialized with level {}", log_level);
}

fn main() {
    let exec_start_time = Instant::now();
    let app = App::parse();
    let log_level = LogLevel::clone(&app.global_opts.level);
    init_logger(log_level);
    // Print a nice header with multiple # to separate the outputs
    info!("{} {} {}", "#".repeat(10), "GTFS General".to_string(), "#".repeat(10));
    // Output the command line arguments
    info!("Command line arguments: {}", app);
    // Execute the command
    let processed_files = app.exec().unwrap_or_else(|e| {
        error!("Error executing command: {}", e);
        vec![]
    });
    info!("{} {} {}", "#".repeat(5), "Statistics".to_string(), "#".repeat(5));
    // Print the execution time
    info!("Execution time: {} seconds", exec_start_time.elapsed().as_secs());
    info!("{} {} {}", "#".repeat(10), "End".to_string(), "#".repeat(10));
}
