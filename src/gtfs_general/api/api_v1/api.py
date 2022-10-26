from fastapi import APIRouter

from gtfs_general.api.api_v1.endpoints import login

api_router = APIRouter()
api_router.include_router(login.router, tags=["test"])
