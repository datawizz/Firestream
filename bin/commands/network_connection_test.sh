#!/bin/bash

# Initial wait time in seconds
wait_time=1
max_attempts=4
attempt=0

while [ $attempt -lt $max_attempts ]
do
  # Try to ping debian.org
  ping -c 1 debian.org > /dev/null 2>&1
  
  # Check if ping command was successful or not
  if [ $? -eq 0 ]
  then
    echo "Ping was successful!"
    exit 0
  else
    echo "Ping failed, trying again in $wait_time seconds..."
    sleep $wait_time
    # Increase the wait_time for next attempt
    wait_time=$((wait_time*2))
    attempt=$((attempt+1))
  fi
done

# If we're here, then all the attempts have failed
echo "All attempts to ping have failed. Please check your network settings."
exit 1
