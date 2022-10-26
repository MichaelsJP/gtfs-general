import pathlib

from pydantic import (
    BaseSettings,
)

script_location = pathlib.Path(__file__).parent.resolve()


class Settings(BaseSettings):
    API_V1_STR: str = "/api/v1"


settings = Settings()
