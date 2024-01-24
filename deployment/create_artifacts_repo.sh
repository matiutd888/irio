#!/bin/bash
set -e


display_usage() {
    echo "End-to-end deployment script for scylla on GKE."
    echo "usage: $0 (GCP_ZONE env var must be declared)"
}

ARTIFACTS_REPO_NAME=alerting-platform

if [ "x$GCP_ZONE" == "x" ]
then
    display_usage
    exit 1
fi

echo "Creating artifcats repo.."


gcloud artifacts repositories create \
$ARTIFACTS_REPO_NAME \
--project $GOOGLE_CLOUD_PROJECT \
--repository-format docker \
--location $GCP_ZONE

echo "possibly $ARTIFACTS_REPO_NAME created"

