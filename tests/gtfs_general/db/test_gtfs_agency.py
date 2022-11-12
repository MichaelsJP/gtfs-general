import pytest
from sqlalchemy.exc import IntegrityError, PendingRollbackError
from sqlmodel import Session, select

from gtfs_general.db.gtfs import Agency


@pytest.mark.parametrize(
    "agency_id,agency_name,agency_url,agency_timezone,agency_lang,agency_phone,agency_fare_url,agency_email,"
    "expect_to_fail",
    [
        (
            "1",
            "foo_agency",
            "https://foo-agency.local",
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
            False,
        ),
        (
            None,
            "foo_agency",
            "https://foo-agency.local",
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
            False,
        ),
        (
            None,
            None,
            "https://foo-agency.local",
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
            True,
        ),
        (
            None,
            "foo_agency",
            None,
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
            True,
        ),
        (
            None,
            "foo_agency",
            "https://foo-agency.local",
            None,
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
            True,
        ),
    ],
)
def test_agency(
    agency_id: str,
    agency_name: str,
    agency_url: str,
    agency_timezone: str,
    agency_lang: str,
    agency_phone: str,
    agency_fare_url: str,
    agency_email: str,
    expect_to_fail: bool,
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    original_object = Agency(
        agency_id=agency_id,
        agency_name=agency_name,
        agency_url=agency_url,
        agency_timezone=agency_timezone,
        agency_lang=agency_lang,
        agency_phone=agency_phone,
        agency_fare_url=agency_fare_url,
        agency_email=agency_email,
    )
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
    "agency_id,agency_name,agency_url,agency_timezone,agency_lang,agency_phone,agency_fare_url,agency_email",
    [
        (
            "1",
            "foo_agency",
            "https://foo-agency.local",
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
        ),
        (
            None,
            "foo_agency",
            "https://foo-agency.local",
            "de",
            "german",
            "0123foo",
            "https://foo-agency.local",
            "test@foo-agency.local",
        ),
    ],
)
def test_agency_duplicate(
    agency_id: str,
    agency_name: str,
    agency_url: str,
    agency_timezone: str,
    agency_lang: str,
    agency_phone: str,
    agency_fare_url: str,
    agency_email: str,
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    original_object = Agency(
        agency_id=agency_id,
        agency_name=agency_name,
        agency_url=agency_url,
        agency_timezone=agency_timezone,
        agency_lang=agency_lang,
        agency_phone=agency_phone,
        agency_fare_url=agency_fare_url,
        agency_email=agency_email,
    )
    duplicate_object = Agency(
        agency_id=agency_id,
        agency_name=agency_name,
        agency_url=agency_url,
        agency_timezone=agency_timezone,
        agency_lang=agency_lang,
        agency_phone=agency_phone,
        agency_fare_url=agency_fare_url,
        agency_email=agency_email,
    )
    session.add(original_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
