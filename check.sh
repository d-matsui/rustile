#!/bin/bash
# Check code quality

echo "=== Code Quality Check ==="
echo ""

# Format check
echo "Checking formatting..."
if cargo fmt -- --check; then
    echo "✓ Formatting OK"
else
    echo "✗ Formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi
echo ""

# Clippy check
echo "Running clippy..."
if cargo clippy -- -D warnings; then
    echo "✓ Clippy OK"
else
    echo "✗ Clippy warnings found"
    exit 1
fi
echo ""

# Test check
echo "Running tests..."
if cargo test --quiet; then
    echo "✓ Tests passed"
else
    echo "✗ Tests failed"
    exit 1
fi
echo ""

# Doc check
echo "Checking documentation..."
if cargo doc --quiet --no-deps; then
    echo "✓ Documentation OK"
else
    echo "✗ Documentation issues"
    exit 1
fi

echo ""
echo "=== All checks passed! ==="