#pragma once

#include <windows.h>
#include <string>
#include <vector>

// Convert language code (e.g., "en-US", "my-MM") to Windows LANGID
LANGID LanguageCodeToLangId(const std::wstring& languageCode);

// Get language name from language code
std::wstring GetLanguageName(const std::wstring& languageCode);

// Get all supported language codes
std::vector<std::wstring> GetSupportedLanguageCodes();