# PowerShell script to clear FirstRunScanKeyboards flag for development testing
# This allows you to reset the state after testing the import wizard

Write-Host "Clearing FirstRunScanKeyboards flag from registry..." -ForegroundColor Yellow

try {
    $settingsPath = "HKCU:\Software\KeyMagic\Settings"
    
    # Check if the path exists
    if (!(Test-Path $settingsPath)) {
        Write-Host "Registry path does not exist: $settingsPath" -ForegroundColor Yellow
        exit 0
    }
    
    # Check if the value exists
    $value = Get-ItemProperty -Path $settingsPath -Name "FirstRunScanKeyboards" -ErrorAction SilentlyContinue
    if ($value -and $null -ne $value.FirstRunScanKeyboards) {
        # Remove the value
        Remove-ItemProperty -Path $settingsPath -Name "FirstRunScanKeyboards" -Force
        Write-Host "Successfully removed FirstRunScanKeyboards flag" -ForegroundColor Green
    } else {
        Write-Host "FirstRunScanKeyboards flag was not set" -ForegroundColor Yellow
    }
    
    # Display current registry values for debugging
    Write-Host "`nCurrent KeyMagic Settings:" -ForegroundColor Yellow
    $props = Get-ItemProperty -Path $settingsPath -ErrorAction SilentlyContinue
    if ($props) {
        $props | Format-List
    } else {
        Write-Host "No settings found" -ForegroundColor Gray
    }
    
} catch {
    Write-Host "Error clearing registry value: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`nDone! The import wizard will not appear on next launch." -ForegroundColor Green