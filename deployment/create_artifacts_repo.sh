
display_usage() {
	echo "End-to-end deployment script for scylla on GKE."
	echo "usage: $0 -z|--gcp-zone [GCP zone]"
}

ARTIFACTS_REPO_NAME=alerting-platform

while (( "$#" )); do
  case "$1" in
    -z|--gcp-zone)
      if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
        GCP_ZONE=$2
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

