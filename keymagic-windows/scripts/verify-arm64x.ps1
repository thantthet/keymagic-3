# verify-arm64x.ps1 - Verify if a DLL is a true ARM64X binary
param(
    [Parameter(Mandatory=$false)]
    [string]$DllPath = "..\target\release\KeyMagicTSF.dll"
)

# Resolve full path
$FullPath = Resolve-Path $DllPath -ErrorAction SilentlyContinue
if (-not $FullPath) {
    Write-Host "ERROR: File not found: $DllPath" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "ARM64X Binary Verification" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "File: $FullPath"
Write-Host ""

# Find dumpbin.exe
$VsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $VsWhere) {
    $VsPath = & $VsWhere -latest -property installationPath
    $DumpBin = "$VsPath\VC\Tools\MSVC\*\bin\Hostarm64\arm64\dumpbin.exe"
    $DumpBin = Get-ChildItem $DumpBin | Select-Object -First 1
} else {
    Write-Host "ERROR: Visual Studio not found" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $DumpBin)) {
    Write-Host "ERROR: dumpbin.exe not found" -ForegroundColor Red
    exit 1
}

# Get file size
$FileInfo = Get-Item $FullPath
$SizeMB = [math]::Round($FileInfo.Length / 1MB, 2)
Write-Host "File Size: $SizeMB MB"

# Check machine type
Write-Host ""
Write-Host "Machine Type:" -ForegroundColor Yellow
$MachineOutput = & $DumpBin /headers $FullPath | Out-String
$MachineType = $MachineOutput | Select-String "machine" | Select-Object -First 1

$IsArm64x = $false
if ($MachineType -match "ARM64X") {
    Write-Host "  OK: ARM64X detected" -ForegroundColor Green
    Write-Host "  $MachineType"
    $IsArm64x = $true
} elseif ($MachineType -match "AA64.*ARM64") {
    Write-Host "  OK: ARM64 detected (checking for ARM64X)" -ForegroundColor Yellow
    Write-Host "  $MachineType"
} else {
    Write-Host "  ERROR: Not ARM64/ARM64X" -ForegroundColor Red
    Write-Host "  $MachineType"
}

# Check code ranges
Write-Host ""
Write-Host "Code Ranges:" -ForegroundColor Yellow
$LoadConfig = & $DumpBin /loadconfig $FullPath | Out-String

$HasHybridTable = $false
if ($LoadConfig -match "Hybrid Code Address Range Table") {
    Write-Host "  OK: Hybrid Code Address Range Table found" -ForegroundColor Green
    $HasHybridTable = $true
    
    # Extract code ranges
    $Lines = $LoadConfig -split "`r?`n"
    $HasArm64 = $false
    $HasArm64ec = $false
    $HasX64 = $false
    
    foreach ($Line in $Lines) {
        if ($Line -match "^\s+arm64ec\s+") {
            Write-Host "  OK: ARM64EC code found: $Line" -ForegroundColor Green
            $HasArm64ec = $true
        } elseif ($Line -match "^\s+x64\s+") {
            Write-Host "  OK: x64 code found: $Line" -ForegroundColor Green
            $HasX64 = $true
        } elseif ($Line -match "^\s+arm64\s+") {
            Write-Host "  OK: ARM64 code found: $Line" -ForegroundColor Green
            $HasArm64 = $true
        }
    }
    
    # Check for ARM64X metadata
    Write-Host ""
    Write-Host "  ARM64X Metadata:" -ForegroundColor Cyan
    $Arm64xCount = 0
    foreach ($Line in $Lines) {
        if ($Line -match "Arm64X") {
            Write-Host "    $Line"
            $Arm64xCount++
            if ($Arm64xCount -ge 5) { break }
        }
    }
} else {
    Write-Host "  ERROR: No hybrid code ranges found" -ForegroundColor Red
    Write-Host "  This is NOT an ARM64X binary" -ForegroundColor Red
}

# Check exports
Write-Host ""
Write-Host "Exported Functions:" -ForegroundColor Yellow
$ExportsOutput = & $DumpBin /exports $FullPath | Out-String
$DllExports = $ExportsOutput | Select-String "Dll\w+" -AllMatches
foreach ($Export in $DllExports.Matches) {
    Write-Host "  $Export"
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
if ($HasHybridTable) {
    Write-Host "RESULT: This IS a valid ARM64X binary!" -ForegroundColor Green
    Write-Host "It contains code for:" -ForegroundColor Green
    if ($HasArm64) { Write-Host "  - ARM64 native" -ForegroundColor Green }
    if ($HasArm64ec) { Write-Host "  - ARM64EC (x64 compatible)" -ForegroundColor Green }
    if ($HasX64) { Write-Host "  - x64 thunks" -ForegroundColor Green }
} else {
    Write-Host "RESULT: This is NOT an ARM64X binary" -ForegroundColor Red
}
Write-Host "========================================" -ForegroundColor Cyan