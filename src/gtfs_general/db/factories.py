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
    Stops,
    WheelchairBoarding,
)


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
        wheelchair_boarding=WheelchairBoarding.ACCESSIBLE,
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
