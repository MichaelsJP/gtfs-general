import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db.factories import (
    calendar_factory,
    routes_factory,
    shapes_factory,
    stop_times_factory,
    stops_factory,
    trips_factory,
)
from gtfs_general.db.gtfs import Calendar, Routes, Shapes, Stops, StopTimes, Trips


@pytest.mark.parametrize(
    "stop_sequence,shape_dist_traveled,expect_to_fail",
    [
        (1, 3, False),
        (-1, 3, True),
        # (1, -3, True),# TODO Condecimal not working properly for sqlmodel
        (1, None, False),
        (None, 1, True),
        (None, None, True),
    ],
)
def test_stop_times(
    stop_sequence: int,
    shape_dist_traveled: int,
    expect_to_fail: bool,
    in_memory_spatialite_function_session: Session,
) -> None:
    session: Session = in_memory_spatialite_function_session
    route: Routes = routes_factory(route_id="foo_test_route")
    calendar: Calendar = calendar_factory(service_id="foo_test_service")
    trip: Trips = trips_factory(route_id=route.route_id, service_id=calendar.service_id, trip_id="foo_trip_id")
    stop: Stops = stops_factory(stop_id="foo_stop_id")

    session.add_all([route, calendar, trip, stop])
    session.commit()

    original_object: StopTimes = stop_times_factory(
        trip_id=trip.trip_id, stop_id=stop.stop_id, stop_sequence=stop_sequence, shape_dist_traveled=shape_dist_traveled
    )
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(
            select(StopTimes).where(
                (StopTimes.trip_id == original_object.trip_id)
                & (StopTimes.stop_id == original_object.stop_id)
                & (StopTimes.stop_sequence == original_object.stop_sequence)
            )
        ).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object
        assert original_object.trip == trip
        assert original_object.stop == stop


# TODO implement the other two tests


@pytest.mark.parametrize(
    "trip_id,set_shape",
    [("1_trip_id", True), ("2_trip_id", False)],
)
def test_stop_times_foreign_key(trip_id: str, set_shape: bool, in_memory_spatialite_function_session: Session) -> None:
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


# fails
# Duplicate stop sequence
#


def test_stop_times_duplicate_fail(in_memory_spatialite_function_session: Session) -> None:
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
