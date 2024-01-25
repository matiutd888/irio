
#!/bin/bash
set -e

display_usage() {
    echo "script that deploys configuration service."
    echo "usage: $0 -v|--version <version of configuration service>"
}


while (( "$#" )); do
    case "$1" in
        -v|--version)
            if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                CONFIGURATION_VERSION=$2
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


if [ "x$GCP_ZONE" == "x" ]
then
    echo "set GCP_ZONE"
    exit 1
fi

if [ "x$CONFIGURATION_VERSION" == "x" ]
then
    display_usage
    exit 1
fi


echo "building configuration:$CONFIGURATION_VERSION image..."

docker buildx build --file ../configuration_scripts/Dockerfile \
-t $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/configuration:$CONFIGURATION_VERSION ../configuration_scripts/

echo "configuration image built"

docker push $GCP_ZONE-docker.pkg.dev/${GOOGLE_CLOUD_PROJECT}/$ARTIFACTS_REPO_NAME/configuration:$CONFIGURATION_VERSION

echo "pushed to repo"

echo "#############################"
echo "Applying deployment"

kubectl apply -f configuration-service/deployment.yaml

echo "Exposing port 8000"
kubectl port-forward service/configuration-svc 8000:8000
