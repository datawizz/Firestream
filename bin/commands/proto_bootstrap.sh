#!/bin/bash

# Default paths
SRC_PATH="/workspace/src/lib/proto"
DST_PATH="/workspace/src/lib/python/gen_proto/out"

# Function to print usage
print_usage() {
    echo "Usage: $0 [source_path] [destination_path]"
    echo "Default source path: $SRC_PATH"
    echo "Default destination path: $DST_PATH"
}

# Handle command line arguments
if [ "$1" ]; then
    SRC_PATH="$1"
fi

if [ "$2" ]; then
    DST_PATH="$2"
fi

# Check if source directory exists
if [ ! -d "$SRC_PATH" ]; then
    echo "Error: Source directory '$SRC_PATH' does not exist"
    exit 1
fi

# Create destination directory if it doesn't exist
mkdir -p "$DST_PATH"

# Function to compile proto files
compile_protos() {
    local src_dir="$1"
    local dst_dir="$2"
    local rel_path="$3"

    # Process all .proto files in current directory
    for proto in "$src_dir"/*.proto; do
        # Check if there are any .proto files
        if [ -f "$proto" ]; then
            echo "Compiling: $proto"

            # Create output directory structure
            if [ ! -z "$rel_path" ]; then
                mkdir -p "$dst_dir/$rel_path"
            fi

            # Run protoc command
            protoc --proto_path="$SRC_PATH" \
                   --python_out="$DST_PATH" \
                   "$proto"

            if [ $? -ne 0 ]; then
                echo "Error: Failed to compile $proto"
                exit 1
            fi
        fi
    done

    # Recursively process subdirectories
    for dir in "$src_dir"/*/; do
        if [ -d "$dir" ]; then
            # Get relative path for subdirectory
            local new_rel_path
            if [ -z "$rel_path" ]; then
                new_rel_path=$(basename "$dir")
            else
                new_rel_path="$rel_path/$(basename "$dir")"
            fi

            compile_protos "$dir" "$dst_dir" "$new_rel_path"
        fi
    done
}

echo "Starting proto compilation..."
echo "Source path: $SRC_PATH"
echo "Destination path: $DST_PATH"

# Start compilation from root directory
compile_protos "$SRC_PATH" "$DST_PATH" ""

echo "Proto compilation complete!"
