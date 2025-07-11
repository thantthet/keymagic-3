# check-dll-arch.ps1
# PowerShell script to check the architecture of DLL files

param(
    [Parameter(Mandatory=$true)]
    [string]$DllPath
)

function Get-DllArchitecture {
    param([string]$Path)
    
    if (-not (Test-Path $Path)) {
        Write-Error "File not found: $Path"
        return $null
    }
    
    try {
        $bytes = [System.IO.File]::ReadAllBytes($Path)
        
        # Check if it's a valid PE file (MZ header)
        if ($bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
            Write-Error "Not a valid PE file"
            return $null
        }
        
        # Get PE header offset from 0x3C
        $peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
        
        # Check PE signature
        $peSignature = [BitConverter]::ToUInt32($bytes, $peOffset)
        if ($peSignature -ne 0x00004550) { # "PE\0\0"
            Write-Error "Invalid PE signature"
            return $null
        }
        
        # Machine type is 2 bytes after PE signature + 4
        $machineType = [BitConverter]::ToUInt16($bytes, $peOffset + 4)
        
        switch ($machineType) {
            0x014C { return "x86 (32-bit)" }
            0x0200 { return "IA64" }
            0x8664 { return "x64 (AMD64)" }
            0xAA64 { return "ARM64" }
            0x01C0 { return "ARM" }
            0x01C4 { return "ARMv7 (Thumb-2)" }
            default { return "Unknown (0x{0:X4})" -f $machineType }
        }
    }
    catch {
        Write-Error "Error reading file: $_"
        return $null
    }
}

# Main script
Write-Host "Checking DLL Architecture" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host ""

$architecture = Get-DllArchitecture -Path $DllPath

if ($architecture) {
    Write-Host "File: " -NoNewline
    Write-Host $DllPath -ForegroundColor Yellow
    Write-Host "Architecture: " -NoNewline
    Write-Host $architecture -ForegroundColor Green
    
    # Additional file info
    $fileInfo = Get-Item $DllPath
    Write-Host "File Size: " -NoNewline
    Write-Host ("{0:N0} bytes" -f $fileInfo.Length) -ForegroundColor Gray
    Write-Host "Modified: " -NoNewline
    Write-Host $fileInfo.LastWriteTime -ForegroundColor Gray
}

# Check multiple DLLs if wildcards are used
if ($DllPath.Contains("*")) {
    Write-Host ""
    Write-Host "Checking multiple files..." -ForegroundColor Cyan
    $files = Get-ChildItem $DllPath -ErrorAction SilentlyContinue
    
    if ($files.Count -gt 0) {
        $results = @()
        foreach ($file in $files) {
            $arch = Get-DllArchitecture -Path $file.FullName
            if ($arch) {
                $results += [PSCustomObject]@{
                    FileName = $file.Name
                    Architecture = $arch
                    Size = "{0:N0}" -f $file.Length
                }
            }
        }
        
        if ($results.Count -gt 0) {
            Write-Host ""
            $results | Format-Table -AutoSize
        }
    }
}