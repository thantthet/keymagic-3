# PowerShell script to check FirstRunScanKeyboards flag status
# Useful for debugging the import wizard behavior

Write-Host "Checking FirstRunScanKeyboards flag status..." -ForegroundColor Yellow
Write-Host ""

try {
    $settingsPath = "HKCU:\Software\KeyMagic\Settings"
    
    # Check if the path exists
    if (!(Test-Path $settingsPath)) {
        Write-Host "Registry path does not exist: $settingsPath" -ForegroundColor Red
        Write-Host "KeyMagic may not be installed or hasn't been run yet" -ForegroundColor Yellow
        exit 0
    }
    
    # Try to get the value
    $value = Get-ItemProperty -Path $settingsPath -Name "FirstRunScanKeyboards" -ErrorAction SilentlyContinue
    
    if ($value -and $null -ne $value.FirstRunScanKeyboards) {
        $flagValue = $value.FirstRunScanKeyboards
        Write-Host "FirstRunScanKeyboards = $flagValue" -ForegroundColor Cyan
        
        if ($flagValue -eq 1) {
            Write-Host "Status: Import wizard WILL appear on next launch" -ForegroundColor Green
        } else {
            Write-Host "Status: Import wizard will NOT appear (value is not 1)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "FirstRunScanKeyboards flag is NOT SET" -ForegroundColor Yellow
        Write-Host "Status: Import wizard will NOT appear on next launch" -ForegroundColor Gray
    }
    
    Write-Host ""
    Write-Host "All KeyMagic Settings:" -ForegroundColor Yellow
    Write-Host "-" * 50
    
    $allProps = Get-ItemProperty -Path $settingsPath -ErrorAction SilentlyContinue
    if ($allProps) {
        # Remove PowerShell default properties
        $allProps.PSObject.Properties | Where-Object { $_.Name -notlike "PS*" } | ForEach-Object {
            Write-Host "$($_.Name): $($_.Value)" -ForegroundColor White
        }
    } else {
        Write-Host "No settings found" -ForegroundColor Gray
    }
    
} catch {
    Write-Host "Error reading registry: $_" -ForegroundColor Red
    exit 1
}