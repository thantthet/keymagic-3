# PowerShell script to set FirstRunScanKeyboards flag for development testing
# This simulates what the installer does, allowing you to test the import wizard

Write-Host "Setting FirstRunScanKeyboards flag in registry..." -ForegroundColor Yellow

try {
    # Create the KeyMagic registry keys if they don't exist
    $keymagicPath = "HKCU:\Software\KeyMagic"
    $settingsPath = "HKCU:\Software\KeyMagic\Settings"
    
    # Create KeyMagic key if it doesn't exist
    if (!(Test-Path $keymagicPath)) {
        New-Item -Path $keymagicPath -Force | Out-Null
        Write-Host "Created registry key: $keymagicPath" -ForegroundColor Green
    }
    
    # Create Settings key if it doesn't exist
    if (!(Test-Path $settingsPath)) {
        New-Item -Path $settingsPath -Force | Out-Null
        Write-Host "Created registry key: $settingsPath" -ForegroundColor Green
    }
    
    # Set the FirstRunScanKeyboards value to 1 (DWORD)
    Set-ItemProperty -Path $settingsPath -Name "FirstRunScanKeyboards" -Value 1 -Type DWord
    
    # Verify the value was set
    $value = Get-ItemProperty -Path $settingsPath -Name "FirstRunScanKeyboards" -ErrorAction SilentlyContinue
    if ($value.FirstRunScanKeyboards -eq 1) {
        Write-Host "Successfully set FirstRunScanKeyboards = 1" -ForegroundColor Green
        Write-Host "The import wizard will appear on next KeyMagic launch" -ForegroundColor Cyan
    } else {
        Write-Host "Failed to verify the registry value" -ForegroundColor Red
    }
    
    # Display current registry values for debugging
    Write-Host "`nCurrent KeyMagic Settings:" -ForegroundColor Yellow
    Get-ItemProperty -Path $settingsPath | Format-List
    
} catch {
    Write-Host "Error setting registry value: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`nDone! Launch KeyMagic to see the import wizard." -ForegroundColor Green