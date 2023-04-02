import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db.factories import agency_factory
from gtfs_general.db.gtfs import Agency


@pytest.mark.parametrize(
    "agency_id,agency_name,expect_to_fail",
    [
        (
            "1",
            "foo_agency",
            False,
        ),
        (
            "2",
            "foo_agency",
            False,
        ),
        (
            None,
            "foo_agency",
            False,
        ),
        (
            "3",
            None,
            True,
        ),
        (
            None,
            None,
            True,
        ),
    ],
)
def test_agency(
    agency_name: str, agency_id: str, expect_to_fail: bool, in_memory_spatialite_function_session: Session
) -> None:
    session: Session = in_memory_spatialite_function_session
    original_object: Agency = agency_factory(agency_id=agency_id, agency_name=agency_name)
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Agency).where(Agency.agency_id == original_object.agency_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object


@pytest.mark.parametrize(
    "agency_id,agency_name",
    [
        ("1", "foo_agency"),
        (None, "foo_agency"),
    ],
)
def test_agency_duplicate(agency_id: str, agency_name: str, in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    original_object = agency_factory(agency_id=agency_id, agency_name=agency_name)
    duplicate_object = agency_factory(agency_id=agency_id, agency_name=agency_name)
    session.add(original_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
