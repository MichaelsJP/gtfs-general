from datetime import datetime
from enum import Enum

from sqlmodel import Field, SQLModel


class DepartsOnDay(Enum):
    NOT_DEPARTING = 0
    DEPARTING = 1


class Calendar(SQLModel, table=True):
    service_id: str = Field(primary_key=True, index=True, nullable=False)
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
