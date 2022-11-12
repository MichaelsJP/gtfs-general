import pytest
from sqlalchemy.exc import IntegrityError, PendingRollbackError
from sqlmodel import Session, select

from gtfs_general.db.factories import stops_factory
from gtfs_general.db.gtfs import Stops


@pytest.mark.parametrize(
    "stop_id,expect_to_fail",
    [("foo_stop_id", False), (None, True)],
)
def test_stops(
    stop_id: str,
    expect_to_fail: bool,
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    original_object = stops_factory(stop_id=stop_id)
    session.add(original_object)
    if expect_to_fail:
        with pytest.raises((IntegrityError, PendingRollbackError)):
            session.commit()
        session.rollback()
    else:
        session.commit()
        object_from_db = session.exec(select(Stops).where(Stops.stop_id == original_object.stop_id)).all()
        assert len(object_from_db) == 1
        assert object_from_db[0] == original_object


def test_stops_foreign_key(
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    first_object: Stops = stops_factory(stop_id="1_stop_id", parent_stop=None)
    second_object: Stops = stops_factory(stop_id="2_stop_id", parent_stop=first_object)
    third_object: Stops = stops_factory(stop_id="3_stop_id", parent_stop=first_object)
    session.add_all([first_object, second_object, third_object])
    session.commit()
    assert first_object == second_object.parent_stop
    assert first_object == third_object.parent_stop
    assert second_object in first_object.stations
    assert third_object in first_object.stations


def test_stops_duplicate_fail(
    in_memory_spatialite_session: Session,
) -> None:
    session: Session = in_memory_spatialite_session
    first_object: Stops = stops_factory(stop_id="1_stop_id", parent_stop=None)
    duplicate_object: Stops = stops_factory(stop_id="1_stop_id", parent_stop=None)
    session.add(first_object)
    session.commit()
    with pytest.raises(IntegrityError):
        session.add(duplicate_object)
        session.commit()
    session.rollback()
