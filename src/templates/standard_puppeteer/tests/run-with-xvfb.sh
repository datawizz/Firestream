#!/bin/bash
# Run tests with proper virtual display using xvfb-run

# Kill any existing Xvfb processes on display :99
pkill -f "Xvfb :99" 2>/dev/null || true

# Set display number
export DISPLAY=:99

# Check if xvfb-run is available
if command -v xvfb-run &> /dev/null; then
    echo "Running tests with xvfb-run for proper rendering..."
    
    # More comprehensive xvfb settings for better rendering
    XVFB_ARGS="-screen 0 1920x1080x24"
    XVFB_ARGS="$XVFB_ARGS -ac"                    # Disable access control
    XVFB_ARGS="$XVFB_ARGS -nolisten tcp"         # Security: don't listen on TCP
    XVFB_ARGS="$XVFB_ARGS -dpi 96"               # Standard DPI
    XVFB_ARGS="$XVFB_ARGS +extension GLX"        # Enable GLX for better rendering
    XVFB_ARGS="$XVFB_ARGS +extension RANDR"      # Enable RANDR for screen management
    XVFB_ARGS="$XVFB_ARGS +extension RENDER"     # Enable RENDER for better graphics
    XVFB_ARGS="$XVFB_ARGS -noreset"              # Don't reset after last client exits
    
    # Run with xvfb-run
    exec xvfb-run -a --server-args="$XVFB_ARGS" "$@"
else
    # Try to start Xvfb manually if xvfb-run is not available
    if command -v Xvfb &> /dev/null; then
        echo "Starting Xvfb manually on display $DISPLAY..."
        Xvfb $DISPLAY -screen 0 1920x1080x24 -ac -nolisten tcp -dpi 96 +extension GLX +extension RANDR +extension RENDER &
        XVFB_PID=$!
        
        # Wait for Xvfb to start
        sleep 2
        
        # Run the command
        "$@"
        EXIT_CODE=$?
        
        # Clean up Xvfb
        kill $XVFB_PID 2>/dev/null || true
        
        exit $EXIT_CODE
    else
        echo "Warning: Neither xvfb-run nor Xvfb found, running without virtual display"
        echo "Screenshots may not work properly without a display server!"
        exec "$@"
    fi
fi