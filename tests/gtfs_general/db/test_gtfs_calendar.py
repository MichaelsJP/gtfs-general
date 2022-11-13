import datetime

import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db import Calendar
from gtfs_general.db.factories import calendar_factory


@pytest.mark.parametrize(
    "service_id,start_date,end_date,expect_to_fail",
    [
        (
            None,
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "1",
            datetime.datetime.utcnow(),
            datetime.datetime.utcnow() + datetime.timedelta(days=8),
            False,
        ),
        (
            "2",
            datetime.datetime.now(),
            datetime.datetime.now() + datetime.timedelta(days=8),
            False,
        ),
        (
            "4",
            None,
            datetime.datetime.now() + datetime.timedelta(days=8),
            True,
        ),
        (
            "6",
            datetime.datetime.now(),
            None,
            True,
        ),
    ],
)
def test_calendar(
    service_id: str,
    start_date: datetime.datetime,
    end_date: datetime.datetime,
    expect_to_fail: bool,
    in_memory_spatialite_function_session: Session,
) -> None:
    session: Session = in_memory_spatialite_function_session
    original_object = calendar_factory(
        service_id=service_id,
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


def test_calendar_duplicate_fail(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    start_date = datetime.datetime.now()
    end_date = datetime.datetime.now() + datetime.timedelta(days=8)
    first_object = calendar_factory(
        service_id="1",
        start_date=start_date,
        end_date=end_date,
    )
    duplicate_object = calendar_factory(
        service_id="1",
        start_date=start_date,
        end_date=end_date,
    )
    session.add(first_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
