#!/bin/bash

# Elasticsearch (and OpenSearch) requires an increase in the vm.max_map_count kernel setting

# Current value of vm.max_map_count
current_value=$(sysctl -n vm.max_map_count)

# Desired value of vm.max_map_count
desired_value=262144

echo "Current vm.max_map_count value: $current_value"

# Check if the current_value is less than the desired_value
if [ "$current_value" -lt "$desired_value" ]
then
    echo "Increasing vm.max_map_count value to $desired_value..."

    # Increase the vm.max_map_count value immediately
    sudo sysctl -w vm.max_map_count=$desired_value

    # Ensure the new setting persists after reboot
    echo 'vm.max_map_count=262144' | sudo tee -a /etc/sysctl.conf

    # Verify that the change has been made
    new_value=$(sysctl -n vm.max_map_count)

    if [ "$new_value" -eq "$desired_value" ]
    then
        echo "Successfully set vm.max_map_count to $new_value"
    else
        echo "Failed to set vm.max_map_count. Current value is $new_value"
    fi
else
    echo "vm.max_map_count is already $current_value, which is not less than $desired_value. No changes made."
fi
