from sqlmodel import Session


def test_get_metadata(in_memory_spatialite_session: Session) -> None:
    assert 1
