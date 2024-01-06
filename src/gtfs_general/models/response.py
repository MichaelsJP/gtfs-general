# Create an enum for the health status
from enum import Enum

from pydantic import BaseModel


class HealthStatus(str, Enum):
    healthy = "healthy"
    unhealthy = "unhealthy"


# Create pydantic model for the health response
class HealthResponse(BaseModel):
    status: HealthStatus
