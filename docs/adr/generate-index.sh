#!/bin/bash

# Generate ADR INDEX.md from Status fields

echo "# ADR Index"
echo ""
echo "Architecture Decision Records for Rustile window manager."
echo ""

# Collect ADRs by status (arrays to hold markdown list items)
declare -a accepted_adrs=()   # ["- **[ADR-001: ...](...)**", "- **[ADR-002: ...](...)**"]
declare -a proposed_adrs=()   # ["- **[ADR-012: ...](...)**"]
declare -a deprecated_adrs=() # ["- **[ADR-003: ...](...)**"]

# Process each ADR file (exclude template)
for file in [0-9][0-9][0-9]*.md; do  # file = "001-rotate-window-implementation.md"
    if [ -f "$file" ] && [ "$file" != "000-template.md" ]; then
        # Extract title from first line (includes ADR number)
        # title = "ADR-001: Window Rotation by Parent Split Flip"
        title=$(head -1 "$file" | sed 's/^# //')

        # Create entry line
        # entry = "- **[ADR-001: Window Rotation by Parent Split Flip](001-rotate-window-implementation.md)**"
        entry="- **[$title]($file)**"

        # Extract status and add to appropriate array
        # grep -A1 gets "## Status" line + 1 line after, tail -1 gets the last line
        # status_line = "**Current**: Accepted (2024-07-30)" or just "Accepted"
        status_line=$(grep -A1 "^## Status" "$file" | tail -1)
        if echo "$status_line" | grep -q "Accepted"; then
            accepted_adrs+=("$entry")
        elif echo "$status_line" | grep -q "Proposed"; then
            proposed_adrs+=("$entry")
        elif echo "$status_line" | grep -q "Deprecated"; then
            deprecated_adrs+=("$entry")
        else
            accepted_adrs+=("$entry")  # Default to accepted
        fi
    fi
done

# Output grouped ADRs
if [ ${#accepted_adrs[@]} -gt 0 ]; then  # Check if array has any elements
    echo "## Accepted"
    for entry in "${accepted_adrs[@]}"; do  # Loop through each markdown list item
        echo "$entry"
    done
    echo ""
fi

if [ ${#proposed_adrs[@]} -gt 0 ]; then
    echo "## Proposed"
    for entry in "${proposed_adrs[@]}"; do
        echo "$entry"
    done
    echo ""
fi

if [ ${#deprecated_adrs[@]} -gt 0 ]; then
    echo "## Deprecated"
    for entry in "${deprecated_adrs[@]}"; do
        echo "$entry"
    done
    echo ""
fi

echo "_Generated: $(date '+%Y-%m-%d %H:%M')_"