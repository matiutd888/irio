#!/bin/bash
set -e

ARTIFACTS_REPO_NAME=alerting-platform

if [ "x$GCP_ZONE" == "x" ]
then
  exit 1
fi

docker buildx build --file ../python_postgres_setup/Dockerfile \
-t $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/python-postgres:v1 ../python_postgres_setup/
docker push $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/python-postgres:v1

