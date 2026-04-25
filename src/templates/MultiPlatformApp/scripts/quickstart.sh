#!/usr/bin/env bash

# Quick start script for multi-platform app template
# Helps new users get started quickly

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Multi-Platform App Template - Quick Start${NC}"
echo "=========================================="
echo ""

# Check for required tools
echo -e "${BLUE}Checking required tools...${NC}"

# Check Node.js
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo -e "${GREEN}✓ Node.js ${NODE_VERSION}${NC}"
else
    echo -e "${RED}✗ Node.js not found${NC}"
    echo "  Install from: https://nodejs.org/"
    exit 1
fi

# Check pnpm
if command -v pnpm &> /dev/null; then
    PNPM_VERSION=$(pnpm --version)
    echo -e "${GREEN}✓ pnpm ${PNPM_VERSION}${NC}"
else
    echo -e "${YELLOW}✗ pnpm not found${NC}"
    echo "  Installing pnpm..."
    npm install -g pnpm
fi

# Check Rust
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}✓ Rust ${RUST_VERSION}${NC}"
else
    echo -e "${YELLOW}✗ Rust not found (required for desktop app)${NC}"
    echo "  Install from: https://rustup.rs/"
fi

# Platform-specific checks
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}macOS detected - checking iOS tools...${NC}"

    if command -v xcodegen &> /dev/null; then
        echo -e "${GREEN}✓ XcodeGen installed${NC}"
    else
        echo -e "${YELLOW}✗ XcodeGen not found (optional for iOS development)${NC}"
        echo "  Install with: brew install xcodegen"
    fi
fi

echo ""
echo -e "${BLUE}Installing dependencies...${NC}"
pnpm install

echo ""
echo -e "${GREEN}✓ Setup complete!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Start web app:     ${YELLOW}make run-web-dev${NC}"
echo "  2. Start desktop app: ${YELLOW}make run-desktop-dev${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  3. Start iOS app:     ${YELLOW}make run-ios-dev${NC}"
fi
echo ""
echo "Run '${YELLOW}make help${NC}' to see all available commands"
