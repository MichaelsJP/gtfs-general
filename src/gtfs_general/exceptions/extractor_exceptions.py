from src.gtfs_general import logger


class CustomException(Exception):
    """Base custom exception class"""

    pass


class GtfsIncompleteException(CustomException):
    def __init__(self) -> None:
        self.message = "Your GTFS input is missing required files."
        logger.error(self.message)
        super().__init__(self.message)

    def __str__(self) -> str:
        return self.message


class GtfsFileNotFound(CustomException):
    def __init__(self, file_path: str) -> None:
        self.message = f"Couldn't find the given file: {file_path}"
        self.file_path = file_path
        logger.error(self.message)
        super().__init__(self.message)

    def __str__(self) -> str:
        return f"{self.message}: {self.file_path}"
