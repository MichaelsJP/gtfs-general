from __future__ import annotations

import sys
from typing import Dict

from fastapi import FastAPI
from gunicorn.app.base import BaseApplication

from gtfs_general import logger, logging
from gtfs_general.api.api_v1.api import api_router
from gtfs_general.config import Settings
from gtfs_general.logging import CustomFormatter


def create_app() -> FastAPI:
    logger.debug("Creating FastAPI app")
    fastapi_app = FastAPI()
    fastapi_app.include_router(api_router, prefix=Settings().api_v1_str)
    return fastapi_app


def initialize_logging(settings: Settings) -> None:
    level = settings.logging_level.value
    correct_level = logging.getLevelName(level)
    logger.setLevel(correct_level)

    stdout_handler = logging.StreamHandler(sys.stdout)
    stdout_handler.setLevel(correct_level)
    stdout_handler.setFormatter(CustomFormatter())

    # file_handler = logging.FileHandler("logs.log")
    # file_handler.setLevel(correct_level)
    # file_handler.setFormatter(CustomFormatter())

    # logger.addHandler(file_handler)
    logger.addHandler(stdout_handler)


class StandaloneApplication(BaseApplication):
    """Our Gunicorn application."""

    def __init__(self, app: str, options: Dict | None = None):
        self.options = options or {}
        self.application = app
        super().__init__()

    def load_config(self) -> None:
        config = {key: value for key, value in self.options.items() if key in self.cfg.settings and value is not None}
        for key, value in config.items():
            self.cfg.set(key.lower(), value)

    def load(self) -> str:
        return self.application
