import pathlib
import shutil
import zipfile
from pathlib import Path
from typing import Generator

import pytest
from _pytest.tmpdir import TempPathFactory
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.event import listen
from sqlalchemy.sql import func, select

from gtfs_general.application import create_app
from gtfs_general.db.session import load_spatialite

script_path = pathlib.Path(__file__).parent.resolve()


@pytest.fixture(scope="module")
def fastapi_client() -> Generator[TestClient, None, None]:
    with TestClient(create_app()) as c:
        yield c


@pytest.fixture(scope="module")
def spatialite_client(tmp_path_factory: TempPathFactory) -> Generator[TestClient, None, None]:
    tmp_path: Path = tmp_path_factory.mktemp("test_db")
    tmp_db_path: Path = tmp_path.joinpath("gis.db")
    engine = create_engine(f"sqlite:///{tmp_db_path}", echo=True)
    listen(engine, "connect", load_spatialite)
    conn = engine.connect()
    conn.execute(select([func.InitSpatialMetaData()]))
    yield engine


@pytest.fixture(scope="function")
def gtfs_test_folder(tmp_path_factory: TempPathFactory) -> Generator:
    tmp_path: Path = tmp_path_factory.mktemp("test_files")
    test_gtfs_file: str = script_path.joinpath("files/ic_ice_gtfs_germany.zip").__str__()
    with zipfile.ZipFile(test_gtfs_file, "r") as zip_ref:
        zip_ref.extractall(tmp_path)
    yield tmp_path
    shutil.rmtree(tmp_path)
