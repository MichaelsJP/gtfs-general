from __future__ import annotations

import errno
import os
import tempfile
import zipfile

from pathlib import Path
from typing import Any, Tuple, Dict

from dask import dataframe as ddf
import numpy as np

from src.gtfs_general import logger
from src.gtfs_general.exceptions.extractor_exceptions import GtfsIncompleteException
from src.gtfs_general.extractor.utils import parse_date_from_str


class GtfsDtypes:
    # Required
    agency: Dict = {
        "agency_id": np.str_,
        "agency_name": np.str_,
        "agency_url": np.str_,
        "agency_timezone": np.str_,
        "agency_lang": np.str_,
        "agency_phone": np.str_,
        "agency_fare_url": np.str_,
        "agency_email": np.str_,
    }
    calendar_dates: Dict = {"service_id": np.str_, "date": np.str_, "exception_type": "Int64"}
    calendar: Dict = {
        "monday": "Int64",
        "tuesday": "Int64",
        "wednesday": "Int64",
        "thursday": "Int64",
        "friday": "Int64",
        "saturday": "Int64",
        "sunday": "Int64",
        "start_date": np.str_,
        "end_date": np.str_,
        "service_id": np.str_,
    }
    feed_info: Dict = {
        "feed_publisher_name": np.str_,
        "feed_publisher_url": np.str_,
        "feed_lang": np.str_,
        "default_lang": np.str_,
        "feed_start_date": np.str_,
        "feed_end_date": np.str_,
        "feed_version": np.str_,
        "feed_contact_email": np.str_,
        "feed_contact_url": np.str_,
    }
    routes: Dict = {
        "route_id": np.str_,
        "agency_id": np.str_,
        "route_short_name": np.str_,
        "route_long_name": np.str_,
        "route_desc": np.str_,
        "route_type": "Int64",
        "route_url": np.str_,
        "route_color": np.str_,
        "route_text_color": np.str_,
        "route_sort_order": "Int64",
        "continuous_pickup": "Int64",
        "continuous_drop_off": "Int64",
    }
    stops: Dict = {
        "stop_id": np.str_,
        "stop_code": np.str_,
        "stop_name": np.str_,
        "stop_desc": np.str_,
        "stop_lat": np.float_,
        "stop_lon": np.float_,
        "zone_id": np.str_,
        "stop_url": np.str_,
        "location_type": "Int64",
        "parent_station": np.str_,
        "stop_timezone": np.str_,
        "wheelchair_boarding": "Int64",
        "level_id": np.str_,
        "platform_code": np.str_,
    }
    trips: Dict = {
        "route_id": np.str_,
        "service_id": np.str_,
        "trip_id": np.str_,
        "trip_headsign": np.str_,
        "trip_short_name": np.str_,
        "direction_id": "Int64",
        "block_id": np.str_,
        "shape_id": np.str_,
        "wheelchair_accessible": "Int64",
        "bikes_allowed": "Int64",
    }
    stop_times: Dict = {
        "trip_id": np.str_,
        "arrival_time": np.str_,
        "departure_time": np.str_,
        "stop_id": np.str_,
        "stop_sequence": "Int64",
        "stop_headsign": np.str_,
        "pickup_type": "Int64",
        "drop_off_type": "Int64",
        "continuous_pickup": "Int64",
        "continuous_drop_off": "Int64",
        "shape_dist_traveled": np.float_,
        "timepoint": "Int64",
    }

    # Optional
    shapes: Dict = {
        "shape_id": np.str_,
        "shape_pt_sequence": "Int64",
        "shape_pt_lat": np.float_,
        "shape_pt_lon": np.float_,
        "shape_dist_traveled": np.float_,
    }
    frequencies: Dict = {
        "trip_id": np.str_,
        "start_time": np.str_,
        "end_time": np.str_,
        "headway_secs": "Int64",
        "exact_times": "Int64",
    }
    transfers: Dict = {
        "from_stop_id": np.str_,
        "to_stop_id": np.str_,
        "transfer_type": "Int64",
        "min_transfer_time": "Int64",
    }


