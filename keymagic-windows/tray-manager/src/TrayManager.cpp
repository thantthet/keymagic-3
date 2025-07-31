#include "TrayManager.h"
#include <shlobj.h>
#include <shellapi.h>

TrayManager* TrayManager::s_instance = nullptr;

TrayManager::TrayManager()
    : m_hWnd(nullptr)
    , m_hasFocus(false)
    , m_contextMenuActive(false)
    , m_hideTimerId(0) {
    s_instance = this;
}

TrayManager::~TrayManager() {
    Cleanup();
    s_instance = nullptr;
}

int TrayManager::Run() {
    // Check for single instance
    HANDLE hMutex = CreateMutexW(nullptr, TRUE, KEYMAGIC_MUTEX_NAME);
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        MessageBoxW(nullptr, L"KeyMagic Tray Manager is already running.", L"KeyMagic", MB_OK | MB_ICONINFORMATION);
        if (hMutex) CloseHandle(hMutex);
        return 1;
    }
    
    // Initialize COM
    HRESULT hr = CoInitialize(nullptr);
    if (FAILED(hr)) {
        if (hMutex) CloseHandle(hMutex);
        return 1;
    }
    
    // Initialize components
    if (!Initialize()) {
        CoUninitialize();
        if (hMutex) CloseHandle(hMutex);
        return 1;
    }
    
    // Message loop
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    
    // Cleanup
    Cleanup();
    CoUninitialize();
    if (hMutex) CloseHandle(hMutex);
    
    return static_cast<int>(msg.wParam);
}

LRESULT CALLBACK TrayManager::WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    if (s_instance) {
        return s_instance->HandleMessage(hWnd, message, wParam, lParam);
    }
    return DefWindowProc(hWnd, message, wParam, lParam);
}

LRESULT TrayManager::HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message) {
        case WM_CREATE:
            return 0;
            
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
            
        case WM_TRAYICON:
            if (m_trayIcon) {
                // Detect right-click early (only on button down to avoid duplicates)
                if (LOWORD(lParam) == WM_RBUTTONDOWN) {
                    OutputDebugStringW(L"TrayManager: Right-click detected on tray icon\n");
                    // Cancel any pending hide timer immediately
                    if (m_hideTimerId) {
                        KillTimer(hWnd, m_hideTimerId);
                        m_hideTimerId = 0;
                    }
                    m_contextMenuActive = true;
                }
                m_trayIcon->HandleMessage(hWnd, message, wParam, lParam);
            }
            return 0;
            
        case WM_COMMAND:
            if (LOWORD(wParam) == 0 && HIWORD(wParam) == 0) {
                // Show context menu
                auto keyboards = m_registryMonitor->GetKeyboards();
                m_trayIcon->ShowContextMenu(hWnd, keyboards, m_currentKeyboardId,
                    [this](UINT cmdId) { 
                        OnMenuCommand(cmdId); 
                    });
            }
            return 0;
            
        case WM_PIPE_MESSAGE:
            // Pipe message received (posted from pipe thread)
            {
                TrayMessage* pMsg = reinterpret_cast<TrayMessage*>(lParam);
                if (pMsg) {
                    OnPipeMessage(*pMsg);
                    delete pMsg;
                }
            }
            return 0;
            
        case WM_MENU_SHOWN:
            OutputDebugStringW(L"TrayManager: Context menu shown\n");
            // Cancel hide timer if it's running
            if (m_hideTimerId) {
                KillTimer(hWnd, m_hideTimerId);
                m_hideTimerId = 0;
            }
            m_contextMenuActive = true;
            return 0;
            
        case WM_MENU_DISMISSED:
            OutputDebugStringW(L"TrayManager: Context menu dismissed\n");
            m_contextMenuActive = false;
            // Update icon visibility after menu closes
            UpdateTrayIcon();
            return 0;
            
        case WM_TIMER:
            if (wParam == TIMER_HIDE_DELAY) {
                OutputDebugStringW(L"TrayManager: Hide timer triggered\n");
                KillTimer(hWnd, TIMER_HIDE_DELAY);
                m_hideTimerId = 0;
                
                // Only hide if context menu is not active AND mouse is not over tray area
                if (!m_contextMenuActive && !IsMouseOverTrayArea()) {
                    OutputDebugStringW(L"TrayManager: Hiding icon (delayed)\n");
                    UpdateTrayIcon();
                } else if (IsMouseOverTrayArea()) {
                    OutputDebugStringW(L"TrayManager: Not hiding - mouse is still over tray area\n");
                }
            }
            return 0;
    }
    
    return DefWindowProc(hWnd, message, wParam, lParam);
}

