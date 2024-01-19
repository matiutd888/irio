source credentials.sh
sudo docker pull postgres:latest
sudo docker stop postgress_container
sudo docker rm postgress_container
docker-compose up -d
