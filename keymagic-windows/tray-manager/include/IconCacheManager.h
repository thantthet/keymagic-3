#pragma once

#include "Common.h"

// Forward declare to avoid including gdiplus.h in header
namespace Gdiplus {
    class GdiplusStartupInput;
}

class IconCacheManager {
public:
    IconCacheManager();
    ~IconCacheManager();
    
    // Initialize the cache directory
    bool Initialize();
    
    // Get icon for keyboard (loads from cache or extracts)
    HICON GetIcon(const std::wstring& keyboardId, const std::wstring& km2Path, int size);
    
    // Clear cache for a specific keyboard
    void ClearCache(const std::wstring& keyboardId);
    
    // Clear all cached icons
    void ClearAllCache();

private:
    // Extract icon from KM2 file using FFI
    bool ExtractIcon(const std::wstring& km2Path, std::vector<BYTE>& iconData);
    
    // Convert image data (PNG, BMP, etc.) to HICON using GDI+
    HICON ImageDataToIcon(const std::vector<BYTE>& imageData, int size);
    
    // Save icon data to cache
    bool SaveToCache(const std::wstring& keyboardId, int size, const std::vector<BYTE>& iconData);
    
    // Load icon from cache
    HICON LoadFromCache(const std::wstring& keyboardId, int size);
    
    // Get cache file path
    std::wstring GetCachePath(const std::wstring& keyboardId, int size);
    
    // Ensure GDI+ is initialized
    bool EnsureGdiPlusInitialized();

private:
    std::wstring m_cacheDir;
    std::map<std::wstring, HICON> m_iconCache;
    std::mutex m_cacheMutex;
    
    // GDI+ token
    ULONG_PTR m_gdiplusToken;
    bool m_gdiplusInitialized;
};