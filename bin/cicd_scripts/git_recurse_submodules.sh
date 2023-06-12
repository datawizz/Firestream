#!/bin/bash

# Change to the root directory of your repository
cd /workspace

# Read each submodule from the .gitmodules file
while IFS= read -r line
do
    if [[ $line == \[submodule* ]]; then
        # Parse the submodule path
        read -r line
        path_line=${line##*=}
        path_line=${path_line%%\[*}
        path_line=${path_line##* }

        # Parse the branch
        read -r line
        branch_line=${line##*=}
        branch_line=${branch_line%%\[*}
        branch_line=${branch_line##* }

        echo "Processing submodule $path_line on branch $branch_line..."

        # Change to the submodule directory
        cd "$path_line"

        # Perform a shallow clone
        git pull --depth 1 origin "$branch_line"

        # Change back to the root directory
        cd ../..
    fi
done < .gitmodules
