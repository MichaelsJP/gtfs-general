from typing import Any

from fastapi import APIRouter

router = APIRouter()


@router.get("/login", response_model={})
def login_test_root() -> Any:
    """
    Test root location
    """
    return {"Success!"}
