import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db.factories import calendar_factory, routes_factory, shapes_factory, trips_factory
from gtfs_general.db.gtfs import Calendar, Routes, Shapes, Trips


@pytest.mark.parametrize(
    "trip_id,set_route,set_service,set_shape,expect_to_fail",
    [
        ("foo_trip_id", True, True, True, False),
        ("foo_trip_id", False, True, True, True),
        ("foo_trip_id", True, False, True, True),
        ("foo_trip_id", True, True, False, False),
        ("foo_trip_id", True, False, False, True),
        (None, True, True, True, True),
    ],
)
def test_trips(
    trip_id: str,
    set_route: bool,
    set_service: bool,
    set_shape: bool,
    expect_to_fail: bool,
    in_memory_spatialite_function_session: Session,
) -> None:
    session: Session = in_memory_spatialite_function_session
    route: Routes = routes_factory(route_id=f"{trip_id}_test_route")
    calendar: Calendar = calendar_factory(service_id=f"{trip_id}_test_service")
    shape: Shapes = shapes_factory(shape_id=f"{trip_id}_test_shape")
    session.add_all([route, calendar, shape])
    session.commit()
    original_object = trips_factory(
        route_id=route.route_id if set_route else None,
        service_id=calendar.service_id if set_service else None,
        trip_id=trip_id,
        shape_id=shape.shape_id if set_shape else None,
    )
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Trips).where(Trips.trip_id == original_object.trip_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object
        assert original_object.route == route
        assert original_object.service == calendar
        if set_shape:
            assert original_object.shape == shape
        else:
            assert original_object.shape is None
            assert original_object.shape_id is None


@pytest.mark.parametrize(
    "trip_id,set_shape",
    [("1_trip_id", True), ("2_trip_id", False)],
)
def test_trips_foreign_key(trip_id: str, set_shape: bool, in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    route: Routes = routes_factory(route_id=f"{trip_id}_test_route")
    calendar: Calendar = calendar_factory(service_id=f"{trip_id}_test_service")
    shape: Shapes = shapes_factory(shape_id=f"{trip_id}_test_shape")
    session.add_all([route, calendar, shape])
    session.commit()
    original_object = trips_factory(
        route_id=route.route_id,
        service_id=calendar.service_id,
        trip_id=trip_id,
        shape_id=shape.shape_id if set_shape else None,
    )
    session.add(original_object)

    assert original_object in route.trips
    assert original_object in calendar.trips
    if set_shape:
        assert original_object in shape.trips
    else:
        assert original_object not in shape.trips


def test_stops_duplicate_fail(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    route: Routes = routes_factory(route_id="1_test_route")
    calendar: Calendar = calendar_factory(service_id="1_test_service")
    shape: Shapes = shapes_factory(shape_id="1_test_shape")
    session.add_all([route, calendar, shape])
    session.commit()
    original_object = trips_factory(
        route_id=route.route_id, service_id=calendar.service_id, trip_id="1", shape_id=shape.shape_id
    )
    duplicate_object = trips_factory(
        route_id=route.route_id, service_id=calendar.service_id, trip_id="1", shape_id=shape.shape_id
    )
    session.add(original_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
