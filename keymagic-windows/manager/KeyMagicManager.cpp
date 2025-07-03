#define UNICODE
#define _UNICODE

#include <windows.h>
#include <commctrl.h>
#include <shellapi.h>
#include <shlwapi.h>
#include <string>
#include <vector>
#include <fstream>
#include "resource.h"

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "shlwapi.lib")

// Window class name
const wchar_t* g_szClassName = L"KeyMagicManagerWindow";

// Control IDs
#define ID_LISTVIEW_KEYBOARDS   1001
#define ID_BUTTON_ADD          1002
#define ID_BUTTON_REMOVE       1003
#define ID_BUTTON_ACTIVATE     1004
#define ID_BUTTON_SETTINGS     1005
#define ID_STATIC_INFO         1006

// Global variables
HINSTANCE g_hInst;
HWND g_hListView;
HWND g_hButtonAdd;
HWND g_hButtonRemove;
HWND g_hButtonActivate;
HWND g_hButtonSettings;
HWND g_hStaticInfo;

// Keyboard info structure
struct KeyboardInfo {
    std::wstring name;
    std::wstring description;
    std::wstring filePath;
    bool isActive;
};

std::vector<KeyboardInfo> g_keyboards;
int g_activeKeyboardIndex = -1;

// Forward declarations
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
void CreateControls(HWND hwnd);
void PopulateKeyboardList();
void RefreshKeyboardList();
void AddKeyboard();
void RemoveKeyboard();
void ActivateKeyboard();
void ShowSettings();
void LoadKeyboardsFromRegistry();
void SaveKeyboardsToRegistry();
std::wstring BrowseForKM2File(HWND hwnd);

// Main entry point
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    g_hInst = hInstance;

    // Initialize common controls
    INITCOMMONCONTROLSEX icex;
    icex.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icex.dwICC = ICC_LISTVIEW_CLASSES;
    InitCommonControlsEx(&icex);

    // Register window class
    WNDCLASSEX wc;
    wc.cbSize = sizeof(WNDCLASSEX);
    wc.style = 0;
    wc.lpfnWndProc = WndProc;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hInstance = hInstance;
    wc.hIcon = LoadIcon(NULL, IDI_APPLICATION);
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.lpszMenuName = NULL;
    wc.lpszClassName = g_szClassName;
    wc.hIconSm = LoadIcon(NULL, IDI_APPLICATION);

    if (!RegisterClassEx(&wc)) {
        MessageBox(NULL, L"Window Registration Failed!", L"Error", MB_ICONEXCLAMATION | MB_OK);
        return 0;
    }

    // Create main window
    HWND hwnd = CreateWindowEx(
        WS_EX_WINDOWEDGE,
        g_szClassName,
        L"KeyMagic Keyboard Manager",
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 600, 400,
        NULL, NULL, hInstance, NULL
    );

    if (hwnd == NULL) {
        MessageBox(NULL, L"Window Creation Failed!", L"Error", MB_ICONEXCLAMATION | MB_OK);
        return 0;
    }

    ShowWindow(hwnd, nCmdShow);
    UpdateWindow(hwnd);

    // Message loop
    MSG Msg;
    while (GetMessage(&Msg, NULL, 0, 0) > 0) {
        TranslateMessage(&Msg);
        DispatchMessage(&Msg);
    }

    return Msg.wParam;
}

