#!/bin/bash
# Quick test script to verify compilation

echo "Testing Firestream compilation..."

# First, check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo is not installed"
    exit 1
fi

# Run cargo check for faster compilation test
echo "Running cargo check..."
if cargo check; then
    echo "✅ Cargo check passed!"
else
    echo "❌ Cargo check failed"
    exit 1
fi

# Run cargo test
echo "Running cargo test..."
if cargo test; then
    echo "✅ All tests passed!"
else
    echo "❌ Tests failed"
    exit 1
fi

echo "✅ All checks passed successfully!"
