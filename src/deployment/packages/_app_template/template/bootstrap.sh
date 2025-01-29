#!/bin/bash

# Variables (customize these)
IMAGE_NAME="my-app"
IMAGE_TAG="latest"
LOCAL_REPO="localhost:5000" # Replace with your local repo URL
FULL_IMAGE_NAME="$LOCAL_REPO/$IMAGE_NAME:$IMAGE_TAG"

# Function to display an error message and exit
function error_exit {
    echo "[ERROR] $1"
    exit 1
}

# Step 1: Build the Docker image
echo "Building Docker image: $FULL_IMAGE_NAME"
docker build -f docker/new_project_template/Dockerfile -t $FULL_IMAGE_NAME . || error_exit "Failed to build Docker image."

# Step 2: Push the Docker image to the local repository
echo "Pushing Docker image to local repository: $LOCAL_REPO"
docker push $FULL_IMAGE_NAME || error_exit "Failed to push Docker image."

# Step 3: Confirm success
echo "Docker image pushed successfully: $FULL_IMAGE_NAME"