// Window procedure
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_CREATE:
            CreateControls(hwnd);
            LoadKeyboardsFromRegistry();
            RefreshKeyboardList();
            break;

        case WM_SIZE: {
            RECT rcClient;
            GetClientRect(hwnd, &rcClient);
            int width = rcClient.right;
            int height = rcClient.bottom;

            // Resize controls
            SetWindowPos(g_hListView, NULL, 10, 10, width - 130, height - 60, SWP_NOZORDER);
            SetWindowPos(g_hButtonAdd, NULL, width - 110, 10, 100, 30, SWP_NOZORDER);
            SetWindowPos(g_hButtonRemove, NULL, width - 110, 50, 100, 30, SWP_NOZORDER);
            SetWindowPos(g_hButtonActivate, NULL, width - 110, 90, 100, 30, SWP_NOZORDER);
            SetWindowPos(g_hButtonSettings, NULL, width - 110, 130, 100, 30, SWP_NOZORDER);
            SetWindowPos(g_hStaticInfo, NULL, 10, height - 40, width - 20, 30, SWP_NOZORDER);
            break;
        }

        case WM_COMMAND:
            switch (LOWORD(wParam)) {
                case ID_BUTTON_ADD:
                    AddKeyboard();
                    break;
                case ID_BUTTON_REMOVE:
                    RemoveKeyboard();
                    break;
                case ID_BUTTON_ACTIVATE:
                    ActivateKeyboard();
                    break;
                case ID_BUTTON_SETTINGS:
                    ShowSettings();
                    break;
            }
            break;

        case WM_NOTIFY: {
            LPNMHDR pnmh = (LPNMHDR)lParam;
            if (pnmh->idFrom == ID_LISTVIEW_KEYBOARDS) {
                switch (pnmh->code) {
                    case NM_DBLCLK:
                        ActivateKeyboard();
                        break;
                }
            }
            break;
        }

        case WM_CLOSE:
            SaveKeyboardsToRegistry();
            DestroyWindow(hwnd);
            break;

        case WM_DESTROY:
            PostQuitMessage(0);
            break;

        default:
            return DefWindowProc(hwnd, msg, wParam, lParam);
    }
    return 0;
}

// Create child controls
void CreateControls(HWND hwnd) {
    // Create ListView
    g_hListView = CreateWindowEx(
        WS_EX_CLIENTEDGE,
        WC_LISTVIEW,
        L"",
        WS_CHILD | WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL,
        10, 10, 460, 300,
        hwnd, (HMENU)ID_LISTVIEW_KEYBOARDS, g_hInst, NULL
    );

    // Set extended styles
    ListView_SetExtendedListViewStyle(g_hListView, LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES);

    // Add columns
    LVCOLUMN lvc;
    lvc.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM;

    lvc.iSubItem = 0;
    lvc.pszText = (LPWSTR)L"Name";
    lvc.cx = 150;
    ListView_InsertColumn(g_hListView, 0, &lvc);

    lvc.iSubItem = 1;
    lvc.pszText = (LPWSTR)L"Description";
    lvc.cx = 200;
    ListView_InsertColumn(g_hListView, 1, &lvc);

    lvc.iSubItem = 2;
    lvc.pszText = (LPWSTR)L"Status";
    lvc.cx = 80;
    ListView_InsertColumn(g_hListView, 2, &lvc);

    // Create buttons
    g_hButtonAdd = CreateWindow(
        L"BUTTON", L"Add...",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        480, 10, 100, 30,
        hwnd, (HMENU)ID_BUTTON_ADD, g_hInst, NULL
    );

    g_hButtonRemove = CreateWindow(
        L"BUTTON", L"Remove",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        480, 50, 100, 30,
        hwnd, (HMENU)ID_BUTTON_REMOVE, g_hInst, NULL
    );

    g_hButtonActivate = CreateWindow(
        L"BUTTON", L"Activate",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        480, 90, 100, 30,
        hwnd, (HMENU)ID_BUTTON_ACTIVATE, g_hInst, NULL
    );

    g_hButtonSettings = CreateWindow(
        L"BUTTON", L"Settings...",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        480, 130, 100, 30,
        hwnd, (HMENU)ID_BUTTON_SETTINGS, g_hInst, NULL
    );

    // Create status text
    g_hStaticInfo = CreateWindow(
        L"STATIC", L"Select a keyboard layout to activate",
        WS_CHILD | WS_VISIBLE | SS_LEFT,
        10, 330, 570, 30,
        hwnd, (HMENU)ID_STATIC_INFO, g_hInst, NULL
    );

    // Set font for all controls
    HFONT hFont = (HFONT)GetStockObject(DEFAULT_GUI_FONT);
    SendMessage(g_hListView, WM_SETFONT, (WPARAM)hFont, TRUE);
    SendMessage(g_hButtonAdd, WM_SETFONT, (WPARAM)hFont, TRUE);
    SendMessage(g_hButtonRemove, WM_SETFONT, (WPARAM)hFont, TRUE);
    SendMessage(g_hButtonActivate, WM_SETFONT, (WPARAM)hFont, TRUE);
    SendMessage(g_hButtonSettings, WM_SETFONT, (WPARAM)hFont, TRUE);
    SendMessage(g_hStaticInfo, WM_SETFONT, (WPARAM)hFont, TRUE);
}

