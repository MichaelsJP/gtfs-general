import pathlib

from fastapi.testclient import TestClient
from typer.testing import CliRunner

from gtfs_general.config import Settings

runner = CliRunner()

script_path = pathlib.Path(__file__).parent.resolve()


def test_server(test_client: TestClient) -> None:
    response = test_client.get(f"{Settings().api_v1_str}/health")
    assert response.status_code == 200
    assert response.json() == {"status": "healthy"}
