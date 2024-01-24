#!/bin/bash
set -e


display_usage() {
    echo "End-to-end deployment script for scylla on GKE."
    echo "usage: $0 -n|--notification-version <version of notification service> -h|--healthcheck-version <version of healthcheck service>"
}

while (( "$#" )); do
  case "$1" in
    -n|--notification-version)
      if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
        NOTIFICATION_VERSION=$2
        shift 2
      else
        echo "Error: Argument for $1 is missing" >&2
        exit 1
      fi
      ;;
    -h|--healthcheck-version)
      if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
        HEALTHCHECK_VERSION=$2
        shift 2
      else
        echo "Error: Argument for $1 is missing" >&2
        exit 1
      fi
      ;;
    -*|--*=) # unsupported flags
      echo "Error: Unsupported flag $1" >&2
      exit 1
      ;;
    *) # preserve positional arguments
      PARAMS="$PARAMS $1"
      shift
      ;;
  esac
done


ARTIFACTS_REPO_NAME=alerting-platform


if [ "x$NOTIFICATION_VERSION" == "x" ]
then
  display_usage
  exit 1
fi

if [ "x$HEALTHCHECK_VERSION" == "x" ]
then
  display_usage
  exit 1
fi


echo "Building images..."

echo "building notification:$NOTIFICATION_VERSION image..."

docker build \
-t $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/notification:$NOTIFICATION_VERSION \
--file services/notification/Dockerfile

echo "notification image built"

echo "building healthcheck:$HEALTHCHECK_VERSION image..."

docker build \
-t $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/healthcheck:$HEALTHCHECK_VERSION \
--file services/healhcheck/Dockerfile

echo "healthcheck image built"

echo "pushing images to artifacts repo.."

docker push us-central1-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/notification:$NOTIFICATION_VERSION
docker push us-central1-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/healthcheck:$HEALTHCHECK_VERSION


echo "pushing images to artifacts repo done"