// Refresh the keyboard list view
void RefreshKeyboardList() {
    ListView_DeleteAllItems(g_hListView);

    for (size_t i = 0; i < g_keyboards.size(); i++) {
        LVITEM lvi = {0};
        lvi.mask = LVIF_TEXT;
        lvi.iItem = i;

        // Name
        lvi.iSubItem = 0;
        lvi.pszText = (LPWSTR)g_keyboards[i].name.c_str();
        ListView_InsertItem(g_hListView, &lvi);

        // Description
        lvi.iSubItem = 1;
        lvi.pszText = (LPWSTR)g_keyboards[i].description.c_str();
        ListView_SetItem(g_hListView, &lvi);

        // Status
        lvi.iSubItem = 2;
        lvi.pszText = (LPWSTR)(g_keyboards[i].isActive ? L"Active" : L"");
        ListView_SetItem(g_hListView, &lvi);
    }

    // Update button states
    BOOL hasSelection = ListView_GetSelectedCount(g_hListView) > 0;
    EnableWindow(g_hButtonRemove, hasSelection);
    EnableWindow(g_hButtonActivate, hasSelection);
}

// Add a new keyboard
void AddKeyboard() {
    std::wstring filePath = BrowseForKM2File(GetParent(g_hListView));
    if (filePath.empty()) return;

    // Extract filename as keyboard name
    wchar_t fileName[MAX_PATH];
    wcscpy_s(fileName, PathFindFileName(filePath.c_str()));
    PathRemoveExtension(fileName);

    // Check if already added
    for (const auto& kb : g_keyboards) {
        if (kb.filePath == filePath) {
            MessageBox(GetParent(g_hListView), L"This keyboard is already added.", L"Information", MB_OK | MB_ICONINFORMATION);
            return;
        }
    }

    // Add to list
    KeyboardInfo kb;
    kb.name = fileName;
    kb.description = L"KeyMagic Keyboard Layout";
    kb.filePath = filePath;
    kb.isActive = false;

    g_keyboards.push_back(kb);
    RefreshKeyboardList();
    SaveKeyboardsToRegistry();
}

// Remove selected keyboard
void RemoveKeyboard() {
    int selected = ListView_GetNextItem(g_hListView, -1, LVNI_SELECTED);
    if (selected < 0) return;

    if (g_keyboards[selected].isActive) {
        MessageBox(GetParent(g_hListView), L"Cannot remove active keyboard. Please activate another keyboard first.", L"Error", MB_OK | MB_ICONERROR);
        return;
    }

    g_keyboards.erase(g_keyboards.begin() + selected);
    RefreshKeyboardList();
    SaveKeyboardsToRegistry();
}

// Activate selected keyboard
void ActivateKeyboard() {
    int selected = ListView_GetNextItem(g_hListView, -1, LVNI_SELECTED);
    if (selected < 0) return;

    // Deactivate all keyboards
    for (auto& kb : g_keyboards) {
        kb.isActive = false;
    }

    // Activate selected
    g_keyboards[selected].isActive = true;
    g_activeKeyboardIndex = selected;

    // TODO: Actually load the keyboard into the TSF engine
    // This would involve IPC with the TSF DLL or updating registry settings
    
    RefreshKeyboardList();
    SaveKeyboardsToRegistry();

    // Show status
    std::wstring status = L"Activated: " + g_keyboards[selected].name;
    SetWindowText(g_hStaticInfo, status.c_str());
}