bool TrayManager::Initialize() {
    // Create message window
    if (!CreateMessageWindow()) {
        return false;
    }
    
    // Initialize icon cache
    m_iconCache = std::make_unique<IconCacheManager>();
    if (!m_iconCache->Initialize()) {
        return false;
    }
    
    // Initialize profile monitor
    m_profileMonitor = std::make_unique<ProfileMonitor>();
    if (!m_profileMonitor->Initialize()) {
        // Non-fatal, continue without profile monitoring
    }
    
    // Initialize registry monitor
    m_registryMonitor = std::make_unique<RegistryMonitor>();
    if (!m_registryMonitor->Start([this]() { OnRegistryChange(); })) {
        return false;
    }
    
    // Initialize tray icon
    m_trayIcon = std::make_unique<TrayIcon>();
    if (!m_trayIcon->Initialize(m_hWnd)) {
        return false;
    }
    
    // Start named pipe server
    m_pipeServer = std::make_unique<NamedPipeServer>();
    std::wstring pipeName = GetPipeName();
    if (!m_pipeServer->Start(pipeName, [this](const TrayMessage& msg) { 
        // Post message to main thread
        TrayMessage* pMsg = new TrayMessage(msg);
        PostMessage(m_hWnd, WM_PIPE_MESSAGE, 0, reinterpret_cast<LPARAM>(pMsg));
    })) {
        return false;
    }
    
    // Check initial state
    if (m_profileMonitor->IsKeyMagicActive()) {
        m_currentKeyboardId = m_registryMonitor->GetDefaultKeyboard();
        UpdateTrayIcon();
    }
    
    return true;
}

void TrayManager::Cleanup() {
    // Kill any pending timer
    if (m_hideTimerId && m_hWnd) {
        KillTimer(m_hWnd, m_hideTimerId);
        m_hideTimerId = 0;
    }
    
    // Stop components in reverse order
    if (m_pipeServer) {
        m_pipeServer->Stop();
        m_pipeServer.reset();
    }
    
    if (m_trayIcon) {
        m_trayIcon->Hide();
        m_trayIcon.reset();
    }
    
    if (m_registryMonitor) {
        m_registryMonitor->Stop();
        m_registryMonitor.reset();
    }
    
    m_profileMonitor.reset();
    m_iconCache.reset();
    
    if (m_hWnd) {
        DestroyWindow(m_hWnd);
        m_hWnd = nullptr;
    }
}

bool TrayManager::CreateMessageWindow() {
    // Register window class
    WNDCLASSEXW wcex = {};
    wcex.cbSize = sizeof(WNDCLASSEXW);
    wcex.lpfnWndProc = WindowProc;
    wcex.hInstance = GetModuleHandle(nullptr);
    wcex.lpszClassName = KEYMAGIC_TRAY_CLASS;
    
    if (!RegisterClassExW(&wcex)) {
        if (GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
            return false;
        }
    }
    
    // Create message-only window
    m_hWnd = CreateWindowExW(
        0,
        KEYMAGIC_TRAY_CLASS,
        L"KeyMagic Tray Manager",
        0,
        0, 0, 0, 0,
        HWND_MESSAGE,
        nullptr,
        GetModuleHandle(nullptr),
        nullptr
    );
    
    return m_hWnd != nullptr;
}

