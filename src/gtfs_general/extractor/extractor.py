from __future__ import annotations

import csv
import errno
import os
import shutil
from datetime import datetime
import dask.dataframe as ddf
import pandas as pd
from pathlib import Path
from typing import List, Set, Union, Dict, Tuple

from tqdm.dask import TqdmCallback

from src.gtfs_general import logger
from src.gtfs_general.exceptions.extractor_exceptions import GtfsFileNotFound
from src.gtfs_general.extractor.bbox import Bbox
from src.gtfs_general.extractor.gtfs import GTFS, GtfsDtypes
from src.gtfs_general.extractor.utils import parse_date_from_str


class Extractor(GTFS):
    def __init__(
        self, input_object: Path, output_folder: Path, scheduler: str = "multiprocessing", cpu_count: int | None = None
    ) -> None:
        super().__init__(input_object, scheduler=scheduler, cpu_count=cpu_count)
        if not output_folder.exists():
            logger.debug(f"Creating output folder: {output_folder}")
            os.makedirs(output_folder)
        else:
            logger.warn("Output folder exists. Using it.")
        if not output_folder.exists():
            logger.error(f"Check access rights. Couldn't find and create the output folder {output_folder}")
            raise FileNotFoundError(errno.ENOENT, os.strerror(errno.ENOENT), output_folder)
        self._output_folder: Path = output_folder

    @staticmethod
    def __row_filter(rows: pd.DataFrame, ids: Set, columns: List) -> pd.DataFrame:
        for i in range(len(columns)):
            column: object = columns[i]
            if isinstance(column, int):
                columns[i] = rows.columns[columns]
        mask = rows[columns].isin(ids)
        for column in columns:
            rows[column].where(mask[column], inplace=True)
            rows = rows[rows[column].notna()]
        return rows

    @staticmethod
    def __filter_stops_by_bbox(rows: pd.DataFrame, bbox: Bbox) -> pd.DataFrame:
        mask: pd.Series = rows.apply(lambda row: bbox.contains(row["stop_lat"], row["stop_lon"]), axis=1)
        rows["stop_id"].where(mask, inplace=True)
        rows.dropna(inplace=True)
        return rows

    def __filter_rows_by_custom_column(
        self,
        file_path: Path | None,
        ids: Set,
        columns: List = None,
        usecols: List = None,
        dtype: Union[str, Dict] = "object",
        return_columns: List = None,
        write_out: bool = False,
        low_memory: bool = False,
    ) -> Tuple:
        if not file_path or not file_path.exists():
            raise GtfsFileNotFound(file_path=file_path.__str__())
        output_path = self._output_folder.joinpath(file_path.name)
        csv_chunks: ddf.DataFrame = ddf.read_csv(
            file_path, usecols=usecols, dtype=dtype, low_memory=low_memory, assume_missing=True
        )
        original_return_columns: List | None = return_columns
        if return_columns:
            return_columns = [column for column in return_columns if column in csv_chunks.columns]
        ddf_out: ddf.DataFrame = csv_chunks.map_partitions(self.__row_filter, ids=ids, columns=columns)
        if write_out:
            with TqdmCallback(desc=f"Filter {file_path.name}/Output to CSV"):
                ddf_out.compute(scheduler=self._scheduler, num_workers=self._cpu_count).to_csv(
                    output_path, index=False, doublequote=True, quoting=csv.QUOTE_ALL
                )
        if isinstance(return_columns, List) and len(return_columns) > 0:
            if write_out:
                ddf_out = ddf.read_csv(output_path, dtype=dtype, low_memory=low_memory, assume_missing=True)
            with TqdmCallback(desc="Load results", unit=" chunks"):
                results: pd.DataFrame = ddf_out[return_columns].compute(
                    scheduler=self._scheduler, num_workers=self._cpu_count
                )
            final_results: Tuple = tuple(set(results[column].dropna().tolist()) for column in results.keys())
            del results
            if isinstance(original_return_columns, List) and len(original_return_columns) > 0:
                missing_results: int = len(original_return_columns) - len(final_results)
                for _ in range(missing_results):
                    final_results = final_results + (set(),)
            return final_results
        return tuple()

    def _get_stops_in_bbox(self, bbox: Bbox) -> Set:
        csv_chunks: ddf.DataFrame = ddf.read_csv(
            self._gtfs_files.stops,
            usecols=["stop_id", "stop_lat", "stop_lon"],
            low_memory=False,
            dtype=GtfsDtypes.stops,
        )
        with TqdmCallback(desc="Filter stops by Bbox", unit=" chunks"):
            ddf_out = csv_chunks.map_partitions(self.__filter_stops_by_bbox, bbox=bbox)
            results: Set = set(
                ddf_out["stop_id"].compute(scheduler=self._scheduler, num_workers=self._cpu_count).dropna().to_list()
            )
        return results

    def _get_trips_of_stop_times(self, stop_ids_to_keep: Set) -> Set:
        return self.__filter_rows_by_custom_column(
            self._gtfs_files.stop_times,
            stop_ids_to_keep,
            usecols=["stop_id", "trip_id"],
            columns=["stop_id"],
            return_columns=["trip_id"],
            write_out=False,
            dtype=GtfsDtypes.stop_times,
        )[0]

    def _filter_trips_by_service_ids(self, service_ids_to_keep: Set) -> Tuple:
        return self.__filter_rows_by_custom_column(
            self._gtfs_files.trips,
            service_ids_to_keep,
            columns=["service_id"],
            return_columns=["route_id", "trip_id", "shape_id"],
            write_out=True,
            dtype=GtfsDtypes.trips,
        )

    def _filter_trips(self, trips_to_keep: Set) -> Tuple:
        return self.__filter_rows_by_custom_column(
            self._gtfs_files.trips,
            trips_to_keep,
            columns=["trip_id"],
            dtype=GtfsDtypes.trips,
            return_columns=["route_id", "service_id", "shape_id"],
            write_out=True,
        )

    def _filter_routes(self, routes_to_keep: Set) -> Set:
        return self.__filter_rows_by_custom_column(
            self._gtfs_files.routes,
            routes_to_keep,
            columns=["route_id"],
            return_columns=["agency_id"],
            write_out=True,
            dtype=GtfsDtypes.routes,
        )[0]

    def _filter_stop_times_using_trips(self, trip_ids_to_keep: Set) -> Set:
        logger.info("Filter stop_times.txt")
        return self.__filter_rows_by_custom_column(
            self._gtfs_files.stop_times,
            trip_ids_to_keep,
            columns=["trip_id"],
            return_columns=["stop_id"],
            write_out=True,
            dtype=GtfsDtypes.stop_times,
        )[0]

    def _filter_shapes(self, shape_ids_to_keep: Set) -> None:
        if not self._gtfs_files.shapes.exists():
            return
        if isinstance(shape_ids_to_keep, Set) and len(shape_ids_to_keep) > 0:
            logger.info("Filter shapes.txt")
            self.__filter_rows_by_custom_column(
                file_path=self._gtfs_files.shapes,
                ids=shape_ids_to_keep,
                columns=["shape_id"],
                write_out=True,
                dtype=GtfsDtypes.shapes,
                low_memory=False,
            )

    def _filter_agencies(self, agency_ids_to_keep: Set) -> None:
        logger.info("Filter agencies.txt")
        self.__filter_rows_by_custom_column(
            self._gtfs_files.agency, agency_ids_to_keep, columns=["agency_id"], write_out=True, dtype=GtfsDtypes.agency
        )

    def _filter_calendar_dates_using_services(self, service_ids_to_keep: Set) -> None:
        logger.info("Filter calendar_dates.txt")
        self.__filter_rows_by_custom_column(
            self._gtfs_files.calendar_dates,
            service_ids_to_keep,
            columns=["service_id"],
            write_out=True,
            dtype=GtfsDtypes.calendar_dates,
        )

    def _filter_calendar_using_services(self, service_ids_to_keep: Set) -> None:
        logger.info("Filter calendar.txt")
        self.__filter_rows_by_custom_column(
            self._gtfs_files.calendar,
            service_ids_to_keep,
            columns=["service_id"],
            write_out=True,
            dtype=GtfsDtypes.calendar,
        )

    def _filter_frequencies_using_trips(self, trip_ids_to_keep: Set) -> None:
        if self._gtfs_files.frequencies.exists():
            logger.info("Filter frequencies.txt")
            self.__filter_rows_by_custom_column(
                self._gtfs_files.frequencies,
                trip_ids_to_keep,
                columns=["trip_id"],
                write_out=True,
                dtype=GtfsDtypes.frequencies,
            )

    def _filter_stops(self, stop_ids_to_keep: Set) -> None:
        logger.info("Filter stops.txt")
        self.__filter_rows_by_custom_column(
            self._gtfs_files.stops, stop_ids_to_keep, columns=["stop_id"], write_out=True, dtype=GtfsDtypes.stops
        )

    def _filter_transfers_using_stops(self, stop_ids_to_keep: Set) -> None:
        # TODO filter_using_custom_column with multiple criterias
        if self._gtfs_files.transfers.exists():
            logger.info("Filter transfers.txt")
            self.__filter_rows_by_custom_column(
                self._gtfs_files.transfers,
                stop_ids_to_keep,
                columns=["from_stop_id", "to_stop_id"],
                write_out=True,
                dtype=GtfsDtypes.transfers,
            )

    def _filter_calendar_by_dates(self, start_date: datetime, end_date: datetime) -> Set:
        csv_chunks: ddf.DataFrame = ddf.read_csv(
            self._gtfs_files.calendar,
            dtype=GtfsDtypes.calendar,
            parse_dates=["start_date", "end_date"],
            date_parser=parse_date_from_str,
            low_memory=False,
        )
        # csv_chunks["start_date"] = ddf.to_datetime(csv_chunks["start_date"].dt.time.astype(str))
        with TqdmCallback(desc="Filter calendar.txt", unit=" chunks"):
            results: pd.DataFrame = csv_chunks.loc[
                (csv_chunks.start_date >= start_date) & (csv_chunks.end_date <= end_date)
            ].compute(scheduler=self._scheduler, num_workers=self._cpu_count)
        output_path: Path = self._output_folder.joinpath(self._gtfs_files.calendar.name)
        results.to_csv(output_path, index=False, doublequote=True, quoting=csv.QUOTE_ALL)
        return set(results.service_id)

    def _filter_calendar_dates_by_dates(self, start_date: datetime, end_date: datetime) -> Set:
        if not self._gtfs_files.calendar_dates.exists():
            return set()
        csv_chunks: ddf.DataFrame = ddf.read_csv(
            self._gtfs_files.calendar_dates,
            dtype=GtfsDtypes.calendar_dates,
            parse_dates=["date"],
            date_parser=parse_date_from_str,
            low_memory=False,
        )
        with TqdmCallback(desc="Filter calendar_dates.txt", unit=" chunks"):
            results: pd.DataFrame = csv_chunks.loc[
                (csv_chunks.date >= start_date) & (csv_chunks.date <= end_date)
            ].compute(scheduler=self._scheduler, num_workers=self._cpu_count)
        output_path: Path = self._output_folder.joinpath(self._gtfs_files.calendar.name)
        results.to_csv(output_path, index=False, doublequote=True, quoting=csv.QUOTE_ALL)
        return set(results.service_id)

    def _process_common_files(self, service_ids_to_keep: Set, trip_ids_to_keep: Set) -> None:
        self._filter_calendar_dates_using_services(service_ids_to_keep)
        self._filter_calendar_using_services(service_ids_to_keep)
        self._filter_frequencies_using_trips(trip_ids_to_keep)

        # Keep the stop_times used by the trips
        stop_ids_to_keep: Set = self._filter_stop_times_using_trips(trip_ids_to_keep)
        self._filter_stops(stop_ids_to_keep)
        self._filter_transfers_using_stops(stop_ids_to_keep)
        logger.info(f"{len(stop_ids_to_keep)} stops to keep")

        # Copy the feed info
        logger.info("Copy feed_info.txt to new location")
        shutil.copyfile(self._gtfs_files.feed_info, self._output_folder.joinpath(self._gtfs_files.feed_info.name))

    def _get_output_files(self) -> List:
        files: List = []
        for file in self._output_folder.glob("*.txt"):
            self._gtfs_files.set_files(file)
            files.append(file)
        return files

    def extract_by_agency(self, agencies: List[str]) -> None:
        ...

    def extract_by_bbox(self, bbox: Bbox) -> List:
        logger.info("Filter stops within bbox")
        stop_ids_in_bbox = self._get_stops_in_bbox(bbox)
        logger.info("Found {} stops in bbox".format(len(stop_ids_in_bbox)))

        logger.info("Filter trips from selected stops")
        trip_ids: Set
        trip_ids = self._get_trips_of_stop_times(stop_ids_in_bbox)
        logger.info("Found {} trips in bbox".format(len(trip_ids)))

        logger.info("Filter routes from selected trips")
        route_ids_to_keep: Set
        service_ids_to_keep: Set
        shape_ids_to_keep: Set
        route_ids_to_keep, service_ids_to_keep, shape_ids_to_keep = self._filter_trips(trip_ids)
        logger.info("Found {} routes in bbox".format(len(route_ids_to_keep)))

        logger.info("Filter agencies")
        agency_ids_to_keep: Set
        agency_ids_to_keep = self._filter_routes(route_ids_to_keep)
        self._filter_agencies(agency_ids_to_keep)
        logger.info("Found {} agencies in bbox".format(len(agency_ids_to_keep)))

        self._filter_shapes(shape_ids_to_keep)

        self._process_common_files(service_ids_to_keep=service_ids_to_keep, trip_ids_to_keep=trip_ids)

        return self._get_output_files()

    def extract_by_date(self, start_date: datetime, end_date: datetime) -> List:
        logger.info(f"Filter calendar.txt between {start_date} and {end_date}")
        service_ids_to_keep: Set = self._filter_calendar_by_dates(start_date, end_date)
        service_ids_to_keep_addition: Set = self._filter_calendar_dates_by_dates(start_date, end_date)
        service_ids_to_keep.update(service_ids_to_keep_addition)
        logger.info(f"Found {len(service_ids_to_keep)} calendar entries")

        logger.info("Filter trips from selected calendar entries")
        trip_ids_to_keep: Set
        route_ids_to_keep, trip_ids_to_keep, shape_ids_to_keep = self._filter_trips_by_service_ids(service_ids_to_keep)
        logger.info(f"Found {len(trip_ids_to_keep)} trips between dates")

        logger.info("Filter agencies")
        agency_ids_to_keep: Set
        agency_ids_to_keep = self._filter_routes(route_ids_to_keep)
        self._filter_agencies(agency_ids_to_keep)
        logger.info("Found {} agencies between dates".format(len(agency_ids_to_keep)))

        self._filter_shapes(shape_ids_to_keep)

        self._process_common_files(service_ids_to_keep=service_ids_to_keep, trip_ids_to_keep=trip_ids_to_keep)

        return self._get_output_files()
