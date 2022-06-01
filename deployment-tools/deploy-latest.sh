#!/bin/bash

set -euo pipefail

print() {
  GREEN="\033[0;32m"
  NO_COLOR='\033[0m'
  echo -e "${GREEN}${1}${NO_COLOR}"
}

# This tag will be assigned to the latest image.
IMAGE_ID="traffic-maker:latest"
# Default container name.
CONTAINER_ID="traffic-maker"

print "[+] Logging into AWS ECR..."
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 573243519133.dkr.ecr.us-east-1.amazonaws.com

print "[+] Obtaining latest image tag..."
TAG=$(aws --region us-east-1 ecr describe-images --repository-name traffic-maker --registry-id 573243519133 --query 'sort_by(imageDetails,& imagePushedAt)[-1].imageTags[0]' | tr -d \")
print "[+] Latest image tag is <$TAG>"

print "[+] Pulling latest image..."
docker pull 573243519133.dkr.ecr.us-east-1.amazonaws.com/traffic-maker:"$TAG"
docker tag 573243519133.dkr.ecr.us-east-1.amazonaws.com/traffic-maker:"$TAG" $IMAGE_ID
print "[+] Image pulled and tagged as <$IMAGE_ID>"

if [ $(docker ps | grep "$CONTAINER_ID" | wc -l) -gt 0 ]; then
  print "[+] Stopping currently launched container..."
  docker stop $CONTAINER_ID
fi

if [ $(docker ps -a | grep "$CONTAINER_ID" | wc -l) -gt 0 ]; then
  print "[+] Removing previously launched container..."
  docker rm $CONTAINER_ID
fi

print "[+] Running image <$IMAGE_ID> in container <$CONTAINER_ID>..."
docker run \
  -d \
  --network host \
  --mount type=bind,src="$(pwd)"/Timetable.toml,dst=/traffic-maker/Timetable.toml \
  --name $CONTAINER_ID \
  $IMAGE_ID