void TrayManager::OnPipeMessage(const TrayMessage& msg) {
    std::lock_guard<std::mutex> lock(m_stateMutex);
    
    // Debug output
    OutputDebugStringW(L"TrayManager: Received pipe message\n");
    OutputDebugStringW((L"  Message type: " + std::to_wstring(msg.messageType) + L"\n").c_str());
    OutputDebugStringW((L"  Process ID: " + std::to_wstring(msg.processId) + L"\n").c_str());
    OutputDebugStringW((L"  Keyboard ID: " + std::wstring(msg.keyboardId) + L"\n").c_str());
    
    switch (msg.messageType) {
        case MSG_TIP_STARTED:
            OutputDebugStringW(L"  -> MSG_TIP_STARTED\n");
            m_activeTipProcesses.insert(msg.processId);
            OutputDebugStringW((L"  Active TIP count: " + std::to_wstring(m_activeTipProcesses.size()) + L"\n").c_str());
            break;
            
        case MSG_TIP_STOPPED:
            OutputDebugStringW(L"  -> MSG_TIP_STOPPED\n");
            m_activeTipProcesses.erase(msg.processId);
            OutputDebugStringW((L"  Active TIP count: " + std::to_wstring(m_activeTipProcesses.size()) + L"\n").c_str());
            if (m_activeTipProcesses.empty()) {
                m_hasFocus = false;
                UpdateTrayIcon();
            }
            break;
            
        case MSG_FOCUS_GAINED:
            OutputDebugStringW(L"  -> MSG_FOCUS_GAINED\n");
            
            // Cancel any pending hide timer
            if (m_hideTimerId) {
                KillTimer(m_hWnd, m_hideTimerId);
                m_hideTimerId = 0;
                OutputDebugStringW(L"  Cancelled hide timer\n");
            }
            
            m_hasFocus = true;
            if (msg.keyboardId[0]) {
                OutputDebugStringW((L"  Setting keyboard: " + std::wstring(msg.keyboardId) + L"\n").c_str());
                m_currentKeyboardId = msg.keyboardId;
            }
            UpdateTrayIcon();
            break;
            
        case MSG_FOCUS_LOST:
            OutputDebugStringW(L"  -> MSG_FOCUS_LOST\n");
            m_hasFocus = false;
            
            // Only start hide timer if mouse is NOT over tray area
            if (!IsMouseOverTrayArea()) {
                OutputDebugStringW(L"  Starting hide timer (mouse not over tray area)\n");
                // Set timer to hide icon after delay
                if (m_hWnd && !m_contextMenuActive) {
                    m_hideTimerId = SetTimer(m_hWnd, TIMER_HIDE_DELAY, HIDE_DELAY_MS, nullptr);
                }
            } else {
                OutputDebugStringW(L"  Not hiding - mouse is over tray area\n");
            }
            break;
            
        case MSG_KEYBOARD_CHANGE:
            OutputDebugStringW(L"  -> MSG_KEYBOARD_CHANGE\n");
            if (msg.keyboardId[0]) {
                OutputDebugStringW((L"  Changing keyboard to: " + std::wstring(msg.keyboardId) + L"\n").c_str());
                m_currentKeyboardId = msg.keyboardId;
                UpdateTrayIcon();
                
                // Update registry with new default keyboard
                HKEY hKey;
                if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Settings", 0, KEY_WRITE, &hKey) == ERROR_SUCCESS) {
                    RegSetValueExW(hKey, L"DefaultKeyboard", 0, REG_SZ, 
                                  (const BYTE*)msg.keyboardId, 
                                  (DWORD)((wcslen(msg.keyboardId) + 1) * sizeof(WCHAR)));
                    RegCloseKey(hKey);
                    
                    // Signal the global registry update event
                    HANDLE hEvent = OpenEventW(EVENT_MODIFY_STATE, FALSE, L"Global\\KeyMagicRegistryUpdate");
                    if (hEvent) {
                        SetEvent(hEvent);
                        CloseHandle(hEvent);
                        OutputDebugStringW(L"  -> Signaled registry update event\n");
                    } else {
                        // Try to create it if it doesn't exist
                        hEvent = CreateEventW(nullptr, TRUE, FALSE, L"Global\\KeyMagicRegistryUpdate");
                        if (hEvent) {
                            SetEvent(hEvent);
                            CloseHandle(hEvent);
                            OutputDebugStringW(L"  -> Created and signaled registry update event\n");
                        }
                    }
                }
            }
            break;
            
        default:
            OutputDebugStringW((L"  -> Unknown message type: " + std::to_wstring(msg.messageType) + L"\n").c_str());
            break;
    }
}

