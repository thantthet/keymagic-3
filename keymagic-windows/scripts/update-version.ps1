#!/usr/bin/env pwsh
# Script to update version numbers across all KeyMagic Windows components

param(
    [Parameter(Mandatory=$false)]
    [string]$NewVersion
)

# Get the directory where the script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Read version from version.txt if no parameter provided
if (-not $NewVersion) {
    $VersionFile = Join-Path $ProjectRoot "version.txt"
    if (Test-Path $VersionFile) {
        $NewVersion = (Get-Content $VersionFile).Trim()
    } else {
        Write-Error "No version specified and version.txt not found"
        exit 1
    }
}

# Validate version format (basic semver validation)
if (-not ($NewVersion -match '^\d+\.\d+\.\d+(-.*)?$')) {
    Write-Error "Invalid version format. Expected format: X.Y.Z or X.Y.Z-suffix"
    exit 1
}

Write-Host "Updating version to: $NewVersion" -ForegroundColor Green

# Convert version to file version format (X.Y.Z to X,Y,Z,0)
$VersionParts = $NewVersion.Split('-')[0].Split('.')
$FileVersion = "$($VersionParts[0]),$($VersionParts[1]),$($VersionParts[2]),0"
$DotFileVersion = "$($VersionParts[0]).$($VersionParts[1]).$($VersionParts[2]).0"

# Function to update file content
function Update-FileContent {
    param(
        [string]$FilePath,
        [scriptblock]$UpdateScript
    )
    
    if (Test-Path $FilePath) {
        $Content = Get-Content $FilePath -Raw
        $NewContent = & $UpdateScript $Content
        if ($Content -ne $NewContent) {
            Set-Content -Path $FilePath -Value $NewContent -NoNewline
            Write-Host "Updated: $FilePath" -ForegroundColor Yellow
        } else {
            Write-Host "No changes needed: $FilePath" -ForegroundColor Gray
        }
    } else {
        Write-Warning "File not found: $FilePath"
    }
}

# Update keymagic-shared/gui/src-tauri/Cargo.toml - only the package version
$CargoTomlPath = Join-Path $ProjectRoot "..\keymagic-shared\gui\src-tauri\Cargo.toml"
Update-FileContent -FilePath $CargoTomlPath -UpdateScript {
    param($Content)
    # Split content into lines
    $lines = $Content -split "`r?`n"
    $inPackageSection = $false
    $packageVersionUpdated = $false
    
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        
        # Check if we're entering the [package] section
        if ($line -match '^\[package\]') {
            $inPackageSection = $true
        }
        # Check if we're leaving the package section (entering another section)
        elseif ($line -match '^\[') {
            $inPackageSection = $false
        }
        # Update version only if we're in the package section and haven't updated yet
        elseif ($inPackageSection -and -not $packageVersionUpdated -and $line -match '^version\s*=\s*"[^"]*"') {
            $lines[$i] = "version = `"$NewVersion`""
            $packageVersionUpdated = $true
        }
    }
    
    # Join lines back together
    return $lines -join "`n"
}

# Update keymagic-shared/gui/src-tauri/tauri.conf.json
$TauriConfPath = Join-Path $ProjectRoot "..\keymagic-shared\gui\src-tauri\tauri.conf.json"
Update-FileContent -FilePath $TauriConfPath -UpdateScript {
    param($Content)
    $Content -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$NewVersion`""
}

# Update tsf/src/KeyMagicTSF.rc
$RcFilePath = Join-Path $ProjectRoot "tsf\src\KeyMagicTSF.rc"
Update-FileContent -FilePath $RcFilePath -UpdateScript {
    param($Content)
    # Update FILEVERSION
    $Content = $Content -replace 'FILEVERSION\s+\d+,\d+,\d+,\d+', "FILEVERSION     $FileVersion"
    # Update PRODUCTVERSION
    $Content = $Content -replace 'PRODUCTVERSION\s+\d+,\d+,\d+,\d+', "PRODUCTVERSION  $FileVersion"
    # Update FileVersion string
    $Content = $Content -replace 'VALUE\s+"FileVersion",\s*"[^"]*"', "VALUE `"FileVersion`",      `"$DotFileVersion`""
    # Update ProductVersion string
    $Content = $Content -replace 'VALUE\s+"ProductVersion",\s*"[^"]*"', "VALUE `"ProductVersion`",   `"$DotFileVersion`""
    return $Content
}

# Update installer/setup-x64.iss
$SetupX64Path = Join-Path $ProjectRoot "installer\setup-x64.iss"
Update-FileContent -FilePath $SetupX64Path -UpdateScript {
    param($Content)
    $Content -replace '#define\s+MyAppVersion\s+"[^"]*"', "#define MyAppVersion `"$NewVersion`""
}

# Update installer/setup-arm64.iss
$SetupArm64Path = Join-Path $ProjectRoot "installer\setup-arm64.iss"
Update-FileContent -FilePath $SetupArm64Path -UpdateScript {
    param($Content)
    $Content -replace '#define\s+MyAppVersion\s+"[^"]*"', "#define MyAppVersion `"$NewVersion`""
}

# Update version.txt
$VersionFile = Join-Path $ProjectRoot "version.txt"
Set-Content -Path $VersionFile -Value $NewVersion -NoNewline

Write-Host "`nVersion update complete!" -ForegroundColor Green
Write-Host "All components have been updated to version: $NewVersion" -ForegroundColor Cyan