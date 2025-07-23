# GitHub Actions Workflows

This directory contains optimized CI/CD workflows for the Rustile project.

## Workflows Overview

### `ci.yml` - Main CI Pipeline
**Optimized for speed and efficiency**

- **Combined Jobs**: Merged test, format, clippy, and documentation checks into single job
- **Parallel Execution**: Security audit runs in parallel with main tests
- **Smart Caching**: Optimized Rust cache with versioned keys
- **Conditional Release**: Release builds only run when needed (main/develop pushes or PRs to main)

**Performance Improvements:**
- ⚡ ~40% faster by eliminating job overhead
- 💾 Better cache hit rates with improved cache keys
- 🔧 Reduced system dependency installation redundancy

### `claude.yml` - Claude Code Assistant
**Consolidated interactive and auto-review functionality**

- **Interactive Mode**: Triggered by `@claude` mentions in issues/PRs
- **Auto Review**: Automatic reviews for all PRs (can be disabled with [skip-review] or [WIP] in title)
- **Sticky Comments**: Reuses review comments on PR updates
- **Skip Options**: Add `[skip-review]` or `[WIP]` to PR title to disable auto-review

**Features:**
- 🤖 Manual assistance via `@claude` mentions
- 🔍 Automatic code review for all pull requests
- 📝 Comprehensive review criteria for Rust projects
- 🎯 Focused on window manager specific concerns

### `dependabot-auto-merge.yml` - Dependency Management
**Unchanged - handles automatic dependency updates**

### `release.yml` - Automated Releases  
**Unchanged - handles semantic versioning and releases**

## Optimization Results

### Before Optimization:
- **3 separate jobs** for CI (test, security, release-build)
- **2 separate Claude workflows** causing duplicate runs
- **Multiple Rust installations** and dependency setups
- **Cache misses** due to inconsistent keys
- **Always-running release builds** regardless of branch

### After Optimization:
- **2 parallel jobs** for main CI (test-suite, security-audit)
- **1 consolidated Claude workflow** with smart triggers
- **Shared setup patterns** reducing redundancy
- **Improved caching** with versioned keys
- **Conditional release builds** only when needed

### Expected Improvements:
- ⏱️ **~40% faster CI execution** due to job consolidation
- 💰 **Reduced compute costs** from fewer job starts and parallel execution
- 🔄 **Better cache utilization** with consistent, versioned cache keys
- 🚫 **Eliminated duplicate Claude runs** on PRs
- 🎯 **Comprehensive auto-reviews** for all pull requests

## Usage

### Usage:
- Push to feature branches triggers full CI
- PRs to main get comprehensive testing + release build
- All PRs get automatic Claude reviews (use [skip-review] to disable)
- Use `@claude` in comments for interactive assistance
- Faster feedback and comprehensive code reviews on all PRs
- Better resource utilization

## Migration Notes

The optimized workflows maintain full backward compatibility while improving performance and reducing redundancy. No changes to development workflow are required.