class GtfsFiles:
    # Required
    agency: Path
    calendar_dates: Path
    calendar: Path
    feed_info: Path
    routes: Path
    stop_times: Path
    stops: Path
    trips: Path

    # Optional - not complete
    _frequencies: Path | None = None
    _shapes: Path | None = None
    _transfers: Path | None = None

    @property
    def frequencies(self) -> Path:
        if self._frequencies is None:
            return Path("foo")
        return self._frequencies

    @property
    def shapes(self) -> Path:
        if self._shapes is None:
            return Path("foo")
        return self._shapes

    @property
    def transfers(self) -> Path:
        if self._transfers is None:
            return Path("foo")
        return self._transfers

    def set_files(self, file_path: Path) -> None:
        file_name: str = file_path.name
        if "agency" in file_name:
            self.agency = file_path
        elif "calendar_dates" in file_name:
            self.calendar_dates = file_path
        elif "calendar" in file_name:
            self.calendar = file_path
        elif "feed_info" in file_name:
            self.feed_info = file_path
        elif "routes" in file_name:
            self.routes = file_path
        elif "stop_times" in file_name:
            self.stop_times = file_path
        elif "stops" in file_name:
            self.stops = file_path
        elif "trips" in file_name:
            self.trips = file_path
        elif "frequencies" in file_name:
            self._frequencies = file_path
        elif "shapes" in file_name:
            self._shapes = file_path
        elif "transfers" in file_name:
            self._transfers = file_path
        else:
            logger.warn(f"Unknown file found: {file_path}")

    def required_is_complete(self) -> bool:
        return all(
            [
                self.agency,
                self.calendar_dates,
                self.calendar,
                self.feed_info,
                self.routes,
                self.stop_times,
                self.stops,
                self.trips,
            ]
        )


class GTFS:
    def __init__(self, input_object: Path, cpu_count: int | None = None, scheduler: str = "multiprocessing") -> None:
        self._input_folder: Path = input_object
        self._temporary_folder_context: Any[tempfile.TemporaryDirectory, None] = None
        self._gtfs_files: GtfsFiles = GtfsFiles()
        self._scheduler = scheduler
        self._cpu_count: int | None = cpu_count

        if input_object.is_file():
            input_object = self._extract_gtfs_file(input_object)
        if not input_object.exists():
            raise FileNotFoundError(errno.ENOENT, os.strerror(errno.ENOENT), input_object)
        for test in input_object.glob("*.txt"):
            self._gtfs_files.set_files(test)
        if not self._gtfs_files.required_is_complete():
            raise GtfsIncompleteException()

    def close(self) -> None:
        if isinstance(self._temporary_folder_context, tempfile.TemporaryDirectory):
            self._temporary_folder_context.cleanup()

    def __enter__(self) -> GTFS:
        return self

    def __exit__(self, type: object, value: object, traceback: object) -> None:
        self.close()

    def _extract_gtfs_file(self, input_file: Path) -> Path:
        self._temporary_folder_context = tempfile.TemporaryDirectory()
        extract_path: Path = Path(self._temporary_folder_context.name)
        if not input_file.suffix == ".zip":
            # TODO raise wrong file
            logger.error("Input path is a file but not a .zip file. Exiting.")
            raise Exception
        logger.info("Input is a .zip file. It will be extracted to a temporary location.")
        with zipfile.ZipFile(input_file, "r") as zip_ref:
            zip_ref.extractall(extract_path)
        return extract_path

    def service_date_range(self) -> Tuple:
        """
        Return the date range of the data set.
        """

        csv_chunks: ddf.DataFrame = ddf.read_csv(
            self._gtfs_files.calendar,
            usecols=["start_date", "end_date"],
            parse_dates=["start_date", "end_date"],
            date_parser=parse_date_from_str,
            low_memory=False,
        )
        xmin, xmax = ddf.compute(
            csv_chunks["start_date"].min(), csv_chunks["end_date"].max(), num_workers=self._cpu_count
        )
        return xmin.strftime("%Y-%m-%d %H:%M:%S"), xmax.strftime("%Y-%m-%d %H:%M:%S")
