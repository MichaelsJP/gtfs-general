import pathlib

from pydantic_settings import BaseSettings

from gtfs_general.logging import LoggingLevel

script_location = pathlib.Path(__file__).parent.resolve()


class Settings(BaseSettings):
    __api_v1_str: str = "/api/v1"
    __logging_level: LoggingLevel = LoggingLevel.INFO

    def __init__(self, logging_level: LoggingLevel = LoggingLevel.INFO) -> None:
        super().__init__()
        self.__logging_level = logging_level

    @property
    def api_v1_str(self) -> str:
        return self.__api_v1_str

    @property
    def logging_level(self) -> LoggingLevel:
        return self.__logging_level
