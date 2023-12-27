import logging
from enum import Enum


class LoggingLevel(str, Enum):
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"

    def __new__(cls, *args, **kwargs):  # type: ignore
        value = args[0]
        obj = str.__new__(cls, value)
        obj._value_ = value
        return obj

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return self.value

    def __eq__(self, other: object) -> bool:
        if isinstance(other, LoggingLevel):
            return self.value == other.value
        elif isinstance(other, str):
            return self.value == other
        else:
            return False

    def __hash__(self) -> int:
        return hash(self.value)


class CustomFormatter(logging.Formatter):
    grey = "\x1b[38;20m"
    yellow = "\x1b[33;20m"
    red = "\x1b[31;20m"
    bold_red = "\x1b[31;1m"
    reset = "\x1b[0m"
    underline = "\x1b[4m"
    format_string = "%(asctime)s - %(name)s - %(levelname)s - %(message)s (%(filename)s:%(lineno)d)"

    FORMATS = {
        logging.DEBUG: grey + underline + format_string + reset,
        logging.INFO: grey + format_string + reset,
        logging.WARNING: yellow + format_string + reset,
        logging.ERROR: red + format_string + reset,
        logging.CRITICAL: bold_red + format_string + reset,
    }

    def format(self, record):  # type: ignore
        log_fmt = self.FORMATS.get(record.levelno)
        formatter = logging.Formatter(log_fmt)
        return formatter.format(record)
