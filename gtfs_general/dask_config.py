# dask_foo/config.py
from __future__ import annotations

import os
from pathlib import Path

import yaml

import dask.config


config_file = os.path.join(os.path.dirname(__file__), "../dask.yaml")


def initialize_dask() -> None:
    defaults: dict | None = None
    if Path(config_file).exists():
        with open(config_file) as f:
            defaults = yaml.safe_load(f)
    if isinstance(defaults, dict):
        dask.config.update_defaults(defaults)
