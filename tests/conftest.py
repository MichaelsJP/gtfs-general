import pathlib
import shutil
import zipfile
from pathlib import Path
from typing import Generator

import pytest
from _pytest.tmpdir import TempPathFactory
from fastapi.testclient import TestClient
from sqlalchemy.event import listen
from sqlalchemy.sql import func, select
from sqlmodel import Session, SQLModel, create_engine

from gtfs_general.application import create_app
from gtfs_general.db.session import load_spatialite

script_path = pathlib.Path(__file__).parent.resolve()


@pytest.fixture(scope="module")
def fastapi_client() -> Generator[TestClient, None, None]:
    with TestClient(create_app()) as c:
        yield c


@pytest.fixture(scope="function")
def in_memory_spatialite_session() -> Generator[Session, None, None]:
    engine = create_engine("sqlite:///:memory:", echo=True)
    listen(engine, "connect", load_spatialite)
    conn = engine.connect()
    conn.execute(select([func.InitSpatialMetaData()]))
    SQLModel.metadata.create_all(engine)
    with Session(engine) as session:
        yield session
    SQLModel.metadata.drop_all(engine, checkfirst=True)
    conn.close()


@pytest.fixture(scope="session")
def gtfs_test_folder(tmp_path_factory: TempPathFactory) -> Generator:
    tmp_path: Path = tmp_path_factory.mktemp("test_files")
    test_gtfs_file: str = script_path.joinpath("files/ic_ice_gtfs_germany.zip").__str__()
    with zipfile.ZipFile(test_gtfs_file, "r") as zip_ref:
        zip_ref.extractall(tmp_path)
    yield tmp_path
    shutil.rmtree(tmp_path)
