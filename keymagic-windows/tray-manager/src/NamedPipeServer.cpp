#include "NamedPipeServer.h"
#include "SecurityUtils.h"
#include <shlwapi.h>

NamedPipeServer::NamedPipeServer()
    : m_running(false)
    , m_stopEvent(nullptr)
    , m_pSecurityAttributes(nullptr) {
}

NamedPipeServer::~NamedPipeServer() {
    Stop();
}

bool NamedPipeServer::Start(const std::wstring& pipeName, MessageCallback callback) {
    if (m_running) {
        return false;
    }
    
    m_pipeName = pipeName;
    m_callback = callback;
    
    // Create stop event
    m_stopEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);
    if (!m_stopEvent) {
        return false;
    }
    
    // Create security attributes
    m_pSecurityAttributes = SecurityUtils::CreateLowIntegritySecurityAttributes();
    if (!m_pSecurityAttributes) {
        CloseHandle(m_stopEvent);
        m_stopEvent = nullptr;
        return false;
    }
    
    m_running = true;
    m_serverThread = std::thread(&NamedPipeServer::ServerThread, this);
    
    return true;
}

void NamedPipeServer::Stop() {
    if (!m_running) {
        return;
    }
    
    m_running = false;
    
    if (m_stopEvent) {
        SetEvent(m_stopEvent);
    }
    
    if (m_serverThread.joinable()) {
        m_serverThread.join();
    }
    
    if (m_stopEvent) {
        CloseHandle(m_stopEvent);
        m_stopEvent = nullptr;
    }
    
    if (m_pSecurityAttributes) {
        SecurityUtils::FreeSecurityAttributes(m_pSecurityAttributes);
        m_pSecurityAttributes = nullptr;
    }
}

void NamedPipeServer::ServerThread() {
    while (m_running) {
        HANDLE hPipe = CreatePipeInstance();
        if (hPipe == INVALID_HANDLE_VALUE) {
            Sleep(1000);
            continue;
        }
        
        // Wait for client connection
        OVERLAPPED overlapped = {};
        overlapped.hEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);
        
        BOOL connected = ConnectNamedPipe(hPipe, &overlapped);
        if (!connected && GetLastError() == ERROR_IO_PENDING) {
            HANDLE waitHandles[] = { m_stopEvent, overlapped.hEvent };
            DWORD waitResult = WaitForMultipleObjects(2, waitHandles, FALSE, INFINITE);
            
            if (waitResult == WAIT_OBJECT_0) {
                // Stop event signaled
                CancelIo(hPipe);
                CloseHandle(overlapped.hEvent);
                CloseHandle(hPipe);
                break;
            } else if (waitResult == WAIT_OBJECT_0 + 1) {
                // Client connected
                DWORD bytesTransferred;
                if (GetOverlappedResult(hPipe, &overlapped, &bytesTransferred, FALSE)) {
                    connected = TRUE;
                }
            }
        }
        
        CloseHandle(overlapped.hEvent);
        
        if (connected || GetLastError() == ERROR_PIPE_CONNECTED) {
            // Handle client in separate thread to allow multiple connections
            std::thread clientThread(&NamedPipeServer::HandleClient, this, hPipe);
            clientThread.detach();
        } else {
            CloseHandle(hPipe);
        }
    }
}

void NamedPipeServer::HandleClient(HANDLE hPipe) {
    TrayMessage message;
    DWORD bytesRead;
    
    while (ReadFile(hPipe, &message, sizeof(message), &bytesRead, nullptr)) {
        if (bytesRead == sizeof(message) && m_callback) {
            m_callback(message);
        }
    }
    
    DisconnectNamedPipe(hPipe);
    CloseHandle(hPipe);
}

HANDLE NamedPipeServer::CreatePipeInstance() {
    return CreateNamedPipeW(
        m_pipeName.c_str(),
        PIPE_ACCESS_INBOUND | FILE_FLAG_OVERLAPPED,
        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        PIPE_BUFFER_SIZE,
        PIPE_BUFFER_SIZE,
        0,
        m_pSecurityAttributes
    );
}