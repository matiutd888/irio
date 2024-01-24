#!/bin/bash
set -e

export GCP_ZONE=us-central1

./create_cluster.sh
./create_artifacts_repo.sh
./build_docker_images.sh -h v1 -n v1
./create_python_postgres_image.sh

cd kubegres

# most probably needs to be done once
# kubectl apply -f https://raw.githubusercontent.com/reactive-tech/kubegres/v1.17/kubegres.yaml

# Check that it works
# kubectl get all -n kubegres-system

# Check controller logs:
# kubectl logs pod/kubegres-controller-manager-999786dd6-74tmb -c manager -n kubegres-system -f

# Check a storage class exists in kubernetes
# kubectl get sc

kubectl apply -f custom-my-postgres-config.yaml
kubectl apply -f my-postgres-secret.yaml
kubectl apply -f my-postgres.yaml

# Check the created resources
# kubectl get pod,statefulset,svc,configmap,pv,pvc -o wide

cd ..

# Setup postgres pod that execute table creation
kubectl apply -f python-postgres-pod.yaml

cd notification
kubectl apply -f deployment.yaml
kubectl apply -f hpa.yaml

cd ..

cd healthcheck
kubectl apply -f deployment.yaml
kubectl apply -f hpa.yaml

cd ..
