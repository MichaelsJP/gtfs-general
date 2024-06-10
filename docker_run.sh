#!/bin/bash
docker rm -f gtfs-general || true
docker build -t gtfs-general .
docker run --rm -it -v .:/app --name gtfs-general --user "$(id -u):$(id -g)" gtfs-general $@
