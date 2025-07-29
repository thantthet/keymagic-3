#pragma once

#include <windows.h>
#include <string>
#include <vector>

// Manager class for handling tray icon visibility promotion across different Windows versions
class IconVisibilityManager {
public:
    IconVisibilityManager() = default;
    ~IconVisibilityManager() = default;
    
    // Main entry point to ensure icon is visible in the notification area
    void EnsureIconVisible();
    
    // Force icon visibility using the appropriate method for the OS version
    void PromoteIcon(const std::wstring& exePath);
    
private:
    // Result of promotion attempt
    enum class PromotionResult {
        NotFound,       // Icon entry not found
        AlreadyPromoted, // Icon is already promoted
        Promoted,       // Successfully promoted
        Failed          // Found but failed to promote
    };
    
    // Windows 11: Use NotifyIconSettings registry
    PromotionResult PromoteIconWindows11(const std::wstring& exePath);
    
    // Windows 10: Use IconStreams binary data
    PromotionResult PromoteIconWindows10(const std::wstring& exePath);
    
    // Parse and update IconStreams visibility
    PromotionResult UpdateIconStreamsVisibility(const std::wstring& exePath);
    
    // Helper function for ROT-13 encoding
    std::wstring Rot13Encode(const std::wstring& input);
    
    // Resolve KNOWNFOLDERID paths to actual paths
    std::wstring ResolveKnownFolderPath(const std::wstring& path);
    
    // Send refresh notification to Explorer
    void RefreshNotificationArea();
    
    // Check if running on Windows 11
    bool IsWindows11();
};