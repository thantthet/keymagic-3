# KeyMagic Diagnostics Tool
# Comprehensive system check and log analysis

param(
    [switch]$Detailed = $false
)

Write-Host "KeyMagic Diagnostics Report" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan
Write-Host ""

# Function to check file/registry with colored output
function Test-Item {
    param($Path, $Description, $Type = "File")
    
    $exists = if ($Type -eq "Registry") { 
        (Get-Item -Path $Path -ErrorAction SilentlyContinue) -ne $null 
    } else { 
        Test-Path $Path 
    }
    
    if ($exists) {
        Write-Host "[OK] " -ForegroundColor Green -NoNewline
    } else {
        Write-Host "[--] " -ForegroundColor Yellow -NoNewline
    }
    Write-Host $Description
    
    return $exists
}

# Check processes
Write-Host "Processes:" -ForegroundColor White
$ctfmon = Get-Process -Name ctfmon -ErrorAction SilentlyContinue
if ($ctfmon) {
    Write-Host "[OK] " -ForegroundColor Green -NoNewline
    Write-Host "Text Services Framework (ctfmon.exe) is running"
} else {
    Write-Host "[--] " -ForegroundColor Yellow -NoNewline
    Write-Host "ctfmon.exe not running"
}

$gui = Get-Process -Name keymagic-config -ErrorAction SilentlyContinue
if ($gui) {
    Write-Host "[OK] " -ForegroundColor Green -NoNewline
    Write-Host "KeyMagic GUI is running (PID: $($gui.Id))"
} else {
    Write-Host "[--] " -ForegroundColor Yellow -NoNewline
    Write-Host "KeyMagic GUI not running"
}

Write-Host ""

# Check files
Write-Host "Files:" -ForegroundColor White
Test-Item "tsf\build\Release\KeyMagicTSF.dll" "TSF DLL built"
Test-Item "target\release\keymagic-config.exe" "GUI executable built"
Test-Item "keyboards\ZawCode.km2" "Test keyboard installed"
Test-Item "resources\icons\keymagic.ico" "Icon resources present"

Write-Host ""

# Check registry
Write-Host "Registry:" -ForegroundColor White
Test-Item "Registry::HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" "COM registration" "Registry"
Test-Item "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\CTF\TIP\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}" "TSF registration" "Registry"
Test-Item "Registry::HKEY_CURRENT_USER\Software\KeyMagic" "KeyMagic settings" "Registry"

Write-Host ""

# Check logs if detailed
if ($Detailed) {
    Write-Host "Log Files:" -ForegroundColor White
    
    # GUI log
    $guiLog = "target\release\keymagic-gui.log"
    if (Test-Path $guiLog) {
        Write-Host "[OK] " -ForegroundColor Green -NoNewline
        Write-Host "GUI log found ($(((Get-Item $guiLog).Length / 1KB).ToString('N2')) KB)"
        Write-Host "Last 10 lines:" -ForegroundColor Gray
        Get-Content $guiLog -Tail 10 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
    } else {
        Write-Host "[--] " -ForegroundColor Yellow -NoNewline
        Write-Host "No GUI log file"
    }
    
    Write-Host ""
    
    # Windows Event Log
    Write-Host "Recent Application Errors:" -ForegroundColor White
    $events = Get-EventLog -LogName Application -EntryType Error -Newest 5 -ErrorAction SilentlyContinue | 
              Where-Object { $_.Source -like "*keymagic*" -or $_.Message -like "*keymagic*" }
    
    if ($events) {
        foreach ($event in $events) {
            Write-Host "  [$($event.TimeGenerated)] $($event.Source): $($event.Message.Split("`n")[0])" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "  No KeyMagic-related errors in Event Log" -ForegroundColor Gray
    }
}

Write-Host ""

# System info
Write-Host "System Information:" -ForegroundColor White
Write-Host "  OS: $([System.Environment]::OSVersion.VersionString)"
Write-Host "  Architecture: $env:PROCESSOR_ARCHITECTURE"
Write-Host "  User: $env:USERNAME"
Write-Host "  Admin: $(if ((New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { 'Yes' } else { 'No' })"

Write-Host ""

# Recommendations
$issues = @()
if (!$ctfmon) { $issues += "Start ctfmon.exe for Text Services Framework" }
if (!(Test-Path "tsf\build\Release\KeyMagicTSF.dll")) { $issues += "Build TSF: make.bat build tsf" }
if (!(Test-Path "target\release\keymagic-config.exe")) { $issues += "Build GUI: make.bat build gui" }
if (!(Test-Path "Registry::HKEY_CLASSES_ROOT\CLSID\{094A562B-D08B-4CAF-8E95-8F8031CFD24C}")) { $issues += "Register TSF: make.bat register (as admin)" }

if ($issues.Count -gt 0) {
    Write-Host "Recommendations:" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "  - $issue" -ForegroundColor Yellow
    }
} else {
    Write-Host "Status: All systems operational" -ForegroundColor Green
}