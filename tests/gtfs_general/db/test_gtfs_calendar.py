import datetime

import pytest
from sqlalchemy.exc import IntegrityError, PendingRollbackError
from sqlmodel import Session, select

from gtfs_general.db import Calendar
from gtfs_general.db.gtfs import DepartsOnDay


@pytest.mark.parametrize(
    "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,expect_to_fail",
    [
        (
            None,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "1",
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.utcnow(),
            datetime.datetime.utcnow() + datetime.timedelta(days=8),
            False,
        ),
        (
            "2",
            None,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "3",
            123,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "4",
            DepartsOnDay.DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            None,
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "6",
            DepartsOnDay.DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.now(),
            None,
            True,
        ),
        (
            "7",
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            DepartsOnDay.DEPARTING,
            DepartsOnDay.NOT_DEPARTING,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            False,
        ),
    ],
)
def test_calendar(
    service_id: str,
    monday: DepartsOnDay,
    tuesday: int,
    wednesday: int,
    thursday: int,
    friday: int,
    saturday: int,
    sunday: int,
    start_date: int,
    end_date: int,
    expect_to_fail: bool,
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    original_object = Calendar(
        service_id=service_id,
        monday=monday,
        tuesday=tuesday,
        wednesday=wednesday,
        thursday=thursday,
        friday=friday,
        saturday=saturday,
        sunday=sunday,
        start_date=start_date,
        end_date=end_date,
    )
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Calendar).where(Calendar.service_id == original_object.service_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object


def test_calendar_duplicate_fail(
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    first_object = Calendar(
        service_id="1",
        monday=DepartsOnDay.NOT_DEPARTING,
        tuesday=DepartsOnDay.NOT_DEPARTING,
        wednesday=DepartsOnDay.NOT_DEPARTING,
        thursday=DepartsOnDay.NOT_DEPARTING,
        friday=DepartsOnDay.NOT_DEPARTING,
        saturday=DepartsOnDay.NOT_DEPARTING,
        sunday=DepartsOnDay.NOT_DEPARTING,
        start_date=datetime.datetime.now(),
        end_date=datetime.datetime.now() + datetime.timedelta(days=8),
    )
    duplicate_object = Calendar(
        service_id="1",
        monday=DepartsOnDay.NOT_DEPARTING,
        tuesday=DepartsOnDay.NOT_DEPARTING,
        wednesday=DepartsOnDay.NOT_DEPARTING,
        thursday=DepartsOnDay.NOT_DEPARTING,
        friday=DepartsOnDay.NOT_DEPARTING,
        saturday=DepartsOnDay.NOT_DEPARTING,
        sunday=DepartsOnDay.NOT_DEPARTING,
        start_date=datetime.datetime.now(),
        end_date=datetime.datetime.now() + datetime.timedelta(days=8),
    )
    session.add(first_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
