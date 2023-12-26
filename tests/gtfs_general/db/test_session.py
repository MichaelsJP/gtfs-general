from sqlmodel import Session


def test_get_session(in_memory_spatialite_function_session: Session) -> None:
    assert isinstance(in_memory_spatialite_function_session, Session)
