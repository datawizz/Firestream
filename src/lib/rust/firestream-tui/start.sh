#!/bin/bash
# Quick start script for Firestream TUI

set -e

echo "🚀 Starting Firestream TUI..."

# Check if running in development mode
if [ "$1" = "--dev" ] || [ "$FIRESTREAM_DEV" = "true" ]; then
    echo "📦 Running in development mode with mock backend..."
    export FIRESTREAM_MOCK_BACKEND=true
fi

# Check for required dependencies
echo "🔍 Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Build the TUI
echo "🔨 Building Firestream TUI..."
cargo build --release

# Create default config directory if it doesn't exist
CONFIG_DIR="${HOME}/.firestream"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "📁 Creating config directory: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
fi

# Create default config if it doesn't exist
CONFIG_FILE="${CONFIG_DIR}/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "📝 Creating default configuration..."
    cat > "$CONFIG_FILE" << 'EOF'
[default]
api_url = "http://localhost:8080/api/v1"
environment = "local-k3d"
theme = "dark"
refresh_interval = 5
mock_backend = true

[default.ui]
show_hidden_resources = false
default_namespace = "default"
log_tail_lines = 100
EOF
    echo "✅ Default configuration created at: $CONFIG_FILE"
fi

# Create log directory
LOG_DIR="${CONFIG_DIR}/logs"
mkdir -p "$LOG_DIR"

# Set up environment
export FIRESTREAM_CONFIG="$CONFIG_FILE"
export FIRESTREAM_LOG_DIR="$LOG_DIR"
export RUST_LOG="${RUST_LOG:-info}"

# Show help if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "
Firestream TUI - Terminal User Interface for Kubernetes Resource Management

Usage: $0 [options]

Options:
  --dev           Run in development mode with mock backend
  --profile NAME  Use a specific configuration profile
  --api-url URL   Override the API URL
  --help, -h      Show this help message

Environment variables:
  FIRESTREAM_API_URL     API endpoint URL
  FIRESTREAM_API_KEY     API authentication key
  FIRESTREAM_PROFILE     Configuration profile to use
  FIRESTREAM_CONFIG      Path to config file
  FIRESTREAM_LOG_LEVEL   Logging level (debug, info, warn, error)

Examples:
  $0                     # Run with default configuration
  $0 --dev               # Run with mock backend
  $0 --profile production # Run with production profile
"
    exit 0
fi

# Parse command line arguments
ARGS=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            export FIRESTREAM_MOCK_BACKEND=true
            shift
            ;;
        --profile)
            export FIRESTREAM_PROFILE="$2"
            shift 2
            ;;
        --api-url)
            export FIRESTREAM_API_URL="$2"
            shift 2
            ;;
        *)
            ARGS="$ARGS $1"
            shift
            ;;
    esac
done

# Display startup information
echo "
╔═══════════════════════════════════════════════════════════╗
║                    Firestream TUI v1.0.0                  ║
╠═══════════════════════════════════════════════════════════╣
║ Environment: ${FIRESTREAM_PROFILE:-default}                              ║
║ API URL: ${FIRESTREAM_API_URL:-http://localhost:8080/api/v1}         ║
║ Config: $CONFIG_FILE                                      ║
║ Logs: $LOG_DIR                                            ║
╚═══════════════════════════════════════════════════════════╝

Starting TUI... Press '?' for help, 'q' to quit.
"

# Run the TUI
exec cargo run --release -- $ARGS
