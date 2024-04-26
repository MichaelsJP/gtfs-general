FROM python:3.11-buster as builder

RUN apt-get update && \
    apt-get install --no-install-suggests --no-install-recommends --yes pipx python3-venv
ENV PATH="/root/.local/bin:${PATH}"

RUN pipx install poetry
RUN pipx inject poetry poetry-plugin-bundle

WORKDIR /app

COPY ./ ./

RUN poetry bundle venv --python=/usr/local/bin/python --only=main /venv

WORKDIR /work

ENTRYPOINT ["/venv/bin/gtfs-general"]
