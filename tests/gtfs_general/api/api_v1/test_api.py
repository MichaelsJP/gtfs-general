import pytest

from gtfs_general.api.api_v1.api import api_router


# Parametrize path, name, methods
@pytest.mark.parametrize(
    "path, name, methods, tags",
    [
        ("/health", "health_endpoint", {"GET"}, ["health"]),
    ],
)
def test_api_router(path: str, name: str, methods: dict, tags: list) -> None:
    api_router_test = api_router
    assert api_router_test.__class__.__name__ == "APIRouter"
    endpoint_found: bool = False
    for route in api_router_test.routes:
        if route.name == name:
            endpoint_found = True
            assert route.path == path
            assert route.name == name
            assert route.methods == methods
            assert route.tags == tags
            assert route.__class__.__name__ == "APIRoute"
    assert endpoint_found
