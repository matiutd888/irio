#!/bin/bash

#build notification service in release mode
cd ../services/notification
cargo build --release

cd ../../notification_dockerfile
cp ../target/release/notification .

#create docker image of it
sudo docker build -t notification-service-image:latest .

#cleanup
rm -f notification
