#!/bin/bash

# Initial wait time in seconds
wait_time=1
max_attempts=4
attempt=0

while [ $attempt -lt $max_attempts ]
do
  # Try to fetch debian.org using curl
  curl -s -I debian.org > /dev/null 2>&1
  
  # Check if curl command was successful or not
  if [ $? -eq 0 ]
  then
    echo "Connection was successful!"
    exit 0
  else
    echo "Connection failed, trying again in $wait_time seconds..."
    sleep $wait_time
    # Increase the wait_time for next attempt
    wait_time=$((wait_time*2))
    attempt=$((attempt+1))
  fi
done

# If we're here, then all the attempts have failed
echo "All attempts to connect have failed. Please check your network settings."
exit 1
