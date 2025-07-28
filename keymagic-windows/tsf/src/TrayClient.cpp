#include "TrayClient.h"
#include <psapi.h>
#include <shlwapi.h>
#include <strsafe.h>
#include <algorithm>
#include <sddl.h>
#include <memory>

// Helper function to get user SID
static std::wstring GetUserSID() {
    HANDLE hToken;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken)) {
        return L"";
    }

    DWORD dwBufferSize = 0;
    GetTokenInformation(hToken, TokenUser, nullptr, 0, &dwBufferSize);
    
    auto buffer = std::make_unique<BYTE[]>(dwBufferSize);
    if (!GetTokenInformation(hToken, TokenUser, buffer.get(), dwBufferSize, &dwBufferSize)) {
        CloseHandle(hToken);
        return L"";
    }

    TOKEN_USER* pTokenUser = reinterpret_cast<TOKEN_USER*>(buffer.get());
    
    LPWSTR stringSid = nullptr;
    if (!ConvertSidToStringSidW(pTokenUser->User.Sid, &stringSid)) {
        CloseHandle(hToken);
        return L"";
    }

    std::wstring result(stringSid);
    LocalFree(stringSid);
    CloseHandle(hToken);
    
    return result;
}

// Helper function to get pipe name
static std::wstring GetPipeName() {
    return L"\\\\.\\pipe\\KeyMagicTray-" + GetUserSID();
}

TrayClient::TrayClient()
    : m_hPipe(INVALID_HANDLE_VALUE)
    , m_connected(false)
    , m_reconnectDelay(1000) {
    m_pipeName = GetPipeName();
}

TrayClient::~TrayClient() {
    Disconnect();
}

bool TrayClient::Connect() {
    if (m_connected) {
        return true;
    }
    
    return TryConnect();
}

void TrayClient::Disconnect() {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    if (m_hPipe != INVALID_HANDLE_VALUE) {
        CloseHandle(m_hPipe);
        m_hPipe = INVALID_HANDLE_VALUE;
    }
    
    m_connected = false;
}

bool TrayClient::SendMessage(const TrayMessage& msg) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    if (!m_connected && !TryConnect()) {
        return false;
    }
    
    DWORD bytesWritten;
    BOOL result = WriteFile(m_hPipe, &msg, sizeof(msg), &bytesWritten, nullptr);
    
    if (!result || bytesWritten != sizeof(msg)) {
        // Connection lost, mark as disconnected
        m_connected = false;
        CloseHandle(m_hPipe);
        m_hPipe = INVALID_HANDLE_VALUE;
        
        // Try to reconnect immediately
        if (TryConnect()) {
            // Retry sending
            result = WriteFile(m_hPipe, &msg, sizeof(msg), &bytesWritten, nullptr);
            return result && bytesWritten == sizeof(msg);
        }
        
        return false;
    }
    
    return true;
}

void TrayClient::NotifyFocusGained(const std::wstring& keyboardId) {
    TrayMessage msg = {};
    msg.messageType = MSG_FOCUS_GAINED;
    msg.processId = GetCurrentProcessId();
    StringCchCopyW(msg.keyboardId, ARRAYSIZE(msg.keyboardId), keyboardId.c_str());
    
    std::wstring processName = GetProcessName();
    StringCchCopyW(msg.processName, ARRAYSIZE(msg.processName), processName.c_str());
    
    SendMessage(msg);
}

void TrayClient::NotifyFocusLost() {
    TrayMessage msg = {};
    msg.messageType = MSG_FOCUS_LOST;
    msg.processId = GetCurrentProcessId();
    
    SendMessage(msg);
}

void TrayClient::NotifyKeyboardChanged(const std::wstring& keyboardId) {
    TrayMessage msg = {};
    msg.messageType = MSG_KEYBOARD_CHANGED;
    msg.processId = GetCurrentProcessId();
    StringCchCopyW(msg.keyboardId, ARRAYSIZE(msg.keyboardId), keyboardId.c_str());
    
    SendMessage(msg);
}

void TrayClient::NotifyTipStarted() {
    TrayMessage msg = {};
    msg.messageType = MSG_TIP_STARTED;
    msg.processId = GetCurrentProcessId();
    
    std::wstring processName = GetProcessName();
    StringCchCopyW(msg.processName, ARRAYSIZE(msg.processName), processName.c_str());
    
    SendMessage(msg);
}

void TrayClient::NotifyTipStopped() {
    TrayMessage msg = {};
    msg.messageType = MSG_TIP_STOPPED;
    msg.processId = GetCurrentProcessId();
    
    SendMessage(msg);
}

bool TrayClient::TryConnect() {
    // Try to open existing pipe
    m_hPipe = CreateFileW(
        m_pipeName.c_str(),
        GENERIC_WRITE,
        0,
        nullptr,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        nullptr
    );
    
    if (m_hPipe != INVALID_HANDLE_VALUE) {
        // Set pipe to message mode
        DWORD mode = PIPE_READMODE_MESSAGE;
        SetNamedPipeHandleState(m_hPipe, &mode, nullptr, nullptr);
        
        m_connected = true;
        m_reconnectDelay = 1000; // Reset delay on successful connection
        return true;
    }
    
    // Increase reconnect delay exponentially up to 30 seconds
    if (m_reconnectDelay < 30000) {
        m_reconnectDelay = (m_reconnectDelay * 2 < 30000) ? m_reconnectDelay * 2 : 30000;
    }
    
    return false;
}

std::wstring TrayClient::GetProcessName() {
    WCHAR processPath[MAX_PATH];
    if (GetModuleFileNameW(nullptr, processPath, MAX_PATH) > 0) {
        WCHAR* fileName = PathFindFileNameW(processPath);
        return std::wstring(fileName);
    }
    
    return L"Unknown";
}