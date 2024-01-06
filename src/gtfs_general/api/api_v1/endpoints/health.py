from typing import Any

from fastapi import APIRouter

from gtfs_general.models.response import HealthResponse, HealthStatus

router = APIRouter()


@router.get("/health", response_model=HealthResponse)
def health_endpoint() -> Any:
    """
    Health endpoint
    """
    return {"status": HealthStatus.healthy}
