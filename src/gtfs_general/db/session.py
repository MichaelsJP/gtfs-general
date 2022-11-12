from pathlib import Path
from typing import Any

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.event import listen

from .gtfs import Calendar  # noqa


def load_spatialite(dbapi_conn: Any, connection_record: Any) -> None:
    dbapi_conn.enable_load_extension(True)
    dbapi_conn.load_extension("mod_spatialite")


def create_spatialite_db(folder: Path) -> Engine:
    engine = create_engine("sqlite:///gis.db", echo=True)
    listen(engine, "connect", load_spatialite)
    return engine
