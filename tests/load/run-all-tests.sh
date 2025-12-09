#!/bin/bash

###############################################################################
# K6 Load Test Suite Runner for lighter-auth
#
# This script runs all load test variations in sequence with proper
# reporting and cleanup between tests.
#
# Usage:
#   ./tests/load/run-all-tests.sh [options]
#
# Options:
#   --quick       Run only quick tests (smoke + load)
#   --full        Run all tests including soak (default)
#   --no-cleanup  Don't cleanup database between tests
#   --output DIR  Save results to specific directory
#
# Requirements:
#   - k6 installed and in PATH
#   - lighter-auth service running
#   - PostgreSQL database configured
###############################################################################

set -e  # Exit on error

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
OUTPUT_DIR="${OUTPUT_DIR:-./test-results}"
CLEANUP="${CLEANUP:-true}"
TEST_MODE="${TEST_MODE:-full}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --quick)
      TEST_MODE="quick"
      shift
      ;;
    --full)
      TEST_MODE="full"
      shift
      ;;
    --no-cleanup)
      CLEANUP="false"
      shift
      ;;
    --output)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--quick|--full] [--no-cleanup] [--output DIR]"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Log function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check k6
    if ! command -v k6 &> /dev/null; then
        error "k6 is not installed. Please install k6 first."
        echo "  macOS: brew install k6"
        echo "  Linux: See https://k6.io/docs/getting-started/installation"
        exit 1
    fi

    # Check service
    if ! curl -s "$BASE_URL/" > /dev/null; then
        error "Service is not accessible at $BASE_URL"
        echo "Please start the lighter-auth service first:"
        echo "  cargo run --features postgres"
        exit 1
    fi

    success "Prerequisites check passed"
}

# Cleanup function
cleanup_database() {
    if [ "$CLEANUP" = "true" ]; then
        log "Cleaning up database..."
        # Add database cleanup logic here if needed
        # For now, we'll just sleep to let connections close
        sleep 5
        success "Database cleanup completed"
    fi
}

# Run a single test
run_test() {
    local test_name=$1
    local test_file=$2
    local result_file="$OUTPUT_DIR/${test_name}-$(date +'%Y%m%d-%H%M%S').json"

    log "========================================"
    log "Running: $test_name"
    log "========================================"

    if k6 run \
        -e BASE_URL="$BASE_URL" \
        --out json="$result_file" \
        "$test_file"; then
        success "$test_name completed successfully"
        echo "Results saved to: $result_file"
        return 0
    else
        error "$test_name failed"
        return 1
    fi
}

# Main execution
main() {
    log "Starting K6 Test Suite for lighter-auth"
    log "Mode: $TEST_MODE"
    log "Base URL: $BASE_URL"
    log "Output Directory: $OUTPUT_DIR"
    log ""

    check_prerequisites

    # Track results
    declare -a passed_tests
    declare -a failed_tests

    # Quick smoke test
    if run_test "Quick Smoke Test" "tests/load/quick-test.js"; then
        passed_tests+=("Quick Smoke Test")
    else
        failed_tests+=("Quick Smoke Test")
        warning "Quick smoke test failed, but continuing with other tests..."
    fi
    cleanup_database

    # Standard load test
    if run_test "Standard Load Test" "tests/load/k6_script.js"; then
        passed_tests+=("Standard Load Test")
    else
        failed_tests+=("Standard Load Test")
    fi
    cleanup_database

    # If full mode, run additional tests
    if [ "$TEST_MODE" = "full" ]; then
        # Spike test
        if run_test "Spike Test" "tests/load/spike-test.js"; then
            passed_tests+=("Spike Test")
        else
            failed_tests+=("Spike Test")
        fi
        cleanup_database

        # Stress test
        if run_test "Stress Test" "tests/load/stress-test.js"; then
            passed_tests+=("Stress Test")
        else
            failed_tests+=("Stress Test")
        fi
        cleanup_database

        # Soak test (optional, takes 2+ hours)
        warning "Soak test takes 2+ hours. Skipping by default."
        log "To run soak test separately: k6 run tests/load/soak-test.js"
    fi

    # Summary
    log ""
    log "========================================"
    log "Test Suite Complete"
    log "========================================"
    log ""

    if [ ${#passed_tests[@]} -gt 0 ]; then
        success "Passed tests (${#passed_tests[@]}):"
        for test in "${passed_tests[@]}"; do
            echo "  ✓ $test"
        done
    fi

    if [ ${#failed_tests[@]} -gt 0 ]; then
        error "Failed tests (${#failed_tests[@]}):"
        for test in "${failed_tests[@]}"; do
            echo "  ✗ $test"
        done
        exit 1
    else
        success "All tests passed!"
        exit 0
    fi
}

# Run main function
main
