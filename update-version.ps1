#!/usr/bin/env pwsh
# Cross-platform version update script wrapper for PowerShell

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Check if Python 3 is available
$python = $null
if (Get-Command python3 -ErrorAction SilentlyContinue) {
    $python = "python3"
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
    # Check if it's Python 3
    $version = & python --version 2>&1
    if ($version -match "Python 3") {
        $python = "python"
    }
}

if (-not $python) {
    Write-Error "Python 3 is required but not found"
    exit 1
}

# Run the Python script with all arguments
& $python (Join-Path $ScriptDir "scripts" "update-version.py") $args