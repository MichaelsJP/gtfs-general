from datetime import datetime


def parse_date_from_str(x: str) -> datetime:
    return datetime.strptime(x, "%Y%m%d")
