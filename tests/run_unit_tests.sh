#!/bin/bash
docker compose -f ./tests/docker-compose.yml build
docker compose -f ./tests/docker-compose.yml up -d
docker compose -f ./tests/docker-compose.yml exec \
  -T host2 cargo test -- --test-threads=1 --nocapture
docker compose -f ./tests/docker-compose.yml down
