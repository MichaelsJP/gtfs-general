from __future__ import annotations

from typing import Dict

from fastapi import FastAPI
from gunicorn.app.base import BaseApplication

from gtfs_general.api.api_v1.api import api_router
from gtfs_general.config import settings


def create_app() -> FastAPI:
    fastapi_app = FastAPI()
    fastapi_app.include_router(api_router, prefix=settings.API_V1_STR)
    return fastapi_app


class StandaloneApplication(BaseApplication):
    """Our Gunicorn application."""

    def __init__(self, app: FastAPI, options: Dict | None = None):
        self.options = options or {}
        self.application = app
        super().__init__()

    def load_config(self) -> None:
        config = {key: value for key, value in self.options.items() if key in self.cfg.settings and value is not None}
        for key, value in config.items():
            self.cfg.set(key.lower(), value)

    def load(self) -> FastAPI:
        return self.application
