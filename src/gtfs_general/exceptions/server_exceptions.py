from gtfs_general.exceptions.extractor_exceptions import CustomException
from src.gtfs_general import logger


class LibSpatialLiteNotFound(CustomException):
    def __init__(self) -> None:
        self.message = "Couldn't find libspatialite.so. Check if installed."
        logger.error(self.message)
        super().__init__(self.message)

    def __str__(self) -> str:
        return self.message