void TrayManager::OnRegistryChange() {
    // Registry changed, update current keyboard if needed
    std::lock_guard<std::mutex> lock(m_stateMutex);
    
    // Check if current keyboard still exists
    KeyboardInfo info;
    if (!m_currentKeyboardId.empty() && 
        !m_registryMonitor->GetKeyboardInfo(m_currentKeyboardId, info)) {
        // Current keyboard removed, switch to default
        m_currentKeyboardId = m_registryMonitor->GetDefaultKeyboard();
        UpdateTrayIcon();
    }
}

void TrayManager::OnMenuCommand(UINT cmdId) {
    const UINT IDM_KEYBOARD_BASE = 1000;
    const UINT IDM_ABOUT = 998;
    const UINT IDM_SETTINGS = 997;
    
    if (cmdId == IDM_SETTINGS) {
        // Launch KeyMagic GUI application
        LaunchKeyMagicApp();
    } else if (cmdId >= IDM_KEYBOARD_BASE) {
        // Keyboard selection
        UINT index = cmdId - IDM_KEYBOARD_BASE;
        auto keyboards = m_registryMonitor->GetKeyboards();
        
        if (index < keyboards.size()) {
            m_registryMonitor->SetDefaultKeyboard(keyboards[index].id);
            m_currentKeyboardId = keyboards[index].id;
            UpdateTrayIcon();
        }
    }
}

bool TrayManager::IsMouseOverTrayArea() const {
    // Get current mouse position
    POINT pt;
    if (!GetCursorPos(&pt)) {
        return false;
    }
    
    // Find the main taskbar window
    HWND hTaskbar = FindWindowW(L"Shell_TrayWnd", nullptr);
    if (!hTaskbar) {
        return false;
    }
    
    // Find the tray notification area within the taskbar
    HWND hTrayNotify = FindWindowExW(hTaskbar, nullptr, L"TrayNotifyWnd", nullptr);
    if (hTrayNotify) {
        RECT trayRect;
        if (GetWindowRect(hTrayNotify, &trayRect) && PtInRect(&trayRect, pt)) {
            return true;
        }
    }
    
    // Check system tray overflow window (when tray icons are hidden)
    HWND hOverflow = FindWindowW(L"NotifyIconOverflowWindow", nullptr);
    if (hOverflow && IsWindowVisible(hOverflow)) {
        RECT overflowRect;
        if (GetWindowRect(hOverflow, &overflowRect) && PtInRect(&overflowRect, pt)) {
            return true;
        }
    }
    
    // For secondary monitors, check their tray areas too
    HWND hSecondaryTaskbar = FindWindowW(L"Shell_SecondaryTrayWnd", nullptr);
    while (hSecondaryTaskbar) {
        // Find the tray area in secondary taskbar
        HWND hSecondaryTray = FindWindowExW(hSecondaryTaskbar, nullptr, L"ClockButton", nullptr);
        if (hSecondaryTray) {
            RECT secondaryTrayRect;
            if (GetWindowRect(hSecondaryTray, &secondaryTrayRect) && PtInRect(&secondaryTrayRect, pt)) {
                return true;
            }
        }
        hSecondaryTaskbar = FindWindowExW(nullptr, hSecondaryTaskbar, L"Shell_SecondaryTrayWnd", nullptr);
    }
    
    return false;
}

