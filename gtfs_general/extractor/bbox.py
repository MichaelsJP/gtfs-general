from __future__ import annotations


class Bbox:
    _min_lat: float
    _max_lat: float
    _min_lon: float
    _max_lon: float

    def __init__(self, min_lon: float, min_lat: float, max_lon: float, max_lat: float) -> None:
        super().__init__()
        self._min_lat = min_lat
        self._max_lat = max_lat
        self._min_lon = min_lon
        self._max_lon = max_lon

    def contains(self, lat: float, lon: float) -> bool:
        return self._max_lat >= lat >= self._min_lat and self._max_lon >= lon >= self._min_lon
