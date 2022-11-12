from enum import Enum


class DepartsOnDay(Enum):
    NOT_DEPARTING = 0
    DEPARTING = 1


class LocationType(Enum):
    STOP_OR_PLATTFORM = 0
    STATION = 1
    ENTRANCE_OR_EXIT = 2
    GENERIC_NODE = 3
    BOARDING_AREA = 4


class WheelchairBoarding(Enum):
    EMPTY = 0
    ACCESSIBLE = 1
    NOT_ACCESSIBLE = 2


class RouteType(Enum):
    LIGHT_RAIL = 0
    UNDERGROUND_RAIL = 1
    RAIL = 2
    BUS = 3
    FERRY = 4
    CABLE_TRAM = 5
    AERIAL_LIFT = 6
    FUNICULAR = 7
    TROLLEYBUS = 11
    MONORAIL = 12


class ContinuousPickup(Enum):
    CONTINUOUS_PICKUP = 0
    NO_CONTINUOUS_PICKUP = 1
    PHONE_PICKUP = 2
    COORDINATE_DRIVER_PICKUP = 3


class ContinuousDropOff(Enum):
    CONTINUOUS_DROP_OFF = 0
    NO_CONTINUOUS_DROP_OFF = 1
    PHONE_DROP_OFF = 2
    COORDINATE_DRIVER_DROP_OFF = 3
