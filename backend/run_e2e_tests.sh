#!/bin/bash

# PlantTracker E2E Test Runner
# This script sets up the environment and runs the end-to-end test suite

set -e

echo "🧪 PlantTracker E2E Test Runner"
echo "================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the backend directory"
    exit 1
fi

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: Python 3 is required but not installed"
    exit 1
fi

# Check if Rust/Cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is required but not installed"
    exit 1
fi

# Create and activate virtual environment
echo "📦 Setting up Python virtual environment..."
if [ ! -d "venv-e2e" ]; then
    python3 -m venv venv-e2e
fi

# Activate virtual environment
source venv-e2e/bin/activate

# Install Python dependencies
echo "📦 Installing Python dependencies..."
if [ -f "requirements-e2e.txt" ]; then
    pip install -r requirements-e2e.txt --quiet
else
    pip install requests pytest --quiet
fi

# Build the backend in release mode for better performance
echo "🔨 Building backend in release mode..."
cargo build --release --quiet

# Clean up any existing test database
echo "🧹 Cleaning up test environment..."
rm -f test_plant_tracker.db

# Run the e2e tests
echo "🚀 Running E2E tests..."
echo ""

# Set environment variables for the test
export DATABASE_URL="sqlite://test_plant_tracker.db"
export RUST_LOG="info"

# Run the Python test suite with virtual environment
venv-e2e/bin/python -m pytest test_e2e_pytest.py -v

echo ""
echo "✅ E2E tests completed successfully!"
echo ""
echo "To run tests manually:"
echo "  source venv-e2e/bin/activate"
echo "  python e2e_tests.py"
echo ""
echo "To run tests with custom port:"
echo "  # Modify the PlantTrackerE2ETest(backend_port=XXXX) in e2e_tests.py"