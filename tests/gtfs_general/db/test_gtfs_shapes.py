import pytest
from sqlalchemy.exc import PendingRollbackError  # type: ignore
from sqlalchemy.exc import IntegrityError
from sqlmodel import Session, select

from gtfs_general.db.factories import calendar_factory, routes_factory, shapes_factory, trips_factory
from gtfs_general.db.gtfs import Calendar, Routes, Shapes


@pytest.mark.parametrize(
    "shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence,shape_dist_traveled,expect_to_fail",
    [
        ("1_shape_id", 9.3403203, 46.404040, 1, 22.22, False),
        ("2_shape_id", 9.3403203, 46.404040, 1, None, False),
        ("3_shape_id", None, 46.404040, 1, 22.22, True),
        ("4_shape_id", 9.3403203, None, 1, 22.22, True),
        ("5_shape_id", 9.3403203, 46.404040, -1, 22.22, True),
        # Todo bug in sqlmodel("6_shape_id", 9.3403203, 46.404040, 1, -1.4, True),
        (None, 9.3403203, 46.404040, 1, 22.22, True),
    ],
)
def test_shapes(
    shape_id: str,
    shape_pt_lat: float,
    shape_pt_lon: float,
    shape_pt_sequence: int,
    shape_dist_traveled: float,
    expect_to_fail: bool,
    in_memory_spatialite_function_session: Session,
) -> None:
    session: Session = in_memory_spatialite_function_session
    original_object: Shapes = shapes_factory(
        shape_id=shape_id,
        shape_pt_lat=shape_pt_lat,
        shape_pt_lon=shape_pt_lon,
        shape_pt_sequence=shape_pt_sequence,
        shape_dist_traveled=shape_dist_traveled,
    )
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Shapes).where(Shapes.shape_id == original_object.shape_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object


def test_shapes_foreign_key(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    route: Routes = routes_factory(route_id="1_test_route")
    calendar: Calendar = calendar_factory(service_id="1_test_service")
    session.add_all([route, calendar])
    session.commit()
    shape: Shapes = shapes_factory(shape_id="1_test_shape")
    session.add(shape)
    session.commit()

    trip = trips_factory(
        route_id=route.route_id, service_id=calendar.service_id, trip_id="1_trip_id", shape_id=shape.shape_id
    )
    session.add(trip)
    session.commit()
    assert shape == trip.shape
    assert shape.shape_id == trip.shape_id
    assert route.trips[0].shape == shape
    assert calendar.trips[0].shape == shape


def test_shapes_duplicate_fail(in_memory_spatialite_function_session: Session) -> None:
    session: Session = in_memory_spatialite_function_session
    first_object: Shapes = shapes_factory(shape_id="1_test_shape")
    duplicate_object: Shapes = shapes_factory(shape_id="1_test_shape")
    session.add(first_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
