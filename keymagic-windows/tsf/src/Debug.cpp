#include "Debug.h"

// Debug file logging implementation
void DebugWriteToFile(const char* message) {
#ifdef _DEBUG
    static FILE* debugFile = nullptr;
    if (!debugFile) {
        char path[MAX_PATH];
        char tempPath[MAX_PATH];
        GetTempPathA(MAX_PATH, tempPath);
        
        time_t now = time(NULL);
        struct tm timeinfo;
        localtime_s(&timeinfo, &now);
        
        sprintf_s(path, sizeof(path), "%sKeyMagicTSF_%04d%02d%02d_%02d%02d%02d.log",
                  tempPath,
                  timeinfo.tm_year + 1900, timeinfo.tm_mon + 1, timeinfo.tm_mday,
                  timeinfo.tm_hour, timeinfo.tm_min, timeinfo.tm_sec);
        
        fopen_s(&debugFile, path, "a");
        if (debugFile) {
            fprintf(debugFile, "=== KeyMagic TSF Debug Log Started ===\n");
            fflush(debugFile);
        }
    }
    
    if (debugFile) {
        time_t now = time(NULL);
        struct tm timeinfo;
        localtime_s(&timeinfo, &now);
        
        fprintf(debugFile, "[%02d:%02d:%02d] %s",
                timeinfo.tm_hour, timeinfo.tm_min, timeinfo.tm_sec, message);
        fflush(debugFile);
    }
#else
    // No-op in release builds
    (void)message;
#endif
}