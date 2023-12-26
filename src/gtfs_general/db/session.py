from pathlib import Path
from typing import Any

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.event import listen

from gtfs_general import logger


def load_spatialite(dbapi_conn: Any, connection_record: Any) -> None:
    dbapi_conn.enable_load_extension(True)
    try:
        logger.info("Trying to load mod_spatialite")
        dbapi_conn.load_extension("mod_spatialite")
    except Exception as err:
        logger.warning("Failed to load mod_spatialite: %s", err)

    try:
        logger.info("Trying to load mod_spatialite.so")
        dbapi_conn.load_extension("mod_spatialite.so")
    except Exception as err:
        logger.error("Failed to load mod_spatialite.so: %s", err)
        raise err


def create_spatialite_db(path: Path, db_name: str = "test.db") -> Engine:
    """Create a spatialite database at the temp path."""
    engine = create_engine(f"sqlite:///{path}/{db_name}", echo=True)
    listen(engine, "connect", load_spatialite)
    return engine
