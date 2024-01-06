import pytest

from gtfs_general.models.response import HealthResponse, HealthStatus


def test_healthy_response() -> None:
    response = HealthResponse(status=HealthStatus.healthy)
    assert response.status == "healthy"
    assert response.model_dump() == {"status": "healthy"}


def test_unhealthy_response() -> None:
    response = HealthResponse(status=HealthStatus.unhealthy)
    assert response.status == "unhealthy"
    assert response.model_dump() == {"status": "unhealthy"}


def test_invalid_response() -> None:
    with pytest.raises(ValueError):
        HealthResponse(status="invalid")
