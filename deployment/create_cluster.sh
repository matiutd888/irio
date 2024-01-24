#!/bin/bash
set -e

#########
# Start #
#########
# https://cloud.google.com/kubernetes-engine/docs/how-to/creating-an-autopilot-cluster


display_usage() {
    echo "End-to-end deployment script for scylla on GKE."
    echo "usage: $0 -c|--k8s-cluster-name [cluster name (optional)]"
}

CLUSTER_NAME="alerting-cluster"

while (( "$#" )); do
    case "$1" in
        -c|--k8s-cluster-name)
            if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                CLUSTER_NAME=$2
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

if [ "x$GCP_ZONE" == "x" ]
then
    echo "GCP_ZONE must be declared"
    exit 1
fi


check_cluster_readiness() {
    until [[ "$(gcloud container clusters list | grep ${CLUSTER_NAME} | awk '{ print $8 }')" == "RUNNING" ]]; do
        echo "Waiting for cluster readiness... "
        echo $(gcloud container clusters list | grep ${CLUSTER_NAME})
        sleep 10
        WAIT_TIME=$((WAIT_TIME+10))
        if [[  "$(gcloud container operations list --sort-by=START_TIME --filter="${CLUSTER_NAME} AND UPGRADE_MASTER" | grep RUNNING)" != "" ]]; then
            gcloud container operations list --sort-by=START_TIME --filter="${CLUSTER_NAME} AND UPGRADE_MASTER"
            gcloud container operations wait $(gcloud container operations list --sort-by=START_TIME --filter="${CLUSTER_NAME} AND UPGRADE_MASTER" | tail -1 | awk '{print $1}')
        else
            gcloud container operations list --sort-by=START_TIME --filter="${CLUSTER_NAME} AND UPGRADE_MASTER" | tail -1
        fi
    done
    gcloud container clusters list | grep "${CLUSTER_NAME}"
}

gcloud container clusters create-auto $CLUSTER_NAME \
--location=$GCP_ZONE \
--project=$GOOGLE_CLOUD_PROJECT



# echo "Waiting for gke to create cluster"
# sleep 120
# check_cluster_readiness
# echo "cluster ${CLUSTER_NAME} created and ready"