void TrayManager::UpdateTrayIcon() {
    if (!m_trayIcon) {
        return;
    }
    
    if (m_hasFocus && !m_currentKeyboardId.empty()) {
        // Get keyboard info
        KeyboardInfo info;
        if (m_registryMonitor->GetKeyboardInfo(m_currentKeyboardId, info)) {
            // Update icon
            HICON hIcon = m_iconCache->GetIcon(m_currentKeyboardId, info.path, DEFAULT_ICON_SIZE);
            if (hIcon) {
                m_trayIcon->SetIcon(hIcon);
            }
            
            // Update tooltip
            std::wstring tooltip = L"KeyMagic - " + info.name;
            m_trayIcon->SetTooltip(tooltip);
            
            // Update keyboard info for preview
            m_trayIcon->SetKeyboardInfo(m_currentKeyboardId, info.path);
        }
        
        m_trayIcon->Show();
    } else {
        m_trayIcon->Hide();
    }
}

bool TrayManager::IsAnyTipActive() const {
    return !m_activeTipProcesses.empty();
}

void TrayManager::LaunchKeyMagicApp() {
    // Get the directory where this executable is located
    wchar_t exePath[MAX_PATH];
    if (GetModuleFileNameW(nullptr, exePath, MAX_PATH) == 0) {
        OutputDebugStringW(L"TrayManager: Failed to get module file name\n");
        MessageBoxW(m_hWnd, L"Failed to determine application directory.", L"KeyMagic", MB_OK | MB_ICONERROR);
        return;
    }
    
    // Remove the filename to get the directory
    wchar_t* lastSlash = wcsrchr(exePath, L'\\');
    if (lastSlash) {
        *lastSlash = L'\0';
    }
    
    // Build path to keymagic.exe (GUI application)
    std::wstring keymagicPath = std::wstring(exePath) + L"\\keymagic.exe";
    
    // Check if keymagic.exe exists
    DWORD fileAttrib = GetFileAttributesW(keymagicPath.c_str());
    if (fileAttrib == INVALID_FILE_ATTRIBUTES || (fileAttrib & FILE_ATTRIBUTE_DIRECTORY)) {
        OutputDebugStringW((L"TrayManager: keymagic.exe not found at " + keymagicPath + L"\n").c_str());
        
        std::wstring errorMsg = L"KeyMagic Settings application not found.\n\n";
        errorMsg += L"Expected location:\n" + keymagicPath + L"\n\n";
        errorMsg += L"Please reinstall KeyMagic to fix this issue.";
        MessageBoxW(m_hWnd, errorMsg.c_str(), L"KeyMagic", MB_OK | MB_ICONERROR);
        return;
    }
    
    OutputDebugStringW((L"TrayManager: Launching " + keymagicPath + L"\n").c_str());
    
    // Launch the application
    SHELLEXECUTEINFOW sei = {};
    sei.cbSize = sizeof(SHELLEXECUTEINFOW);
    sei.fMask = SEE_MASK_NOCLOSEPROCESS;
    sei.lpFile = keymagicPath.c_str();
    sei.lpParameters = nullptr;
    sei.lpDirectory = exePath;
    sei.nShow = SW_SHOWNORMAL;
    
    if (!ShellExecuteExW(&sei)) {
        DWORD error = GetLastError();
        OutputDebugStringW((L"TrayManager: Failed to launch KeyMagic app, error: " + std::to_wstring(error) + L"\n").c_str());
        
        // Show error message to user
        std::wstring errorMsg = L"Failed to launch KeyMagic Settings.\n\n";
        errorMsg += L"Error code: " + std::to_wstring(error);
        
        // Add common error explanations
        if (error == ERROR_ACCESS_DENIED) {
            errorMsg += L"\n\nAccess denied. Please check file permissions.";
        } else if (error == ERROR_FILE_NOT_FOUND) {
            errorMsg += L"\n\nFile not found. Please reinstall KeyMagic.";
        }
        
        MessageBoxW(m_hWnd, errorMsg.c_str(), L"KeyMagic", MB_OK | MB_ICONERROR);
    } else {
        // Close the handle as we don't need to wait for the process
        if (sei.hProcess) {
            CloseHandle(sei.hProcess);
        }
    }
}