import pathlib
import random
import shutil
import string
import zipfile
from pathlib import Path
from typing import Generator

import pytest
from _pytest.tmpdir import TempPathFactory
from fastapi.testclient import TestClient
from sqlalchemy.event import listen
from sqlmodel import Session, SQLModel, create_engine

from gtfs_general.application import create_app
from gtfs_general.db.session import create_spatialite_db, load_spatialite

script_path = pathlib.Path(__file__).parent.resolve()


@pytest.fixture(scope="module")
def test_client() -> Generator[TestClient, None, None]:
    with TestClient(create_app()) as c:
        yield c


@pytest.fixture(scope="function")
def in_memory_spatialite_function_session() -> Generator[Session, None, None]:
    engine = create_engine("sqlite:///:memory:", echo=True)
    listen(engine, "connect", load_spatialite)
    conn = engine.connect()
    SQLModel.metadata.create_all(engine)
    with Session(engine) as session:
        yield session
    SQLModel.metadata.drop_all(engine, checkfirst=True)
    conn.close()


@pytest.fixture(scope="function")
def in_mapped_spatialite_function_session(tmp_path_factory: TempPathFactory) -> Generator[Session, None, None]:
    # Create random db name as a string
    db_name = "".join(random.sample(string.ascii_lowercase, 4)) + ".db"
    engine = create_spatialite_db(tmp_path_factory.mktemp("db-files"), db_name)
    listen(engine, "connect", load_spatialite)
    conn = engine.connect()
    SQLModel.metadata.create_all(engine)
    with Session(engine) as session:
        yield session
    SQLModel.metadata.drop_all(engine, checkfirst=True)
    conn.close()


@pytest.fixture(scope="function")
def gtfs_test_folder(
    tmp_path_factory: TempPathFactory,
) -> Generator[Path, None, None]:
    tmp_path: Path = tmp_path_factory.mktemp("test_files")
    test_gtfs_file: str = script_path.joinpath("files/ic_ice_gtfs_germany.zip").__str__()
    with zipfile.ZipFile(test_gtfs_file, "r") as zip_ref:
        zip_ref.extractall(tmp_path)
    yield tmp_path
    shutil.rmtree(tmp_path)
