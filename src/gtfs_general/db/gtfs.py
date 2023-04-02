from datetime import datetime
from decimal import Decimal
from typing import List, Optional

from pydantic import NonNegativeInt, condecimal
from sqlmodel import Field, Relationship, SQLModel

from gtfs_general.utils.enumerations import (
    BikesAllowed,
    ContinuousDropOff,
    ContinuousPickup,
    DepartsOnDay,
    DropOffType,
    LocationType,
    PickupType,
    RouteType,
    TimePoint,
    TravelDirection,
    WheelchairAccessible,
)


class Agency(SQLModel, table=True):
    agency_name: str = Field(primary_key=True, unique=True)
    agency_id: str = Field(default=None, primary_key=True, index=True, nullable=True, unique=True)
    agency_url: str = Field(nullable=False)
    agency_timezone: str = Field(nullable=False)
    agency_lang: str = Field(nullable=True)
    agency_phone: str = Field(nullable=True)
    agency_fare_url: str = Field(nullable=True)
    agency_email: str = Field(nullable=True)

    routes: List["Routes"] = Relationship(back_populates="agency")

    class Config:
        arbitrary_types_allowed = True


class Stops(SQLModel, table=True):
    stop_id: str = Field(primary_key=True, unique=True)
    stop_code: Optional[str] = Field(nullable=True)
    stop_name: Optional[str] = Field(nullable=True)
    stop_desc: Optional[str] = Field(nullable=True)
    stop_lat: Optional[float] = Field(nullable=True)
    stop_lon: Optional[float] = Field(nullable=True)
    zone_id: Optional[str] = Field(nullable=True)
    stop_url: Optional[str] = Field(nullable=True)
    location_type: Optional[LocationType] = Field(nullable=True)

    parent_stop_id: Optional[int] = Field(
        foreign_key="stops.stop_id",  # notice the lowercase "n" to refer to the database table name
        default=None,
        nullable=True,
    )
    parent_stop: Optional["Stops"] = Relationship(
        back_populates="stations",
        sa_relationship_kwargs=dict(
            remote_side="Stops.stop_id"  # notice the uppercase "N" to refer to this table class
        ),
    )
    stations: List["Stops"] = Relationship(back_populates="parent_stop")

    stop_timezone: Optional[str] = Field(nullable=True)
    wheelchair_boarding: Optional[WheelchairAccessible] = Field(nullable=True)
    # Todo foreign key to levels
    level_id: Optional[str] = Field(nullable=True)
    platform_code: Optional[str] = Field()

    stop_times: List["StopTimes"] = Relationship(back_populates="stop")

    class Config:
        arbitrary_types_allowed = True

    # geom = Field(Column(Geometry(geometry_type='POINT', management=True, srid=4326, spatial_index=True)))
    # @validates("geom")
    # def validate_geom(self, _: str, geom: Any) -> Union[WKBElement, None]:
    #     return json_decoders.json_to_wkb_element(geom)


class Routes(SQLModel, table=True):
    route_id: str = Field(primary_key=True, unique=True)

    agency_id: Optional[str] = Field(
        foreign_key="agency.agency_id",
        default=None,
        nullable=True,
    )
    agency: Optional["Agency"] = Relationship(back_populates="routes")

    route_short_name: Optional[str] = Field(nullable=True)
    route_long_name: Optional[str] = Field(nullable=True)
    route_desc: Optional[str] = Field(nullable=True)
    route_type: RouteType = Field(nullable=False)
    route_url: Optional[str] = Field(nullable=True)
    route_color: Optional[str] = Field(nullable=True)
    route_text_color: Optional[str] = Field(nullable=True)
    route_sort_order: Optional[NonNegativeInt] = Field(nullable=True)
    continuous_pickup: Optional[ContinuousPickup] = Field(nullable=True)
    continuous_drop_off: Optional[ContinuousDropOff] = Field(nullable=True)

    trips: List["Trips"] = Relationship(back_populates="route")


class Trips(SQLModel, table=True):
    route_id: str = Field(foreign_key="routes.route_id", default=None, nullable=False, primary_key=True)
    route: Optional["Routes"] = Relationship(back_populates="trips")

    # Todo add relationship for calendar
    service_id: str = Field(foreign_key="calendar.service_id", nullable=False, primary_key=True)
    service: Optional["Calendar"] = Relationship(back_populates="trips")

    trip_id: str = Field(primary_key=True, unique=True)
    trip_headsign: Optional[str] = Field(nullable=True)
    trip_short_name: Optional[str] = Field(nullable=True)
    direction_id: TravelDirection = Field(nullable=True)
    block_id: Optional[str] = Field(nullable=True, index=True)

    shape_id: str = Field(foreign_key="shapes.shape_id", default=None, nullable=True)
    shape: Optional["Shapes"] = Relationship(back_populates="trips")

    wheelchair_accessible: Optional[WheelchairAccessible] = Field(nullable=True)
    bikes_allowed: Optional[BikesAllowed] = Field(nullable=True)

    stop_times: List["StopTimes"] = Relationship(back_populates="trip")


class StopTimes(SQLModel, table=True):
    trip_id: str = Field(foreign_key="trips.trip_id", default=None, nullable=False, primary_key=True)
    trip: Optional["Trips"] = Relationship(back_populates="stop_times")

    arrival_time: Optional[datetime] = Field(nullable=True)
    departure_time: Optional[datetime] = Field(nullable=True)

    stop_id: str = Field(foreign_key="stops.stop_id", default=None, nullable=False, primary_key=True)
    stop: Optional["Stops"] = Relationship(back_populates="stop_times")

    stop_sequence: NonNegativeInt = Field(nullable=False)
    stop_headsign: Optional[str] = Field(nullable=True)
    pickup_type: Optional[PickupType] = Field(nullable=True)
    drop_off_type: Optional[DropOffType] = Field(nullable=True)
    continuous_pickup: Optional[ContinuousPickup] = Field(nullable=True)
    continuous_drop_off: Optional[ContinuousDropOff] = Field(nullable=True)
    shape_dist_traveled: Optional[condecimal(ge=Decimal(0),)] = Field(
        default=0
    )  # Bug. Sqlmodel is ignoring condition for now.
    timepoint: Optional[TimePoint] = Field(nullable=True)

    class Config:
        arbitrary_types_allowed = True


class Calendar(SQLModel, table=True):
    service_id: str = Field(primary_key=True, index=True, nullable=False, unique=True)
    monday: DepartsOnDay = Field(nullable=False)
    tuesday: DepartsOnDay = Field(nullable=False)
    wednesday: DepartsOnDay = Field(nullable=False)
    thursday: DepartsOnDay = Field(nullable=False)
    friday: DepartsOnDay = Field(nullable=False)
    saturday: DepartsOnDay = Field(nullable=False)
    sunday: DepartsOnDay = Field(nullable=False)
    start_date: datetime = Field(nullable=False)
    end_date: datetime = Field(nullable=False)

    trips: List["Trips"] = Relationship(back_populates="service")

    class Config:
        arbitrary_types_allowed = True


class Shapes(SQLModel, table=True):
    shape_id: str = Field(primary_key=True, index=True, nullable=False, unique=True)
    shape_pt_lat: float = Field(nullable=False)
    shape_pt_lon: float = Field(nullable=False)
    shape_pt_sequence: NonNegativeInt = Field(nullable=False)
    # Todo bug when putting NonNegativeFloat optional. It doesn't check for positivity anymore.
    shape_dist_traveled: Optional[
        condecimal(
            ge=Decimal(0),
        )
    ] = Field(nullable=True)

    trips: List["Trips"] = Relationship(back_populates="shape")
