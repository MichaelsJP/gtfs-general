from datetime import datetime

from numpy import ndarray


def parse_date_from_str(x: [str, ndarray]) -> datetime | list[datetime]:
    # if x str do the original call if ndarray do the same but with a list comprehension
    if isinstance(x, str):
        return datetime.strptime(x, "%Y%m%d")
    else:
        return [datetime.strptime(i, "%Y%m%d") for i in x]
