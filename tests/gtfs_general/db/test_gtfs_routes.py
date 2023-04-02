import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db.factories import agency_factory, routes_factory
from gtfs_general.db.gtfs import Agency, Routes


@pytest.mark.parametrize(
    "route_id,agency_id,expect_to_fail",
    [
        ("1_stop_id", None, False),
        ("2_stop_id", "foo_agency_id", False),
        (None, "foo_agency_id", True),
        (None, None, True),
    ],
)
def test_routes(
    route_id: str, agency_id: str, expect_to_fail: bool, in_memory_spatialite_function_session: Session
) -> None:
    session: Session = in_memory_spatialite_function_session
    original_object: Routes = routes_factory(route_id=route_id, agency_id=agency_id)
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Routes).where(Routes.route_id == original_object.route_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object


def test_routes_foreign_key(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    agency: Agency = agency_factory(agency_name="1_agency", agency_id="1_agency")
    session.add(agency)
    session.commit()
    first_object: Routes = routes_factory(route_id="1_route_id", agency=None)
    second_object: Routes = routes_factory(route_id="2_route_id", agency=agency)
    third_object: Routes = routes_factory(route_id="3_route_id", agency=agency)
    session.add_all([first_object, second_object, third_object])
    session.commit()
    session.refresh(second_object)
    assert all(test in agency.routes for test in [second_object, third_object])
    assert first_object not in agency.routes
    assert first_object.agency is None
    assert first_object.agency_id is None
    assert second_object.agency == agency
    assert second_object.agency_id == agency.agency_id
    assert third_object.agency == agency
    assert third_object.agency_id == agency.agency_id


def test_routes_duplicate_fail(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    first_object: Routes = routes_factory(route_id="1_route_id", agency=None)
    duplicate_object: Routes = routes_factory(route_id="1_route_id", agency=None)
    session.add(first_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
