use std::path::PathBuf;

use clap::Parser;

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::fixture;
    use tempfile::tempdir;

    use gtfs_general::command::{App, LogLevel};
    use gtfs_general::command::Command::{ExtractBbox, ExtractDate};
    use utilities::testing::environment_module::setup_temp_gtfs_data;

    use super::*;

    // Create a fixture that returns a vector with default args ["test", "--input-data", "path/to/data"]
    #[fixture]
    fn default_args() -> Vec<String> {
        vec![
            String::from("test"),
            String::from("--input-data"),
            String::from("path/to/data"),
        ]
    }

    #[rstest::rstest]
    fn test_global_opts_defaults(default_args: Vec<String>) {
        // Create an App instance with the default args that are extended by the subcommand "metadata"
        let app = App::parse_from(&[&default_args[..], &vec![String::from("metadata")]].concat());
        assert_eq!(app.global_opts.level, LogLevel::Info);
        assert_eq!(app.global_opts.input_data, PathBuf::from("path/to/data"));
        assert_eq!(app.global_opts.working_directory, PathBuf::from("./"));

        // Test with short option for input-data and without default args
        let app = App::parse_from(
            &[
                &vec![
                    String::from("test"),
                    String::from("-i"),
                    String::from("path/to/short"),
                ][..],
                &vec![String::from("metadata")],
            ]
                .concat(),
        );
        assert_eq!(app.global_opts.level, LogLevel::Info);
        assert_eq!(app.global_opts.input_data, PathBuf::from("path/to/short"));
        assert_eq!(app.global_opts.working_directory, PathBuf::from("./"));
    }

    #[rstest::rstest]
    fn test_global_opts_custom_working_directory(default_args: Vec<String>) {
        // Create an App instance with the default args that are extended by the subcommand "metadata"
        let app = App::parse_from(
            &[
                &default_args[..],
                &vec![
                    String::from("--working-directory"),
                    String::from("path/to/working_directory"),
                    String::from("metadata"),
                ],
            ]
                .concat(),
        );
        assert_eq!(
            app.global_opts.working_directory,
            PathBuf::from("path/to/working_directory")
        );
        // With short option
        let app = App::parse_from(
            &[
                &default_args[..],
                &vec![
                    String::from("-w"),
                    String::from("path/to/working_directory"),
                    String::from("metadata"),
                ],
            ]
                .concat(),
        );
        assert_eq!(
            app.global_opts.working_directory,
            PathBuf::from("path/to/working_directory")
        );
    }

    #[rstest::rstest(
        args,
        log_level,
        case::test1(default_args(), LogLevel::Debug),
        case::test2(default_args(), LogLevel::Info),
        case::test3(default_args(), LogLevel::Warning),
        case::test4(default_args(), LogLevel::Error)
    )]
    fn test_logging(args: Vec<String>, log_level: LogLevel) {
        // Create an App instance with the default args that are extended by the subcommand "metadata"
        let app = App::parse_from(
            &[
                &args[..],
                &vec![
                    String::from("--level"),
                    log_level.to_string(),
                    String::from("metadata"),
                ],
            ]
                .concat(),
        );
        // Assert that the log level is info
        assert_eq!(app.global_opts.level, log_level);
    }

    #[rstest::rstest]
    fn test_subcommand_not_implemented(default_args: Vec<String>) {
        let app =
            App::parse_from(&[&default_args[..], &vec![String::from("not-implemented")]].concat());
        // Assert that the subcommand is not implemented
        assert_eq!(
            app.command,
            gtfs_general::command::Command::NotImplemented {}
        );
        let result = app.exec();
        // Assert that the result is an error
        assert!(result.is_err());
    }

    #[rstest::rstest]
    fn test_subcommand_metadata() {
        let temp_folder_valid = tempdir().expect("Failed to create temp folder");
        let temp_working_directory = tempdir().expect("Failed to create temp folder");

        setup_temp_gtfs_data(&temp_folder_valid).expect("Failed to setup temp gtfs data");

        // Construct default args
        let args = vec![
            String::from("test"),
            String::from("--input-data"),
            String::from(temp_folder_valid.path().to_str().unwrap()),
            String::from("--working-directory"),
            String::from(temp_working_directory.path().to_str().unwrap()),
            String::from("metadata"),
        ];

        let app = App::parse_from(args);
        // Assert that the subcommand is metadata
        assert_eq!(app.command, gtfs_general::command::Command::Metadata {});
        app.exec().expect("Failed to execute metadata command");
    }

    #[rstest::rstest(
        args,
        bbox,
        expected_bbox,
        case::test1(
            default_args(), vec_of_strings ! ["extract-bbox", "--bbox", "1.0", "2.0", "3.0", "4.0"], vec ! [1.0, 2.0, 3.0, 4.0]
        ),
        case::test2(
            default_args(), vec_of_strings ! ["extract-bbox", "--bbox", "4.0", "3.0", "2.0", "1.0"], vec ! [4.0, 3.0, 2.0, 1.0]
        ),
        case::test3(
            default_args(), vec_of_strings ! ["extract-bbox", "--bbox", "1.2", "2.3", "3.4", "4.5"], vec ! [1.2, 2.3, 3.4, 4.5]
        ),
    )]
    fn test_subcommand_extract_bbox(args: Vec<String>, bbox: Vec<String>, expected_bbox: Vec<f64>) {
        let mut args = args;
        args.extend(bbox);
        let app = App::parse_from(args);
        // Assert that the subcommand is extract-bbox
        assert_eq!(
            app.command,
            ExtractBbox {
                bbox: expected_bbox
            }
        );
        // let result = app.exec().expect("Failed to execute extract-bbox command");
    }

    #[rstest::rstest(
        args,
        date_query,
        expected_start_date,
        expected_end_date,
        case::test1(
            default_args(), vec_of_strings ! ["extract-date", "--start-date", "2020-01-01", "--end-date", "2020-01-31"], "2020-01-01", "2020-01-31"
        ),
    )]
    fn test_subcommand_extract_date(
        args: Vec<String>,
        date_query: Vec<String>,
        expected_start_date: String,
        expected_end_date: String,
    ) {
        // Arrange
        let temp_folder = tempdir().expect("Failed to create temp folder");
        let output_directory = tempdir().expect("Failed to create temp folder");
        setup_temp_gtfs_data(&temp_folder).expect("Failed to setup temp gtfs data");

        let mut args = args;
        // replace input-data with gtfs_zip_path
        args[2] = temp_folder.path().to_str().unwrap().to_string();

        args.extend(date_query);
        args.extend(vec![
            String::from("--output-folder"),
            output_directory.path().to_str().unwrap().to_string(),
        ]);
        let app = App::parse_from(args);
        // Assert that the subcommand is extract-date
        assert_eq!(
            app.command,
            ExtractDate {
                start_date: expected_start_date.to_string(),
                end_date: expected_end_date.to_string(),
                output_folder: PathBuf::from(output_directory.path().to_str().unwrap()),
            }
        );
        let result = app.exec().expect("Failed to execute extract-date command");
    }
}