// Show settings dialog
void ShowSettings() {
    MessageBox(GetParent(g_hListView), 
        L"Settings:\n\n"
        L"• Keyboard layouts are stored in: HKCU\\Software\\KeyMagic\\Keyboards\n"
        L"• Active keyboard is loaded when TSF starts\n"
        L"• Use Win+Space to switch input methods\n\n"
        L"Version: 1.0.0",
        L"KeyMagic Settings", MB_OK | MB_ICONINFORMATION);
}

// Browse for KM2 file
std::wstring BrowseForKM2File(HWND hwnd) {
    wchar_t filename[MAX_PATH] = L"";
    
    OPENFILENAME ofn = {0};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = hwnd;
    ofn.lpstrFilter = L"KeyMagic Keyboard Files (*.km2)\0*.km2\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filename;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrTitle = L"Select KeyMagic Keyboard File";
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
    ofn.lpstrDefExt = L"km2";

    if (GetOpenFileName(&ofn)) {
        return std::wstring(filename);
    }
    return L"";
}

// Load keyboards from registry
void LoadKeyboardsFromRegistry() {
    g_keyboards.clear();
    
    HKEY hKey;
    if (RegOpenKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Keyboards", 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        DWORD index = 0;
        wchar_t valueName[256];
        DWORD valueNameSize;
        BYTE data[1024];
        DWORD dataSize;
        DWORD type;

        while (true) {
            valueNameSize = 256;
            dataSize = 1024;
            
            if (RegEnumValue(hKey, index, valueName, &valueNameSize, NULL, &type, data, &dataSize) != ERROR_SUCCESS) {
                break;
            }

            if (type == REG_SZ) {
                KeyboardInfo kb;
                kb.name = valueName;
                kb.filePath = (wchar_t*)data;
                kb.description = L"KeyMagic Keyboard Layout";
                kb.isActive = false;
                g_keyboards.push_back(kb);
            }

            index++;
        }

        // Load active keyboard index
        DWORD activeIndex = 0;
        DWORD size = sizeof(DWORD);
        if (RegQueryValueEx(hKey, L"ActiveKeyboard", NULL, NULL, (BYTE*)&activeIndex, &size) == ERROR_SUCCESS) {
            if (activeIndex < g_keyboards.size()) {
                g_keyboards[activeIndex].isActive = true;
                g_activeKeyboardIndex = activeIndex;
            }
        }

        RegCloseKey(hKey);
    }
}

// Save keyboards to registry
void SaveKeyboardsToRegistry() {
    HKEY hKey;
    DWORD disposition;
    
    if (RegCreateKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Keyboards", 0, NULL, 
                       REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hKey, &disposition) == ERROR_SUCCESS) {
        
        // Clear existing entries
        if (disposition == REG_OPENED_EXISTING_KEY) {
            // Delete all values
            RegDeleteTree(hKey, NULL);
            RegCloseKey(hKey);
            
            // Recreate
            RegCreateKeyEx(HKEY_CURRENT_USER, L"Software\\KeyMagic\\Keyboards", 0, NULL, 
                          REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hKey, &disposition);
        }

        // Save keyboards
        for (size_t i = 0; i < g_keyboards.size(); i++) {
            RegSetValueEx(hKey, g_keyboards[i].name.c_str(), 0, REG_SZ, 
                         (BYTE*)g_keyboards[i].filePath.c_str(), 
                         (g_keyboards[i].filePath.length() + 1) * sizeof(wchar_t));
        }

        // Save active keyboard index
        if (g_activeKeyboardIndex >= 0) {
            DWORD activeIndex = g_activeKeyboardIndex;
            RegSetValueEx(hKey, L"ActiveKeyboard", 0, REG_DWORD, (BYTE*)&activeIndex, sizeof(DWORD));
        }

        RegCloseKey(hKey);
    }
}