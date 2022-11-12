import datetime

import pytest
from sqlalchemy.exc import IntegrityError, PendingRollbackError
from sqlmodel import Session, select

from gtfs_general.db import Calendar


@pytest.mark.parametrize(
    "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date,expect_to_fail",
    [
        (None, 0, 1, 0, 1, 0, 1, 0, datetime.datetime.now(), datetime.datetime.now() + datetime.timedelta(days=8), True),
        (
            "1",
            0,
            1,
            0,
            1,
            0,
            1,
            0,
            datetime.datetime.utcnow(),
            datetime.datetime.utcnow() + datetime.timedelta(days=8),
            False,
        ),
        ("1", 0, 1, 0, 1, 0, 1, 0, datetime.datetime.now(), datetime.datetime.now() + datetime.timedelta(days=8), True),
        (
            "2",
            None,
            1,
            0,
            1,
            0,
            1,
            0,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "3",
            123,
            1,
            0,
            1,
            0,
            1,
            0,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        ("4", 1, 1, 0, 1, 0, 1, 0, None, datetime.datetime.now() + datetime.timedelta(days=8), True),
        ("6", 1, 1, 0, 1, 0, 1, 0, datetime.datetime.now(), None, True),
        ("7", 0, 1, 0, 1, 0, 1, 0, datetime.datetime.now(), datetime.datetime.now() + datetime.timedelta(days=8), False),
    ],
)
def test_calendar(
    service_id: str,
    monday: int,
    tuesday: int,
    wednesday: int,
    thursday: int,
    friday: int,
    saturday: int,
    sunday: int,
    start_date: int,
    end_date: int,
    expect_to_fail: int,
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    calendar_object = Calendar(
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
    session.add(calendar_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        calendar_object_from_db = session.exec(
            select(Calendar).where(Calendar.service_id == calendar_object.service_id)
        ).all()
        assert len(calendar_object_from_db) == 1
        assert calendar_object_from_db[0] == calendar_object
