import pathlib
import shutil
import zipfile
from pathlib import Path
from typing import Generator

import pytest
from _pytest.tmpdir import TempPathFactory

script_path = pathlib.Path(__file__).parent.resolve()


@pytest.fixture(scope="function")
def gtfs_test_folder(tmp_path_factory: TempPathFactory) -> Generator:
    tmp_path: Path = tmp_path_factory.mktemp("test_files")
    test_gtfs_file: str = script_path.joinpath("files/ic_ice_gtfs_germany.zip").__str__()
    with zipfile.ZipFile(test_gtfs_file, "r") as zip_ref:
        zip_ref.extractall(tmp_path)
    yield tmp_path
    shutil.rmtree(tmp_path)
