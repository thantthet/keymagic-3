#include "IconCacheManager.h"
#include "keymagic_ffi.h"
#include <windows.h>
#include <objbase.h>
#include <gdiplus.h>
#include <shlwapi.h>
#include <sstream>
#include <fstream>

#pragma comment(lib, "gdiplus.lib")

using namespace Gdiplus;

IconCacheManager::IconCacheManager()
    : m_gdiplusToken(0)
    , m_gdiplusInitialized(false) {
}

IconCacheManager::~IconCacheManager() {
    // Clean up cached icons
    for (auto& pair : m_iconCache) {
        if (pair.second) {
            DestroyIcon(pair.second);
        }
    }
    
    // Shutdown GDI+
    if (m_gdiplusInitialized) {
        GdiplusShutdown(m_gdiplusToken);
    }
}

bool IconCacheManager::Initialize() {
    // Create cache directory
    m_cacheDir = GetIconCachePath();
    if (!CreateDirectoryW(m_cacheDir.c_str(), nullptr)) {
        if (GetLastError() != ERROR_ALREADY_EXISTS) {
            return false;
        }
    }
    
    // Initialize GDI+
    return EnsureGdiPlusInitialized();
}

HICON IconCacheManager::GetIcon(const std::wstring& keyboardId, const std::wstring& km2Path, int size) {
    std::lock_guard<std::mutex> lock(m_cacheMutex);
    
    // Check in-memory cache first
    std::wstring cacheKey = keyboardId + L"_" + std::to_wstring(size);
    auto it = m_iconCache.find(cacheKey);
    if (it != m_iconCache.end()) {
        return it->second;
    }
    
    // Try to load from disk cache
    HICON hIcon = LoadFromCache(keyboardId, size);
    if (hIcon) {
        m_iconCache[cacheKey] = hIcon;
        return hIcon;
    }
    
    // Extract from KM2 file
    std::vector<BYTE> iconData;
    if (ExtractIcon(km2Path, iconData) && !iconData.empty()) {
        // Convert to requested size and create HICON
        hIcon = ImageDataToIcon(iconData, size);
        if (hIcon) {
            // Save to cache for future use
            SaveToCache(keyboardId, size, iconData);
            m_iconCache[cacheKey] = hIcon;
            return hIcon;
        }
    }
    
    return nullptr;
}

void IconCacheManager::ClearCache(const std::wstring& keyboardId) {
    std::lock_guard<std::mutex> lock(m_cacheMutex);
    
    // Remove from memory cache
    auto it = m_iconCache.begin();
    while (it != m_iconCache.end()) {
        if (it->first.find(keyboardId) == 0) {
            if (it->second) {
                DestroyIcon(it->second);
            }
            it = m_iconCache.erase(it);
        } else {
            ++it;
        }
    }
    
    // Remove disk cache files
    std::vector<int> sizes = {16, 24, 32, 48};
    for (int size : sizes) {
        std::wstring cachePath = GetCachePath(keyboardId, size);
        DeleteFileW(cachePath.c_str());
    }
}

void IconCacheManager::ClearAllCache() {
    std::lock_guard<std::mutex> lock(m_cacheMutex);
    
    // Clear memory cache
    for (auto& pair : m_iconCache) {
        if (pair.second) {
            DestroyIcon(pair.second);
        }
    }
    m_iconCache.clear();
    
    // Clear disk cache
    WIN32_FIND_DATAW findData;
    std::wstring searchPath = m_cacheDir + L"\\*.png";
    HANDLE hFind = FindFirstFileW(searchPath.c_str(), &findData);
    
    if (hFind != INVALID_HANDLE_VALUE) {
        do {
            std::wstring filePath = m_cacheDir + L"\\" + findData.cFileName;
            DeleteFileW(filePath.c_str());
        } while (FindNextFileW(hFind, &findData));
        FindClose(hFind);
    }
}

