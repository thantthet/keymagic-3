#!/usr/bin/env pwsh
# Script to check version numbers across all KeyMagic Windows components

# Get the directory where the script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "KeyMagic Windows Version Check" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

# Function to extract version from file
function Get-VersionFromFile {
    param(
        [string]$FilePath,
        [string]$Pattern,
        [int]$GroupIndex = 1
    )
    
    if (Test-Path $FilePath) {
        $Content = Get-Content $FilePath -Raw
        if ($Content -match $Pattern) {
            return $matches[$GroupIndex]
        }
        return "Not found"
    } else {
        return "File missing"
    }
}

# Check version.txt (master version)
$VersionFile = Join-Path $ProjectRoot "version.txt"
if (Test-Path $VersionFile) {
    $MasterVersion = (Get-Content $VersionFile).Trim()
    Write-Host "`nMaster Version (version.txt): " -NoNewline
    Write-Host $MasterVersion -ForegroundColor Green
} else {
    Write-Host "`nMaster Version (version.txt): " -NoNewline
    Write-Host "Not found" -ForegroundColor Red
    $MasterVersion = "Unknown"
}

Write-Host "`nComponent Versions:" -ForegroundColor Yellow

# Check gui-tauri/src-tauri/Cargo.toml - only package version
$CargoTomlPath = Join-Path $ProjectRoot "gui-tauri\src-tauri\Cargo.toml"
$CargoVersion = "Not found"
if (Test-Path $CargoTomlPath) {
    $Content = Get-Content $CargoTomlPath -Raw
    # Look for version in [package] section
    if ($Content -match '\[package\][^\[]*version\s*=\s*"([^"]*)"') {
        $CargoVersion = $matches[1]
    }
}
Write-Host "  GUI Cargo.toml:        " -NoNewline
if ($CargoVersion -eq $MasterVersion) {
    Write-Host $CargoVersion -ForegroundColor Green
} else {
    Write-Host $CargoVersion -ForegroundColor Red
}

# Check gui-tauri/src-tauri/tauri.conf.json
$TauriConfPath = Join-Path $ProjectRoot "gui-tauri\src-tauri\tauri.conf.json"
$TauriVersion = Get-VersionFromFile -FilePath $TauriConfPath -Pattern '"version"\s*:\s*"([^"]*)"'
Write-Host "  Tauri Config:          " -NoNewline
if ($TauriVersion -eq $MasterVersion) {
    Write-Host $TauriVersion -ForegroundColor Green
} else {
    Write-Host $TauriVersion -ForegroundColor Red
}

# Check tsf/src/KeyMagicTSF.rc
$RcFilePath = Join-Path $ProjectRoot "tsf\src\KeyMagicTSF.rc"
$RcFileVersion = Get-VersionFromFile -FilePath $RcFilePath -Pattern 'VALUE\s+"FileVersion",\s*"([^"]*)"'
$RcProductVersion = Get-VersionFromFile -FilePath $RcFilePath -Pattern 'VALUE\s+"ProductVersion",\s*"([^"]*)"'
Write-Host "  TSF File Version:      " -NoNewline
# Convert master version to file version format for comparison
$MasterFileVersion = "$($MasterVersion.Split('-')[0]).0"
if ($RcFileVersion -eq $MasterFileVersion) {
    Write-Host $RcFileVersion -ForegroundColor Green
} else {
    Write-Host $RcFileVersion -ForegroundColor Red
}
Write-Host "  TSF Product Version:   " -NoNewline
if ($RcProductVersion -eq $MasterFileVersion) {
    Write-Host $RcProductVersion -ForegroundColor Green
} else {
    Write-Host $RcProductVersion -ForegroundColor Red
}

# Check installer/setup-x64.iss
$SetupX64Path = Join-Path $ProjectRoot "installer\setup-x64.iss"
$SetupX64Version = Get-VersionFromFile -FilePath $SetupX64Path -Pattern '#define\s+MyAppVersion\s+"([^"]*)"'
Write-Host "  Installer x64:         " -NoNewline
if ($SetupX64Version -eq $MasterVersion) {
    Write-Host $SetupX64Version -ForegroundColor Green
} else {
    Write-Host $SetupX64Version -ForegroundColor Red
}

# Check installer/setup-arm64.iss
$SetupArm64Path = Join-Path $ProjectRoot "installer\setup-arm64.iss"
$SetupArm64Version = Get-VersionFromFile -FilePath $SetupArm64Path -Pattern '#define\s+MyAppVersion\s+"([^"]*)"'
Write-Host "  Installer ARM64:       " -NoNewline
if ($SetupArm64Version -eq $MasterVersion) {
    Write-Host $SetupArm64Version -ForegroundColor Green
} else {
    Write-Host $SetupArm64Version -ForegroundColor Red
}

# Summary
Write-Host "`nVersion Consistency Check:" -ForegroundColor Yellow
$AllVersions = @($CargoVersion, $TauriVersion, $SetupX64Version, $SetupArm64Version)
$FileVersions = @($RcFileVersion, $RcProductVersion)
$ExpectedFileVersion = "$($MasterVersion.Split('-')[0]).0"

$AllMatch = $true
foreach ($Version in $AllVersions) {
    if ($Version -ne $MasterVersion) {
        $AllMatch = $false
        break
    }
}
foreach ($Version in $FileVersions) {
    if ($Version -ne $ExpectedFileVersion) {
        $AllMatch = $false
        break
    }
}

if ($AllMatch) {
    Write-Host "  All versions are synchronized!" -ForegroundColor Green
} else {
    Write-Host "  Version mismatch detected!" -ForegroundColor Red
    Write-Host "  Run 'update-version.bat' to synchronize all versions" -ForegroundColor Yellow
}