import pathlib
from typing import List

from _pytest._py.path import LocalPath
from typer.testing import CliRunner

from gtfs_general import main
from src.gtfs_general import __app_name__, __version__

runner = CliRunner()

script_path = pathlib.Path(__file__).parent.resolve()


def check_file_consistency(directory: LocalPath) -> None:
    output_files: List = [file for file in pathlib.Path(directory.__str__()).glob("*.txt")]
    assert len(output_files) == 9

    actual_files: List = [file.name for file in output_files]

    expected_files: List = [
        "stop_times.txt",
        "stops.txt",
        "trips.txt",
        "calendar.txt",
        "routes.txt",
        "feed_info.txt",
        "calendar_dates.txt",
        "agency.txt",
        "shapes.txt",
    ]

    assert len(actual_files) == len(expected_files)
    assert all([file in expected_files for file in actual_files])


def check_ic_ice_gtfs_germany_bbox_extraction_results(
    directory: LocalPath,
) -> None:
    output_files: List = [file for file in pathlib.Path(directory.__str__()).glob("*.txt")]

    for file in output_files:
        with open(file, "r") as fp:
            x = len(fp.readlines())
            if file.name == "stop_times.txt":
                assert x == 2234
            elif file.name == "stops.txt":
                assert x == 372
            elif file.name == "trips.txt":
                assert x == 147
            elif file.name == "calendar.txt":
                assert x == 21
            elif file.name == "routes.txt":
                assert x == 19
            elif file.name == "feed_info.txt":
                assert x == 2
            elif file.name == "calendar_dates.txt":
                assert x == 7
            elif file.name == "agency.txt":
                assert x == 2
            elif file.name == "shapes.txt":
                assert x == 6


def test_version() -> None:
    result = runner.invoke(main.app, ["--version"])
    assert result.exit_code == 0
    assert f"{__app_name__} v{__version__}\n" in result.stdout


def test_extract_by_bbox_with_file(tmpdir: LocalPath) -> None:
    test_gtfs_file: str = script_path.joinpath("../files/ic_ice_gtfs_germany.zip").__str__()

    result = runner.invoke(
        main.app,
        [
            "--logging",
            "INFO",
            "--no-progress",
            "extract-bbox",
            "--input-object",
            test_gtfs_file,
            "--output-folder",
            tmpdir.__str__(),
            "--bbox",
            "8.573179,49.352003,8.79405,49.459693",
        ],
    )
    assert result.exit_code == 0
    check_file_consistency(tmpdir)
    check_ic_ice_gtfs_germany_bbox_extraction_results(tmpdir)


def test_extract_by_bbox_with_folder(gtfs_test_folder: pathlib.Path, tmpdir: LocalPath) -> None:
    result = runner.invoke(
        main.app,
        [
            "--logging",
            "INFO",
            "--no-progress",
            "extract-bbox",
            "--input-object",
            gtfs_test_folder.__str__(),
            "--output-folder",
            tmpdir.__str__(),
            "--bbox",
            "8.573179,49.352003,8.79405,49.459693",
        ],
    )
    assert result.exit_code == 0

    check_file_consistency(tmpdir)
    check_ic_ice_gtfs_germany_bbox_extraction_results(tmpdir)


def test_get_metadata(gtfs_test_folder: pathlib.Path) -> None:
    result = runner.invoke(
        main.app,
        [
            "--logging",
            "INFO",
            "metadata",
            "--input-object",
            gtfs_test_folder.__str__(),
        ],
    )
    assert result.exit_code == 0
    assert "Service date window from '2022-10-02 00:00:00' to '2022-10-09 00:00:00'" in result.stdout


def test_filter_by_date(gtfs_test_folder: pathlib.Path, tmpdir: LocalPath) -> None:
    result = runner.invoke(
        main.app,
        [
            "--logging",
            "INFO",
            "--no-progress",
            "extract-date",
            "--input-object",
            gtfs_test_folder.__str__(),
            "--output-folder",
            tmpdir.__str__(),
            "--start-date",
            "20221002",
            "--end-date",
            "20221003",
        ],
    )
    assert result.exit_code == 0

    check_file_consistency(tmpdir)

    file: pathlib.PosixPath
    output_files: List = [file for file in pathlib.Path(tmpdir.__str__()).glob("*.txt")]

    for file in output_files:
        with open(file, "r") as fp:
            x = len(fp.readlines())
            if file.name == "stop_times.txt":
                assert x == 5539
            elif file.name == "stops.txt":
                assert x == 934
            elif file.name == "trips.txt":
                assert x == 540
            elif file.name == "calendar.txt":
                assert x == 2
            elif file.name == "routes.txt":
                assert x == 73
            elif file.name == "feed_info.txt":
                assert x == 2
            elif file.name == "calendar_dates.txt":
                assert x == 3
            elif file.name == "agency.txt":
                assert x == 10
            elif file.name == "shapes.txt":
                assert x == 6
