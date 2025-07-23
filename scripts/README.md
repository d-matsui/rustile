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

For backward compatibility, these scripts remain in the project root:

- `setup_config.sh` - Configuration setup (use `dev-tools.sh setup`)
- `test_layout.sh` - Layout testing (use `dev-tools.sh layout`)
- `switch_layout.sh` - Layout switching (standalone utility)

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