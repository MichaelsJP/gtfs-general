import datetime
from typing import Optional

from gtfs_general.db import Calendar
from gtfs_general.db.gtfs import (
    Agency,
    ContinuousDropOff,
    ContinuousPickup,
    DepartsOnDay,
    LocationType,
    Routes,
    RouteType,
    Shapes,
    Stops,
    StopTimes,
    Trips,
    WheelchairAccessible,
)
from gtfs_general.utils.enumerations import BikesAllowed, DropOffType, PickupType, TimePoint, TravelDirection


def stops_factory(stop_id: str, parent_stop: Optional[Stops] = None) -> Stops:
    return Stops(
        stop_id=stop_id,
        stop_code=f"{stop_id}_stop_code",
        stop_name=f"{stop_id}_stop_name",
        stop_desc=f"{stop_id}_stop_desc",
        stop_lat=f"{stop_id}_stop_lat",
        stop_lon=f"{stop_id}_stop_lon",
        zone_id=f"{stop_id}_zone_id",
        stop_url=f"{stop_id}_stop_url",
        location_type=LocationType.STOP_OR_PLATTFORM,
        parent_stop=parent_stop,
        stop_timezone=f"{stop_id}_stop_timezone",
        wheelchair_boarding=WheelchairAccessible.ACCESSIBLE,
        level_id=f"{stop_id}_level_id",
        platform_code=f"{stop_id}_platform_code",
    )


def agency_factory(agency_name: str, agency_id: Optional[str] = None) -> Agency:
    return Agency(
        agency_name=agency_name,
        agency_id=agency_id,
        agency_url=f"{agency_name}_agency_url",
        agency_timezone=f"{agency_name}_agency_timezone",
        agency_lang=f"{agency_name}_agency_lang",
        agency_phone=f"{agency_name}_agency_phone",
        agency_fare_url=f"{agency_name}_agency_fare_url",
        agency_email=f"{agency_name}_agency_email",
    )


def calendar_factory(
    service_id: str,
    start_date: datetime.datetime = datetime.datetime.now(),
    end_date: datetime.datetime = datetime.datetime.now() + datetime.timedelta(days=8),
) -> Calendar:
    return Calendar(
        service_id=service_id,
        monday=DepartsOnDay.DEPARTING,
        tuesday=DepartsOnDay.NOT_DEPARTING,
        wednesday=DepartsOnDay.DEPARTING,
        thursday=DepartsOnDay.NOT_DEPARTING,
        friday=DepartsOnDay.DEPARTING,
        saturday=DepartsOnDay.DEPARTING,
        sunday=DepartsOnDay.NOT_DEPARTING,
        start_date=start_date,
        end_date=end_date,
    )


def routes_factory(
    route_id: str,
    agency_id: Optional[str] = None,
    agency: Optional[Agency] = None,
    route_type: RouteType = RouteType.BUS,
    route_sort_order: int = 1,
    continuous_pickup: Optional[ContinuousPickup] = ContinuousPickup.NO_CONTINUOUS_PICKUP,
    continuous_drop_off: Optional[ContinuousDropOff] = ContinuousDropOff.NO_CONTINUOUS_DROP_OFF,
) -> Routes:
    return Routes(
        route_id=route_id,
        agency_id=agency_id,
        agency=agency,
        route_short_name="route_short_name",
        route_long_name="route_long_name",
        route_desc="route_desc",
        route_type=route_type,
        route_url="route_url",
        route_color="route_color",
        route_text_color="route_text_color",
        route_sort_order=route_sort_order,
        continuous_pickup=continuous_pickup,
        continuous_drop_off=continuous_drop_off,
    )


def trips_factory(route_id: str, service_id: str, trip_id: str, shape_id: Optional[str] = None) -> Trips:
    return Trips(
        route_id=route_id,
        service_id=service_id,
        trip_id=trip_id,
        trip_headsign=f"{trip_id}_trip_headsign",
        trip_short_name=f"{trip_id}_trip_short_name",
        direction_id=TravelDirection.TRAVEL_IN_ONE_DIRECTION,
        block_id=f"{trip_id}_block_id",
        shape_id=shape_id,
        wheelchair_accessible=WheelchairAccessible.ACCESSIBLE,
        bikes_allowed=BikesAllowed.BICYCLES_ALLOWED,
    )


def shapes_factory(
    shape_id: str,
    shape_pt_lat: float = 9.343212233,
    shape_pt_lon: float = 45.08930220,
    shape_pt_sequence: int = 1,
    shape_dist_traveled: Optional[float] = 0,
) -> Shapes:
    return Shapes(
        shape_id=shape_id,
        shape_pt_lat=shape_pt_lat,
        shape_pt_lon=shape_pt_lon,
        shape_pt_sequence=shape_pt_sequence,
        shape_dist_traveled=shape_dist_traveled,
    )


def stop_times_factory(
    trip_id: str,
    stop_id: str,
    stop_sequence: int,
    arrival_time: datetime.datetime = datetime.datetime.now(),
    departure_time: datetime.datetime = datetime.datetime.now() + datetime.timedelta(minutes=8),
    shape_dist_traveled: int = 1,
) -> StopTimes:
    return StopTimes(
        trip_id=trip_id,
        arrival_time=arrival_time,
        departure_time=departure_time,
        stop_id=stop_id,
        stop_sequence=stop_sequence,
        stop_headsign="Foo_Headsign",
        pickup_type=PickupType.PHONE_PICKUP,
        drop_off_type=DropOffType.REGULAR_DROP_OFF,
        continuous_pickup=ContinuousPickup.NO_CONTINUOUS_PICKUP,
        continuous_drop_off=ContinuousDropOff.COORDINATE_DRIVER_DROP_OFF,
        shape_dist_traveled=shape_dist_traveled,
        timepoint=TimePoint.TIMES_ARE_EXACT,
    )
