#!/usr/bin/env bash
# Cross-platform version update script wrapper

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is required but not found"
    exit 1
fi

# Run the Python script with all arguments
python3 "$SCRIPT_DIR/scripts/update-version.py" "$@"