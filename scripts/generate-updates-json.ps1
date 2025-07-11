# Generate updates.json from installer files in the output directory
# This script reads installer files and creates the updates.json file with
# proper version information, file sizes, and SHA256 hashes.

param(
    [string]$InstallerDir = "$PSScriptRoot\..\keymagic-windows\installer\output",
    [string]$OutputFile = "$PSScriptRoot\..\updates.json"
)

$ErrorActionPreference = "Stop"

# Configuration
$GithubRepo = "thantthet/keymagic-3"
$FilenamePattern = "KeyMagic3-Setup-(\d+\.\d+\.\d+(?:-\w+)?)-(\w+)\.exe"

function Get-FileHash256 {
    param([string]$FilePath)
    
    $hash = Get-FileHash -Path $FilePath -Algorithm SHA256
    return $hash.Hash.ToLower()
}

function Get-NormalizedArch {
    param([string]$Arch)
    
    $archMap = @{
        "x64" = "x86_64"
        "arm64" = "aarch64"
        "x86" = "x86"
    }
    
    $normalized = $archMap[$Arch.ToLower()]
    if ($normalized) {
        return $normalized
    }
    return $Arch
}

function Parse-InstallerFilename {
    param([string]$Filename)
    
    if ($Filename -match $FilenamePattern) {
        return @{
            Version = $Matches[1]
            Arch = $Matches[2]
        }
    }
    return $null
}

function Get-LatestVersion {
    param([string[]]$Versions)
    
    $sorted = $Versions | Sort-Object { 
        $parts = $_ -split '-'
        $versionParts = $parts[0] -split '\.'
        [version]::new($versionParts[0], $versionParts[1], $versionParts[2])
    }
    
    return $sorted[-1]
}

# Main script
Write-Host "Generating updates.json from installer files..." -ForegroundColor Green

# Check if installer directory exists
if (!(Test-Path $InstallerDir)) {
    Write-Error "Installer directory not found: $InstallerDir"
    exit 1
}

# Find all installer files
$installers = Get-ChildItem -Path $InstallerDir -Filter "KeyMagic3-Setup-*.exe"

if ($installers.Count -eq 0) {
    Write-Error "No installer files found in $InstallerDir"
    exit 1
}

# Group installers by version
$versions = @{}

foreach ($installer in $installers) {
    Write-Host "Processing: $($installer.Name)"
    
    $parsed = Parse-InstallerFilename -Filename $installer.Name
    if (!$parsed) {
        Write-Warning "Could not parse filename: $($installer.Name)"
        continue
    }
    
    $version = $parsed.Version
    $arch = Get-NormalizedArch -Arch $parsed.Arch
    
    if (!$versions.ContainsKey($version)) {
        $versions[$version] = @{}
    }
    
    # Calculate file info
    $fileSize = $installer.Length
    $sha256 = Get-FileHash256 -FilePath $installer.FullName
    
    $versions[$version][$arch] = @{
        version = $version
        releaseDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        minimumSystemVersion = "10.0.17763"  # Windows 10 1809
        url = "https://github.com/$GithubRepo/releases/download/v$version/$($installer.Name)"
        signature = ""
        size = $fileSize
        sha256 = $sha256
    }
}

if ($versions.Count -eq 0) {
    Write-Error "No valid installer files found"
    exit 1
}

# Get the latest version
$latestVersion = Get-LatestVersion -Versions $versions.Keys

# Create the full JSON structure
$updatesData = @{
    name = "KeyMagic"
    platforms = @{
        windows = $versions[$latestVersion]
        macos = @{
            x86_64 = @{
                version = $latestVersion
                releaseDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
                minimumSystemVersion = "10.15"
                url = "https://github.com/$GithubRepo/releases/download/v$latestVersion/KeyMagic3-$latestVersion-x64.dmg"
                signature = ""
                size = 0
                sha256 = ""
            }
            aarch64 = @{
                version = $latestVersion
                releaseDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
                minimumSystemVersion = "11.0"
                url = "https://github.com/$GithubRepo/releases/download/v$latestVersion/KeyMagic3-$latestVersion-arm64.dmg"
                signature = ""
                size = 0
                sha256 = ""
            }
        }
        linux = @{
            x86_64 = @{
                version = $latestVersion
                releaseDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
                minimumSystemVersion = ""
                url = "https://github.com/$GithubRepo/releases/download/v$latestVersion/keymagic3-$latestVersion-amd64.deb"
                signature = ""
                size = 0
                sha256 = ""
            }
        }
    }
    releaseNotes = @{
        $latestVersion = @{
            en = "### KeyMagic $latestVersion`n`n- Please add release notes here`n"
        }
    }
}

# Convert to JSON and save
$json = $updatesData | ConvertTo-Json -Depth 10
$json | Out-File -FilePath $OutputFile -Encoding UTF8

Write-Host "`nSuccessfully generated $OutputFile" -ForegroundColor Green
Write-Host "Latest version: $latestVersion" -ForegroundColor Cyan
Write-Host "`nFile details:" -ForegroundColor Yellow

foreach ($arch in $versions[$latestVersion].Keys) {
    $info = $versions[$latestVersion][$arch]
    $sizeKB = [math]::Round($info.size / 1KB, 2)
    $shortHash = $info.sha256.Substring(0, 16)
    Write-Host "  ${arch}: $sizeKB KB, SHA256: $shortHash..."
}

Write-Host "`nNext steps:" -ForegroundColor Green
Write-Host "1. Review the generated updates.json file"
Write-Host "2. Add appropriate release notes"
Write-Host "3. Deploy to GitHub Pages using ./scripts/deploy-updates.sh"