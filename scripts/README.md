# Development Scripts

This directory contains development utilities for Rustile.

## Main Tool

### `dev-tools.sh` - Unified Development Interface

**All-in-one script for common development tasks:**

```bash
# Setup development environment
./scripts/dev-tools.sh setup

# Run comprehensive tests
./scripts/dev-tools.sh test

# Interactive layout testing
./scripts/dev-tools.sh layout

# Quality checks (fmt, clippy, test, docs)
./scripts/dev-tools.sh check

# Clean build artifacts
./scripts/dev-tools.sh clean

# Build release binary
./scripts/dev-tools.sh release
```

## Legacy Scripts (Root Directory)

The following standalone script remains in the project root:

- `switch_layout.sh` - Layout switching utility (used during testing)

## Usage Tips

**First time setup:**
```bash
./scripts/dev-tools.sh setup
./scripts/dev-tools.sh test
```

**Before committing:**
```bash
./scripts/dev-tools.sh check
```

**Testing layouts:**
```bash
./scripts/dev-tools.sh layout
# In another terminal:
./switch_layout.sh
```

All scripts include error handling and helpful output messages.