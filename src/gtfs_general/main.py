from __future__ import annotations

import os
import time
from dataclasses import dataclass
from datetime import datetime
from functools import partialmethod
from pathlib import Path

from typing import Optional, List

import typer
from tqdm import tqdm

from . import __app_name__, __version__, logger
from .application import StandaloneApplication, create_app
from .dask_config import initialize_dask
from .extractor.bbox import Bbox
from .extractor.extractor import Extractor
from .extractor.gtfs import GTFS
from .docs import app as docs_app
from .logging import initialize_logging
import uvicorn

app = typer.Typer()

app.add_typer(docs_app, name="docs", help="Generate documentation")
script_start_time = time.time()

cpu_count: int | None = os.cpu_count()

if cpu_count is None or cpu_count == 1:
    cpu_count = 1
else:
    cpu_count = cpu_count - 1


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(f"{__app_name__} v{__version__}")
        raise typer.Exit()


@dataclass
class Shared:
    cpu_count: int


@app.command()
def extract_bbox(
    ctx: typer.Context,
    input_object: str = typer.Option(..., help="Directory or zip File from which the GFTS files are read"),
    output_folder: str = typer.Option(..., help="Directory to which the GFTS files are written"),
    bbox: str = typer.Option(
        ...,
        help="The bbox for selecting the GTFS data to keep. Format is WGS84 Coordinates lon/lat (lon min, lat min, "
        'lon max, lat max) Example: "8.573179,49.352003,8.79405,49.459693"',
    ),
) -> None:
    coordinates: List[float] = [float(x.strip()) for x in bbox.split(",")]
    logger.info("#################################")
    logger.info("######## Extract by bbox ########")
    logger.info(f"Input: {input_object}")
    logger.info(f"bbox: {coordinates}")
    logger.info("#################################")
    logger.info("####### Start processing ########")
    keep_bbox: Bbox = Bbox(*coordinates)
    extractor: Extractor = Extractor(
        input_object=Path(input_object), output_folder=Path(output_folder), cpu_count=ctx.obj.cpu_count
    )
    files: List = extractor.extract_by_bbox(bbox=keep_bbox)
    extractor.close()
    logger.info("#################################")
    logger.info("############ Result ############")
    run_time: str = datetime.utcfromtimestamp(time.time() - script_start_time).strftime("%H:%M:%S.%f")
    logger.info(f"Run time: {run_time}")
    logger.info(f"Processed {len(files)} files:")
    file: Path
    for file in files:
        logger.info(file.__str__())
    logger.info("################################")


@app.command()
def extract_date(
    ctx: typer.Context,
    input_object: str = typer.Option(..., help="Directory or zip File from which the GFTS files are read"),
    output_folder: str = typer.Option(..., help="Directory to which the GFTS files are written"),
    start_date: str = typer.Option(
        ..., help="Lower date boundary. Format: YYYYMMDD. e.g. 20221002 for 2nd October 2022"
    ),
    end_date: str = typer.Option(..., help="Lower date boundary. Format: YYYYMMDD. e.g. 20221002 for 2nd October 2022"),
) -> None:
    logger.info("#################################")
    logger.info("######## Extract by date ########")
    logger.info(f"Input: {input_object}")
    logger.info(f"Start date: {start_date}")
    logger.info(f"End date: {end_date}")
    logger.info("#################################")
    logger.info("####### Start processing ########")
    extractor: Extractor = Extractor(
        input_object=Path(input_object), output_folder=Path(output_folder), cpu_count=ctx.obj.cpu_count
    )
    files: List = extractor.extract_by_date(
        start_date=datetime.strptime(start_date, "%Y%m%d"), end_date=datetime.strptime(end_date, "%Y%m%d")
    )
    extractor.close()
    logger.info("################################")
    logger.info("############ Result ############")
    run_time: str = datetime.utcfromtimestamp(time.time() - script_start_time).strftime("%H:%M:%S.%f")
    logger.info(f"Run time: {run_time}")
    logger.info(f"Processed {len(files)} files:")
    file: Path
    for file in files:
        logger.info(file.__str__())
    logger.info("################################")


@app.command()
def metadata(
    ctx: typer.Context,
    input_object: str = typer.Option(..., help="Directory or zip File from which the GFTS files are read"),
) -> None:
    logger.info("################################")
    logger.info("####### Extract Metadata #######")
    logger.info(f"Input: {input_object}")
    logger.info("################################")
    logger.info("####### Start processing #######")
    gtfs: GTFS = GTFS(input_object=Path(input_object), cpu_count=ctx.obj.cpu_count)
    dates = gtfs.service_date_range()
    gtfs.close()
    logger.info("############ Result ############")
    run_time: str = datetime.utcfromtimestamp(time.time() - script_start_time).strftime("%H:%M:%S.%f")
    logger.info(f"Run time: {run_time}")
    logger.info(f"Service date window from '{dates[0]}' to '{dates[1]}'")
    logger.info("################################")


@app.command()
def server(
    ctx: typer.Context,
    host: str = typer.Option("0.0.0.0", help="Provide the desired host."),
    port: int = typer.Option(8080, help="Provide the desired port."),
    reload: bool = typer.Option(False, help="Activate automatic server reload on code changes."),
    workers: int = typer.Option(1, help="Define number of uvicorn workers."),
    gunicorn: bool = typer.Option(False, help="Use Gunicorn instead of uvicorn."),
) -> None:
    if not gunicorn:
        uvicorn.run("gtfs_general.main:create_app", host=host, port=port, reload=reload, workers=workers)
    else:
        options = {"bind": f"{host}:{port}", "workers": workers, "worker_class": "uvicorn.workers.UvicornWorker"}
        StandaloneApplication(create_app(), options).run()


@app.callback()
def main(
    ctx: typer.Context,
    logging: Optional[str] = "INFO",
    cores: int = typer.Option(cpu_count - 1 if cpu_count else 1, help="Set the number of cores to use for processing."),
    progress: Optional[bool] = typer.Option(True, help="Deactivate the progress bars."),
    version: Optional[bool] = typer.Option(
        None,
        "--version",
        "-v",
        help="Show the application's version and exit.",
        callback=_version_callback,
        is_eager=True,
    ),
) -> None:
    if not progress:
        tqdm.__init__ = partialmethod(tqdm.__init__, disable=True)
    if logging is None:
        logging = "INFO"
    initialize_logging(logging)
    initialize_dask()
    logger.info("############ Run info ############")
    logger.info(f"Log level: {logging}")
    logger.info(f"Number of cores: {cores}")
    ctx.obj = Shared(cpu_count=cores)
    return
