@echo off
:: KeyMagic Windows Build System - Shared Functions
:: ===============================================

:: This file is loaded by make.bat and provides reusable functions
:: Functions are called using 'call :function_name'

:: Note: Functions are defined in make.bat to avoid CALL stack issues
:: This file can be extended with additional utility functions as needed

:: Common error codes
set "ERROR_NOT_ADMIN=5"
set "ERROR_FILE_NOT_FOUND=2"
set "ERROR_BUILD_FAILED=1"

:: Color codes for output (if supported)
:: These can be used with PowerShell for colored output if needed