bool IconCacheManager::ExtractIcon(const std::wstring& km2Path, std::vector<BYTE>& iconData) {
    // Convert wide string to UTF-8
    int len = WideCharToMultiByte(CP_UTF8, 0, km2Path.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::vector<char> utf8Path(len);
    WideCharToMultiByte(CP_UTF8, 0, km2Path.c_str(), -1, utf8Path.data(), len, nullptr, nullptr);
    
    // Load KM2 file
    Km2FileHandle* km2Handle = keymagic_km2_load(utf8Path.data());
    if (!km2Handle) {
        return false;
    }
    
    // Get icon data size first
    size_t iconSize = keymagic_km2_get_icon_data(km2Handle, nullptr, 0);
    if (iconSize == 0) {
        keymagic_km2_free(km2Handle);
        return false;
    }
    
    // Allocate buffer and get icon data
    iconData.resize(iconSize);
    size_t actualSize = keymagic_km2_get_icon_data(km2Handle, iconData.data(), iconSize);
    
    keymagic_km2_free(km2Handle);
    
    if (actualSize != iconSize) {
        iconData.clear();
        return false;
    }
    
    return true;
}

HICON IconCacheManager::ImageDataToIcon(const std::vector<BYTE>& imageData, int size) {
    if (!EnsureGdiPlusInitialized()) {
        return nullptr;
    }
    
    // Create stream from data
    IStream* pStream = nullptr;
    HGLOBAL hGlobal = GlobalAlloc(GMEM_MOVEABLE, imageData.size());
    if (!hGlobal) {
        return nullptr;
    }
    
    void* pData = GlobalLock(hGlobal);
    if (!pData) {
        GlobalFree(hGlobal);
        return nullptr;
    }
    
    memcpy(pData, imageData.data(), imageData.size());
    GlobalUnlock(hGlobal);
    
    if (CreateStreamOnHGlobal(hGlobal, TRUE, &pStream) != S_OK) {
        GlobalFree(hGlobal);
        return nullptr;
    }
    
    // Load image from stream
    Bitmap* pBitmap = Bitmap::FromStream(pStream);
    pStream->Release();
    
    if (!pBitmap || pBitmap->GetLastStatus() != Ok) {
        delete pBitmap;
        return nullptr;
    }
    
    // Scale to requested size if needed
    HICON hIcon = nullptr;
    if (pBitmap->GetWidth() != size || pBitmap->GetHeight() != size) {
        Bitmap* pResized = new Bitmap(size, size, PixelFormat32bppARGB);
        Graphics graphics(pResized);
        graphics.SetInterpolationMode(InterpolationModeHighQualityBicubic);
        graphics.DrawImage(pBitmap, 0, 0, size, size);
        
        pResized->GetHICON(&hIcon);
        delete pResized;
    } else {
        pBitmap->GetHICON(&hIcon);
    }
    
    delete pBitmap;
    return hIcon;
}

bool IconCacheManager::SaveToCache(const std::wstring& keyboardId, int size, const std::vector<BYTE>& iconData) {
    std::wstring cachePath = GetCachePath(keyboardId, size);
    
    std::ofstream file(cachePath, std::ios::binary);
    if (!file) {
        return false;
    }
    
    file.write(reinterpret_cast<const char*>(iconData.data()), iconData.size());
    return file.good();
}

HICON IconCacheManager::LoadFromCache(const std::wstring& keyboardId, int size) {
    std::wstring cachePath = GetCachePath(keyboardId, size);
    
    // Check if file exists
    if (!PathFileExistsW(cachePath.c_str())) {
        return nullptr;
    }
    
    // Read file data
    std::ifstream file(cachePath, std::ios::binary | std::ios::ate);
    if (!file) {
        return nullptr;
    }
    
    std::streamsize fileSize = file.tellg();
    file.seekg(0, std::ios::beg);
    
    std::vector<BYTE> iconData(fileSize);
    if (!file.read(reinterpret_cast<char*>(iconData.data()), fileSize)) {
        return nullptr;
    }
    
    return ImageDataToIcon(iconData, size);
}

std::wstring IconCacheManager::GetCachePath(const std::wstring& keyboardId, int size) {
    std::wstringstream ss;
    ss << m_cacheDir << L"\\" << keyboardId << L"_" << size << L".png";
    return ss.str();
}

bool IconCacheManager::EnsureGdiPlusInitialized() {
    if (m_gdiplusInitialized) {
        return true;
    }
    
    GdiplusStartupInput gdiplusStartupInput;
    Status status = GdiplusStartup(&m_gdiplusToken, &gdiplusStartupInput, nullptr);
    
    if (status == Ok) {
        m_gdiplusInitialized = true;
        return true;
    }
    
    return false;
}