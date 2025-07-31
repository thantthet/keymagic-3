#pragma once

#include "Common.h"
#include "../../shared/include/TrayProtocol.h"
#include <functional>

class NamedPipeServer {
public:
    using MessageCallback = std::function<void(const TrayMessage&)>;
    
    NamedPipeServer();
    ~NamedPipeServer();
    
    // Start the pipe server
    bool Start(const std::wstring& pipeName, MessageCallback callback);
    
    // Stop the pipe server
    void Stop();
    
    // Check if server is running
    bool IsRunning() const { return m_running; }

private:
    // Worker thread for handling pipe connections
    void ServerThread();
    
    // Handle a single client connection
    void HandleClient(HANDLE hPipe);
    
    // Create pipe instance with security attributes
    HANDLE CreatePipeInstance();

private:
    std::wstring m_pipeName;
    MessageCallback m_callback;
    std::atomic<bool> m_running;
    std::thread m_serverThread;
    HANDLE m_stopEvent;
    SECURITY_ATTRIBUTES* m_pSecurityAttributes;
};