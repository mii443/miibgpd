#!/bin/bash
docker compose -f ./tests/docker-compose.yml build
docker compose -f ./tests/docker-compose.yml up -d
HOST_2_LOOPBACK_IP=10.200.220.3
docker compose -f ./tests/docker-compose.yml exec \
  -T host1 ping -c 5 $HOST_2_LOOPBACK_IP

TEST_RESULT=$?
if [ $TEST_RESULT -eq 0 ]; then
  echo "Integration tests passed"
else
  echo "Integration tests failed"
fi

docker compose -f ./tests/docker-compose.yml down

exit $TEST_RESULT

