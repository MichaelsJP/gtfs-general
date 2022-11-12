from datetime import datetime
from enum import Enum
from typing import Optional

from sqlmodel import Field, Relationship, SQLModel


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


class Agency(SQLModel, table=True):
    agency_name: str = Field(primary_key=True, unique=True)
    agency_id: str = Field(default=None, primary_key=True, index=True, nullable=True, unique=True)
    agency_url: str = Field(nullable=False)
    agency_timezone: str = Field(nullable=False)
    agency_lang: str = Field(nullable=True)
    agency_phone: str = Field(nullable=True)
    agency_fare_url: str = Field(nullable=True)
    agency_email: str = Field(nullable=True)

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
    stations: list["Stops"] = Relationship(back_populates="parent_stop")

    stop_timezone: Optional[str] = Field(nullable=True)
    wheelchair_boarding: Optional[WheelchairBoarding] = Field(nullable=True)
    # Todo foreign key to levels
    level_id: Optional[str] = Field(nullable=True)
    platform_code: Optional[str] = Field()

    class Config:
        arbitrary_types_allowed = True

    # geom = Field(Column(Geometry(geometry_type='POINT', management=True, srid=4326, spatial_index=True)))
    # @validates("geom")
    # def validate_geom(self, _: str, geom: Any) -> Union[WKBElement, None]:
    #     return json_decoders.json_to_wkb_element(geom)


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

    # Needed for Column(JSON)
    class Config:
        arbitrary_types_allowed = True
