# KeyMagic Icon Downloader
# Downloads icon resources from GitHub

$ErrorActionPreference = "Stop"

# Configuration
$baseUrl = "https://raw.githubusercontent.com/thantthet/keymagic/master/windows/KeyMagic2/KeyMagic2/app_icon"
$iconDir = Join-Path $PSScriptRoot "..\..\resources\icons"

# Icon files to download
$icons = @(
    @{ Name = "keymagic.ico"; Url = "$baseUrl.ico" },
    @{ Name = "keymagic-16.png"; Url = "$baseUrl-16.png" },
    @{ Name = "keymagic-32.png"; Url = "$baseUrl-32.png" },
    @{ Name = "keymagic-48.png"; Url = "$baseUrl-48.png" },
    @{ Name = "keymagic-80.png"; Url = "$baseUrl-80.png" },
    @{ Name = "keymagic-512.png"; Url = "$baseUrl-512.png" }
)

# Create icons directory if it doesn't exist
if (!(Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir | Out-Null
    Write-Host "Created icons directory: $iconDir"
}

# Download each icon
$downloaded = 0
$failed = 0

foreach ($icon in $icons) {
    $outputPath = Join-Path $iconDir $icon.Name
    
    try {
        Write-Host "Downloading $($icon.Name)..." -NoNewline
        Invoke-WebRequest -Uri $icon.Url -OutFile $outputPath -UseBasicParsing
        Write-Host " OK" -ForegroundColor Green
        $downloaded++
    }
    catch {
        Write-Host " FAILED" -ForegroundColor Red
        Write-Host "  Error: $_" -ForegroundColor Yellow
        $failed++
    }
}

# Summary
Write-Host ""
Write-Host "Download complete: $downloaded succeeded, $failed failed" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Yellow" })

# Exit with error if any downloads failed
if ($failed -gt 0) {
    exit 1
}