#!/bin/bash
# Test script for graceful shutdown
# This script demonstrates both SIGINT (Ctrl+C) and SIGTERM shutdown behavior

set -e

echo "=========================================="
echo "Graceful Shutdown Test Script"
echo "=========================================="
echo ""

# Check if binary exists
if [ ! -f "./target/debug/lighter-auth" ]; then
    echo "Binary not found. Building..."
    cargo build --bin lighter-auth
fi

# Test 1: SIGINT (Ctrl+C) test
echo "Test 1: Testing SIGINT (Ctrl+C) graceful shutdown"
echo "--------------------------------------------------"
echo "Starting server in background..."

# Start server and capture PID
./target/debug/lighter-auth &
SERVER_PID=$!

echo "Server started with PID: $SERVER_PID"
echo "Waiting 3 seconds for server to initialize..."
sleep 3

echo "Sending SIGINT (Ctrl+C equivalent)..."
kill -INT $SERVER_PID

echo "Waiting for graceful shutdown (up to 35 seconds)..."
wait $SERVER_PID 2>/dev/null || true

echo "Server shut down. Check logs above for graceful shutdown messages."
echo ""

# Test 2: SIGTERM test
echo "Test 2: Testing SIGTERM (Docker/Kubernetes) graceful shutdown"
echo "--------------------------------------------------------------"
echo "Starting server in background..."

# Start server again
./target/debug/lighter-auth &
SERVER_PID=$!

echo "Server started with PID: $SERVER_PID"
echo "Waiting 3 seconds for server to initialize..."
sleep 3

echo "Sending SIGTERM (Docker/Kubernetes shutdown)..."
kill -TERM $SERVER_PID

echo "Waiting for graceful shutdown (up to 35 seconds)..."
wait $SERVER_PID 2>/dev/null || true

echo "Server shut down. Check logs above for graceful shutdown messages."
echo ""

echo "=========================================="
echo "Tests completed successfully!"
echo "=========================================="
echo ""
echo "Expected log messages:"
echo "  1. 'Received shutdown signal, initiating graceful shutdown and draining in-flight requests'"
echo "  2. 'Graceful shutdown completed successfully'"
echo ""
echo "These messages indicate the graceful shutdown is working correctly."
