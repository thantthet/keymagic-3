# Enable Windows Error Reporting crash dumps
# Run as Administrator

$registryPath = "HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps"

if (!(Test-Path $registryPath)) {
    New-Item -Path $registryPath -Force
}

# Set dump folder
Set-ItemProperty -Path $registryPath -Name "DumpFolder" -Value "%LOCALAPPDATA%\CrashDumps" -Type String

# Set dump count (keep last 10 dumps)
Set-ItemProperty -Path $registryPath -Name "DumpCount" -Value 10 -Type DWord

# Set dump type (2 = Full dump)
Set-ItemProperty -Path $registryPath -Name "DumpType" -Value 2 -Type DWord

Write-Host "Crash dumps enabled. Dumps will be saved to %LOCALAPPDATA%\CrashDumps"