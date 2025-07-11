# register-tsf.ps1
# PowerShell script to register TSF DLL with proper architecture detection
# Must be run as Administrator

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("x64", "ARM64", "Auto")]
    [string]$Architecture = "Auto",
    
    [Parameter(Mandatory=$false)]
    [string]$DllPath = ""
)

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator"))
{
    Write-Error "This script must be run as Administrator. Exiting..."
    exit 1
}

# Detect system architecture if Auto
if ($Architecture -eq "Auto") {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "ARM64") {
        $Architecture = "ARM64"
    } else {
        $Architecture = "x64"
    }
    Write-Host "Detected architecture: $Architecture" -ForegroundColor Cyan
}

# Set DLL path if not provided
if ([string]::IsNullOrEmpty($DllPath)) {
    $scriptDir = Split-Path -Parent $PSScriptRoot
    $DllPath = Join-Path $scriptDir "..\tsf\build-$Architecture\Release\KeyMagicTSF.dll"
}

# Verify DLL exists
if (-NOT (Test-Path $DllPath)) {
    Write-Error "TSF DLL not found at: $DllPath"
    Write-Host "Please build the project first using: make.bat build $Architecture Release"
    exit 1
}

Write-Host "Registering TSF DLL: $DllPath" -ForegroundColor Green

# Unregister any existing registration
Write-Host "Unregistering any existing TSF..." -ForegroundColor Yellow
& regsvr32.exe /s /u $DllPath 2>$null

# Clean up temporary registrations
$tempDirs = Get-ChildItem "$env:TEMP" -Directory -Filter "KeyMagicTSF_*" -ErrorAction SilentlyContinue
foreach ($dir in $tempDirs) {
    Write-Host "Cleaning up: $($dir.FullName)" -ForegroundColor Yellow
    Remove-Item $dir.FullName -Recurse -Force -ErrorAction SilentlyContinue
}

# Register the DLL
Write-Host "Registering TSF..." -ForegroundColor Green
$result = & regsvr32.exe /s $DllPath 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "SUCCESS: TSF registered successfully!" -ForegroundColor Green
    
    # Verify registration
    $clsid = "{094A562B-D08B-4CAF-8E95-8F8031CFD24C}"
    $regPath = "Registry::HKEY_CLASSES_ROOT\CLSID\$clsid"
    
    if (Test-Path $regPath) {
        Write-Host "Verified: Registry entries created" -ForegroundColor Green
        
        # Display registration details
        $inprocServer = Get-ItemProperty "$regPath\InprocServer32" -ErrorAction SilentlyContinue
        if ($inprocServer) {
            Write-Host "DLL Path: $($inprocServer.'(default)')" -ForegroundColor Cyan
        }
    } else {
        Write-Warning "Registry verification failed - entries not found"
    }
} else {
    Write-Error "Failed to register TSF DLL"
    Write-Host "Error details: $result"
    exit 1
}

Write-Host "`nTSF Registration completed!" -ForegroundColor Green
Write-Host "You may need to restart applications or log off/on for changes to take effect." -ForegroundColor Yellow