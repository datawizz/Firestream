#!/usr/bin/env bash

# Proto generation script for multi-platform app template
# Generates TypeScript types from Protocol Buffer definitions

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROTO_DIR="${PROJECT_ROOT}/src/lib/proto"
PROTO_TYPES_DIR="${PROJECT_ROOT}/src/lib/typescript/proto-types"

echo -e "${BLUE}Proto Generation Script${NC}"
echo "======================"
echo ""

# Check if proto directory exists
if [ ! -d "${PROTO_DIR}" ]; then
    echo -e "${YELLOW}Proto directory not found: ${PROTO_DIR}${NC}"
    echo "Creating example proto structure..."
    mkdir -p "${PROTO_DIR}/example/v1"

    # Create example proto file
    cat > "${PROTO_DIR}/example/v1/messages.proto" << 'EOF'
syntax = "proto3";

package example.v1;

message ExampleRequest {
  string id = 1;
  string data = 2;
}

message ExampleResponse {
  bool success = 1;
  string message = 2;
}
EOF

    echo -e "${GREEN}✓ Created example proto file${NC}"
fi

# Check if proto-types package exists
if [ ! -d "${PROTO_TYPES_DIR}" ]; then
    echo -e "${RED}Error: proto-types package not found: ${PROTO_TYPES_DIR}${NC}"
    exit 1
fi

# Change to proto-types directory
cd "${PROTO_TYPES_DIR}"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing proto-types dependencies...${NC}"
    pnpm install
fi

# Run proto generation
echo -e "${BLUE}Generating TypeScript types from proto files...${NC}"
pnpm proto:generate

# Check if generation was successful
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ Proto generation completed successfully${NC}"

    # Count generated files
    if [ -d "src/generated" ]; then
        FILE_COUNT=$(find src/generated -name "*.ts" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$FILE_COUNT" -gt 0 ]; then
            echo -e "${GREEN}✓ Generated ${FILE_COUNT} TypeScript file(s)${NC}"
        else
            echo -e "${YELLOW}No proto files were processed${NC}"
        fi
    fi
else
    echo -e "${RED}✗ Proto generation failed${NC}"
    exit 1
fi
