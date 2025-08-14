use anyhow::Result;
use crate::commands::AppInfo;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::fs;
use windows::Win32::Foundation::{CloseHandle, HANDLE, BOOL, HWND, LPARAM, MAX_PATH, WPARAM};
use std::sync::{Arc, Mutex};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, 
    QueryFullProcessImageNameW, PROCESS_NAME_WIN32
};
use windows::Win32::System::ProcessStatus::{GetModuleFileNameExW, GetProcessImageFileNameW};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowThreadProcessId, IsWindowVisible, EnumWindows, GetClassNameW, 
    GetWindowTextW, GetWindowTextLengthW, GetWindow, GW_OWNER, IsIconic,
    EnumChildWindows, HICON
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use log::{debug, warn};

// Constants for icon extraction
const ICON_BIG: u32 = 1;

// Structure to hold window information
struct WindowInfo {
    hwnd: HWND,
    pid: u32,
    class_name: String,
    window_title: String,
    is_uwp: bool,
}

pub fn get_running_apps() -> Result<Vec<AppInfo>> {
    let mut apps = Vec::new();
    let mut seen_identifiers = HashSet::new();
    let mut processed_pids = HashSet::new();
    
    // Get all visible windows with their info
    let window_infos = get_visible_windows()?;
    
    // First, handle UWP apps (ApplicationFrameWindows)
    for window in &window_infos {
        if window.class_name == "ApplicationFrameWindow" && !window.window_title.is_empty() {
            // This is a UWP app window - find the actual app process
            if let Some(app_info) = get_uwp_app_info_from_frame_window(window.hwnd, &window.window_title) {
                if !seen_identifiers.contains(&app_info.identifier) {
                    seen_identifiers.insert(app_info.identifier.clone());
                    apps.push(app_info);
                    // Mark the actual app PID as processed
                    if let Some(actual_pid) = get_actual_uwp_pid(window.hwnd) {
                        processed_pids.insert(actual_pid);
                    }
                }
            }
            // Mark ApplicationFrameHost PID as processed
            processed_pids.insert(window.pid);
        }
    }
    
    // Then handle regular Win32 apps
    let mut pid_to_windows: HashMap<u32, Vec<WindowInfo>> = HashMap::new();
    for window in window_infos {
        if !processed_pids.contains(&window.pid) && !window.is_uwp {
            pid_to_windows.entry(window.pid).or_insert_with(Vec::new).push(window);
        }
    }
    
    for (pid, _windows) in pid_to_windows {
        if !processed_pids.contains(&pid) {
            if let Some(app_info) = get_win32_app_info(pid) {
                if !seen_identifiers.contains(&app_info.identifier) {
                    seen_identifiers.insert(app_info.identifier.clone());
                    apps.push(app_info);
                }
            }
        }
    }
    
    // Sort apps alphabetically by display name
    apps.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    
    Ok(apps)
}

fn get_visible_windows() -> Result<Vec<WindowInfo>> {
    let mut windows = Vec::new();
    
    unsafe {
        // Enumerate all top-level windows
        EnumWindows(
            Some(enum_window_callback), 
            LPARAM(&mut windows as *mut _ as isize)
        )?;
    }
    
    Ok(windows)
}

unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
    
    // Check if window is visible and not minimized
    if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
        // Check if window has no owner (is a top-level window)
        let owner = GetWindow(hwnd, GW_OWNER);
        if owner.is_err() || owner.unwrap().0.is_null() {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            
            if pid != 0 {
                // Get window class name
                let mut class_name = vec![0u16; 256];
                let len = GetClassNameW(hwnd, &mut class_name);
                let class_name = if len > 0 {
                    String::from_utf16_lossy(&class_name[..len as usize])
                } else {
                    String::new()
                };
                
                // Get window title
                let title_len = GetWindowTextLengthW(hwnd);
                let window_title = if title_len > 0 {
                    let mut title = vec![0u16; (title_len + 1) as usize];
                    let actual_len = GetWindowTextW(hwnd, &mut title);
                    if actual_len > 0 {
                        String::from_utf16_lossy(&title[..actual_len as usize])
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                
                // Check if this is a UWP window
                let is_uwp = class_name == "ApplicationFrameWindow" || 
                             class_name == "Windows.UI.Core.CoreWindow";
                
                // Skip certain system windows
                if !should_skip_window(&class_name, &window_title) {
                    windows.push(WindowInfo {
                        hwnd,
                        pid,
                        class_name,
                        window_title,
                        is_uwp,
                    });
                }
            }
        }
    }
    
    BOOL::from(true) // Continue enumeration
}

fn should_skip_window(class_name: &str, title: &str) -> bool {
    // Skip system windows and shells
    class_name == "Shell_TrayWnd" || 
    class_name == "Shell_SecondaryTrayWnd" ||
    class_name == "Progman" ||
    class_name == "WorkerW" ||
    title == "Program Manager" ||
    title.is_empty()
}

// Get the actual UWP app PID from ApplicationFrameWindow
fn get_actual_uwp_pid(frame_hwnd: HWND) -> Option<u32> {
    unsafe {
        let enum_data = Arc::new(Mutex::new(EnumData {
            core_window: None,
        }));
        
        let enum_data_ptr = Arc::into_raw(enum_data.clone()) as isize;
        
        // Enumerate child windows to find the CoreWindow
        let _ = EnumChildWindows(
            frame_hwnd,
            Some(enum_child_callback),
            windows::Win32::Foundation::LPARAM(enum_data_ptr),
        );
        
        // Reclaim the Arc
        let enum_data = Arc::from_raw(enum_data_ptr as *const Mutex<EnumData>);
        
        // Get the PID of the CoreWindow
        let core_window = if let Ok(data) = enum_data.lock() {
            data.core_window
        } else {
            None
        };
        
        if let Some(core_hwnd) = core_window {
            let mut pid = 0u32;
            GetWindowThreadProcessId(core_hwnd, Some(&mut pid));
            if pid != 0 {
                return Some(pid);
            }
        }
    }
    None
}

// Get UWP app info from ApplicationFrameWindow
fn get_uwp_app_info_from_frame_window(frame_hwnd: HWND, window_title: &str) -> Option<AppInfo> {
    unsafe {
        // Find the actual app process via CoreWindow
        let actual_pid = get_actual_uwp_pid(frame_hwnd)?;
        
        // Get process info for the actual app
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            actual_pid
        ).ok()?;
        
        let process_path = get_process_full_path_ex(handle);
        let _ = CloseHandle(handle);
        let process_path = process_path.ok()?;
        
        // Extract app name
        let display_name = if !window_title.is_empty() && window_title != "Settings" {
            clean_uwp_title(window_title)
        } else {
            extract_uwp_app_name(&process_path)
        };
        
        // Create identifier from the actual exe name (e.g., "whatsapp.exe")
        let identifier = Path::new(&process_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        // Extract icon from UWP app manifest
        debug!("Extracting icon for UWP app: {} ({})", display_name, process_path);
        let icon_base64 = extract_uwp_logo_from_manifest(&process_path);
        
        if icon_base64.is_some() {
            debug!("Successfully extracted icon for {} from manifest", display_name);
        } else {
            warn!("Failed to extract icon for {} from manifest", display_name);
        }
        
        Some(AppInfo {
            display_name,
            identifier,
            icon_base64,
            is_running: true,
        })
    }
}

fn get_win32_app_info(pid: u32) -> Option<AppInfo> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid
        ).ok()?;
        
        let process_path = get_process_full_path_ex(handle);
        let _ = CloseHandle(handle);
        
        let process_path = process_path.ok()?;
        
        // Extract exe name
        let exe_name = Path::new(&process_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        // Extract display name
        let display_name = extract_display_name(&process_path);
        
        // Extract icon
        debug!("Extracting icon for Win32 app: {} ({})", display_name, process_path);
        let icon_base64 = extract_icon_as_base64(&process_path);
        
        if icon_base64.is_some() {
            debug!("Successfully extracted icon for {}", display_name);
        } else {
            warn!("Failed to extract icon for {} at path: {}", display_name, process_path);
        }
        
        Some(AppInfo {
            display_name,
            identifier: exe_name,
            icon_base64,
            is_running: true,
        })
    }
}

fn get_process_full_path_ex(handle: HANDLE) -> Result<String> {
    unsafe {
        let mut buffer = vec![0u16; MAX_PATH as usize];
        let mut size = buffer.len() as u32;
        
        // Try QueryFullProcessImageNameW first (works better for UWP)
        if QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size
        ).is_ok() {
            buffer.truncate(size as usize);
            return Ok(String::from_utf16_lossy(&buffer));
        }
        
        // Fallback to GetModuleFileNameExW
        let len = GetModuleFileNameExW(
            handle,
            None,
            &mut buffer
        );
        
        if len > 0 {
            buffer.truncate(len as usize);
            Ok(String::from_utf16_lossy(&buffer))
        } else {
            // Last resort: GetProcessImageFileNameW
            let len = GetProcessImageFileNameW(handle, &mut buffer);
            if len > 0 {
                buffer.truncate(len as usize);
                // This returns device path, need to convert
                let device_path = String::from_utf16_lossy(&buffer);
                Ok(convert_device_path_to_dos_path(&device_path).unwrap_or(device_path))
            } else {
                Err(anyhow::anyhow!("Failed to get process path"))
            }
        }
    }
}

fn convert_device_path_to_dos_path(device_path: &str) -> Option<String> {
    // Convert paths like \Device\HarddiskVolume3\... to C:\...
    if device_path.starts_with("\\Device\\") {
        // Simple heuristic: try common drive mappings
        let drives = ["C:", "D:", "E:", "F:"];
        for drive in &drives {
            if let Some(pos) = device_path.find("\\Windows\\") {
                return Some(format!("{}{}", drive, &device_path[pos..]));
            }
            if let Some(pos) = device_path.find("\\Program Files") {
                return Some(format!("{}{}", drive, &device_path[pos..]));
            }
            if let Some(pos) = device_path.find("\\Users\\") {
                return Some(format!("{}{}", drive, &device_path[pos..]));
            }
        }
    }
    None
}

fn clean_uwp_title(title: &str) -> String {
    // Remove common UWP app suffixes
    title
        .trim()
        .trim_end_matches(" - Personal")
        .trim_end_matches(" - Work or school")
        .to_string()
}

fn extract_uwp_app_name(path: &str) -> String {
    // Try to extract a meaningful name from UWP app path
    // UWP paths often look like: C:\Program Files\WindowsApps\<PackageName>\<Executable>
    if path.contains("WindowsApps") {
        if let Some(pos) = path.find("WindowsApps\\") {
            let after_apps = &path[pos + 12..];
            if let Some(slash_pos) = after_apps.find('\\') {
                let package_name = &after_apps[..slash_pos];
                // Extract app name from package name (e.g., "Microsoft.WindowsTerminal_xxx" -> "Windows Terminal")
                if let Some(dot_pos) = package_name.find('.') {
                    let app_part = &package_name[dot_pos + 1..];
                    if let Some(underscore_pos) = app_part.find('_') {
                        let name = &app_part[..underscore_pos];
                        // Convert CamelCase to spaced words
                        return split_camel_case(name);
                    }
                }
            }
        }
    }
    
    // Fallback to exe name
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| clean_display_name(s))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn split_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() && !result.is_empty() {
            // Check if next char is lowercase (indicates new word)
            if let Some(&next_ch) = chars.peek() {
                if next_ch.is_lowercase() {
                    result.push(' ');
                }
            }
        }
        result.push(ch);
    }
    
    result
}

// Not currently used - we're using manifest extraction only
#[allow(dead_code)]
fn extract_uwp_icon(exe_path: &str) -> Option<String> {
    // For UWP apps, try to find the app's Assets folder and look for icons
    if exe_path.contains("WindowsApps") {
        let path = Path::new(exe_path);
        if let Some(parent) = path.parent() {
            // Look for common icon files in the app directory
            let icon_candidates = [
                "Assets\\Square44x44Logo.targetsize-48_altform-unplated.png",
                "Assets\\Square44x44Logo.targetsize-48.png",
                "Assets\\Square44x44Logo.targetsize-32_altform-unplated.png",
                "Assets\\Square44x44Logo.targetsize-32.png",
                "Assets\\Square44x44Logo.targetsize-24_altform-unplated.png",
                "Assets\\Square44x44Logo.targetsize-24.png",
                "Assets\\Square44x44Logo.scale-200.png",
                "Assets\\Square44x44Logo.scale-150.png",
                "Assets\\Square44x44Logo.scale-125.png",
                "Assets\\Square44x44Logo.scale-100.png",
                "Assets\\Square44x44Logo.png",
                "Assets\\Square150x150Logo.scale-200.png",
                "Assets\\Square150x150Logo.scale-150.png",
                "Assets\\Square150x150Logo.scale-125.png",
                "Assets\\Square150x150Logo.scale-100.png",
                "Assets\\Square150x150Logo.png",
                "Assets\\StoreLogo.scale-200.png",
                "Assets\\StoreLogo.scale-150.png",
                "Assets\\StoreLogo.scale-125.png",
                "Assets\\StoreLogo.scale-100.png",
                "Assets\\StoreLogo.png",
                "Square44x44Logo.targetsize-48_altform-unplated.png",
                "Square44x44Logo.targetsize-48.png",
                "Square44x44Logo.scale-200.png",
                "Square44x44Logo.png",
                "Square150x150Logo.png",
                "StoreLogo.png",
            ];
            
            for icon_name in &icon_candidates {
                let icon_path = parent.join(icon_name);
                if icon_path.exists() {
                    // Read and encode the PNG file
                    if let Ok(png_data) = std::fs::read(&icon_path) {
                        use base64::{Engine as _, engine::general_purpose::STANDARD};
                        debug!("extract_uwp_icon: Successfully read icon from {:?}", icon_path);
                        return Some(STANDARD.encode(png_data));
                    }
                }
            }
            
            // Try searching recursively in Assets folder
            let assets_path = parent.join("Assets");
            if assets_path.exists() && assets_path.is_dir() {
                // Look for any PNG files that might be icons
                if let Ok(entries) = std::fs::read_dir(&assets_path) {
                    let mut png_files: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path().extension()
                                .and_then(|s| s.to_str())
                                .map(|s| s.eq_ignore_ascii_case("png"))
                                .unwrap_or(false)
                        })
                        .map(|e| e.path())
                        .collect();
                    
                    // Sort by file name to prioritize certain patterns
                    png_files.sort_by(|a, b| {
                        let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        
                        // Prioritize Square44x44Logo and StoreLogo
                        let a_priority = if a_name.contains("Square44x44Logo") { 0 }
                            else if a_name.contains("StoreLogo") { 1 }
                            else if a_name.contains("Square") { 2 }
                            else { 3 };
                        
                        let b_priority = if b_name.contains("Square44x44Logo") { 0 }
                            else if b_name.contains("StoreLogo") { 1 }
                            else if b_name.contains("Square") { 2 }
                            else { 3 };
                        
                        a_priority.cmp(&b_priority)
                    });
                    
                    // Try the first suitable PNG file
                    if let Some(icon_path) = png_files.first() {
                        if let Ok(png_data) = std::fs::read(icon_path) {
                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                            debug!("extract_uwp_icon: Successfully read PNG from Assets: {:?}", icon_path);
                            return Some(STANDARD.encode(png_data));
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn extract_display_name(path: &str) -> String {
    let file_name = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");
    
    clean_display_name(file_name)
}

fn clean_display_name(name: &str) -> String {
    // Remove common suffixes
    let cleaned = name
        .trim_end_matches(".exe")
        .trim_end_matches(".EXE");
    
    // Split on dashes and underscores, capitalize each word
    cleaned
        .split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_icon_as_base64(exe_path: &str) -> Option<String> {
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, SelectObject, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        BITMAP, GetObjectW, BitBlt, SRCCOPY
    };
    use windows::core::PCWSTR;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    unsafe {
        // Convert path to wide string
        let path_wide: Vec<u16> = OsStr::new(exe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        // Get file info with icon
        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(path_wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        
        if result == 0 || file_info.hIcon.0.is_null() {
            return None;
        }
        
        // Get icon info
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(file_info.hIcon, &mut icon_info).is_err() {
            let _ = DestroyIcon(file_info.hIcon);
            return None;
        }
        
        // Use color bitmap if available, otherwise use mask
        let bitmap_handle = if !icon_info.hbmColor.0.is_null() {
            icon_info.hbmColor
        } else if !icon_info.hbmMask.0.is_null() {
            icon_info.hbmMask
        } else {
            let _ = DestroyIcon(file_info.hIcon);
            return None;
        };
        
        // Get bitmap info
        let mut bitmap = BITMAP::default();
        if GetObjectW(
            bitmap_handle,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _)
        ) == 0 {
            if !icon_info.hbmColor.0.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.0.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(file_info.hIcon);
            return None;
        }
        
        // Create DIB section to copy icon data
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);
        
        // Prepare bitmap info for 32-bit RGBA
        let bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight, // Negative for top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            ..Default::default()
        };
        
        let mut bits_ptr = std::ptr::null_mut();
        let dib = CreateDIBSection(
            mem_dc,
            &bmp_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0
        ).ok()?;
        
        if dib.0.is_null() || bits_ptr.is_null() {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);
            if !icon_info.hbmColor.0.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.0.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(file_info.hIcon);
            return None;
        }
        
        // Select the DIB into memory DC and copy icon data
        let old_bmp = SelectObject(mem_dc, dib);
        
        // Create source DC and select the icon bitmap
        let src_dc = CreateCompatibleDC(screen_dc);
        let old_src_bmp = SelectObject(src_dc, bitmap_handle);
        
        // Copy the bitmap data
        let _ = BitBlt(
            mem_dc,
            0,
            0,
            bitmap.bmWidth,
            bitmap.bmHeight,
            src_dc,
            0,
            0,
            SRCCOPY,
        );
        
        // Calculate bitmap size
        let width = bitmap.bmWidth as usize;
        let height = bitmap.bmHeight.abs() as usize;
        let row_size = width * 4; // 32 bits per pixel
        let total_size = row_size * height;
        
        // Copy bitmap data
        let mut pixel_data = vec![0u8; total_size];
        std::ptr::copy_nonoverlapping(bits_ptr as *const u8, pixel_data.as_mut_ptr(), total_size);
        
        // Convert BGRA to RGBA
        for i in (0..total_size).step_by(4) {
            pixel_data.swap(i, i + 2); // Swap B and R
        }
        
        // Clean up GDI objects
        SelectObject(src_dc, old_src_bmp);
        let _ = DeleteDC(src_dc);
        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(dib);
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);
        if !icon_info.hbmColor.0.is_null() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.0.is_null() {
            let _ = DeleteObject(icon_info.hbmMask);
        }
        let _ = DestroyIcon(file_info.hIcon);
        
        // Convert to PNG using image crate
        use image::{ImageBuffer, Rgba};
        
        // Try to create image buffer
        let img = match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            width as u32,
            height as u32,
            pixel_data,
        ) {
            Some(img) => img,
            None => return None,
        };
        
        // Encode as PNG
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        match img.write_to(&mut cursor, image::ImageFormat::Png) {
            Ok(_) => {},
            Err(_) => return None,
        }
        
        // Convert to base64
        Some(STANDARD.encode(png_data))
    }
}

// Structure for enum callback data
struct EnumData {
    core_window: Option<HWND>,
}

// Helper function to extract icon from UWP ApplicationFrameWindow by finding CoreWindow child
// Not currently used - we're using manifest extraction only
#[allow(dead_code)]
fn extract_icon_from_uwp_window(hwnd: HWND) -> Option<String> {
    unsafe {
        let enum_data = Arc::new(Mutex::new(EnumData {
            core_window: None,
        }));
        
        let enum_data_ptr = Arc::into_raw(enum_data.clone()) as isize;
        
        // Enumerate child windows to find the CoreWindow
        let _ = EnumChildWindows(
            hwnd,
            Some(enum_child_callback),
            windows::Win32::Foundation::LPARAM(enum_data_ptr),
        );
        
        // Reclaim the Arc
        let enum_data = Arc::from_raw(enum_data_ptr as *const Mutex<EnumData>);
        
        // Check if we found a CoreWindow
        let core_window = if let Ok(data) = enum_data.lock() {
            data.core_window
        } else {
            None
        };
        
        if let Some(core_hwnd) = core_window {
            // Try to get icon from the CoreWindow
            return extract_icon_from_window(core_hwnd);
        }
    }
    
    None
}

unsafe extern "system" fn enum_child_callback(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> BOOL {
    let enum_data = Arc::from_raw(lparam.0 as *const Mutex<EnumData>);
    
    // Get the class name of this child window
    let mut class_name = vec![0u16; 256];
    let len = GetClassNameW(hwnd, &mut class_name);
    
    if len > 0 {
        let class_name = String::from_utf16_lossy(&class_name[..len as usize]);
        
        // Check if this is a CoreWindow
        if class_name == "Windows.UI.Core.CoreWindow" {
            if let Ok(mut data) = enum_data.lock() {
                data.core_window = Some(hwnd);
            }
            // Don't reclaim Arc here, just increment ref count
            std::mem::forget(enum_data);
            return BOOL::from(false); // Stop enumeration
        }
    }
    
    // Don't reclaim Arc here, just increment ref count
    std::mem::forget(enum_data);
    BOOL::from(true) // Continue enumeration
}

// Helper function to extract icon from a window handle
// Not currently used - we're using manifest extraction only
#[allow(dead_code)]
fn extract_icon_from_window(hwnd: HWND) -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO, CopyIcon};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, SelectObject, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        BITMAP, GetObjectW, BitBlt, SRCCOPY
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    unsafe {
        // Try to get icon from window using SendMessage with WM_GETICON
        use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, SMTO_ABORTIFHUNG, WM_GETICON};
        
        let mut icon_handle = HICON::default();
        let mut result = 0_isize;
        
        // Try to get large icon first
        let _ = SendMessageTimeoutW(
            hwnd,
            WM_GETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            100,
            Some(&mut result as *mut _ as *mut usize),
        );
        
        if result != 0 {
            icon_handle = HICON(result as *mut _);
        } else {
            // Try to get class icon
            use windows::Win32::UI::WindowsAndMessaging::{GetClassLongPtrW, GCLP_HICON};
            let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON);
            if class_icon != 0 {
                icon_handle = HICON(class_icon as *mut _);
            } else {
                return None;
            }
        }
        
        if icon_handle.0.is_null() {
            return None;
        }
        
        // Copy the icon to ensure we own it
        let owned_icon = CopyIcon(icon_handle).ok()?;
        
        // Get icon info
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(owned_icon, &mut icon_info).is_err() {
            let _ = DestroyIcon(owned_icon);
            return None;
        }
        
        // Use color bitmap if available
        let bitmap_handle = if !icon_info.hbmColor.0.is_null() {
            icon_info.hbmColor
        } else {
            let _ = DestroyIcon(owned_icon);
            return None;
        };
        
        // Get bitmap info
        let mut bitmap = BITMAP::default();
        if GetObjectW(
            bitmap_handle,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _)
        ) == 0 {
            if !icon_info.hbmColor.0.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.0.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(owned_icon);
            return None;
        }
        
        // Create DIB section to copy icon data
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);
        
        let bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight, // Negative for top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            ..Default::default()
        };
        
        let mut bits_ptr = std::ptr::null_mut();
        let dib = CreateDIBSection(
            mem_dc,
            &bmp_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0
        ).ok()?;
        
        if dib.0.is_null() || bits_ptr.is_null() {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);
            if !icon_info.hbmColor.0.is_null() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.0.is_null() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(owned_icon);
            return None;
        }
        
        // Select the DIB into memory DC and copy icon data
        let old_bmp = SelectObject(mem_dc, dib);
        
        // Create source DC and select the icon bitmap
        let src_dc = CreateCompatibleDC(screen_dc);
        let old_src_bmp = SelectObject(src_dc, bitmap_handle);
        
        // Copy the bitmap data
        let _ = BitBlt(
            mem_dc,
            0,
            0,
            bitmap.bmWidth,
            bitmap.bmHeight,
            src_dc,
            0,
            0,
            SRCCOPY,
        );
        
        // Calculate bitmap size
        let width = bitmap.bmWidth as usize;
        let height = bitmap.bmHeight.abs() as usize;
        let row_size = width * 4; // 32 bits per pixel
        let total_size = row_size * height;
        
        // Copy bitmap data
        let mut pixel_data = vec![0u8; total_size];
        std::ptr::copy_nonoverlapping(bits_ptr as *const u8, pixel_data.as_mut_ptr(), total_size);
        
        // Convert BGRA to RGBA
        for i in (0..total_size).step_by(4) {
            pixel_data.swap(i, i + 2); // Swap B and R
        }
        
        // Clean up GDI objects
        SelectObject(src_dc, old_src_bmp);
        let _ = DeleteDC(src_dc);
        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(dib);
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);
        if !icon_info.hbmColor.0.is_null() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.0.is_null() {
            let _ = DeleteObject(icon_info.hbmMask);
        }
        let _ = DestroyIcon(owned_icon);
        
        // Convert to PNG using image crate
        use image::{ImageBuffer, Rgba};
        
        // Try to create image buffer
        let img = match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            width as u32,
            height as u32,
            pixel_data,
        ) {
            Some(img) => img,
            None => return None,
        };
        
        // Encode as PNG
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        match img.write_to(&mut cursor, image::ImageFormat::Png) {
            Ok(_) => {},
            Err(_) => return None,
        }
        
        // Convert to base64
        Some(STANDARD.encode(png_data))
    }
}

// Extract UWP logo from AppxManifest.xml
fn extract_uwp_logo_from_manifest(exe_path: &str) -> Option<String> {
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    // Get the app directory
    let path = Path::new(exe_path);
    let app_dir = match path.parent() {
        Some(dir) => dir,
        None => {
            warn!("extract_uwp_logo_from_manifest: Failed to get parent directory for {}", exe_path);
            return None;
        }
    };
    
    // Read AppxManifest.xml
    let manifest_path = app_dir.join("AppxManifest.xml");
    if !manifest_path.exists() {
        warn!("extract_uwp_logo_from_manifest: AppxManifest.xml not found at {:?}", manifest_path);
        return None;
    }
    
    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            warn!("extract_uwp_logo_from_manifest: Failed to read manifest: {}", e);
            return None;
        }
    };
    
    // Parse logo path from manifest using simple regex
    // Look for <Logo>path</Logo> or Square44x44Logo="path"
    let logo_path = if let Some(caps) = extract_logo_from_manifest(&manifest_content) {
        caps
    } else {
        warn!("extract_uwp_logo_from_manifest: No logo path found in manifest");
        return None;
    };
    
    // Find the largest scale variant
    let logo_file = match find_largest_scale_logo(app_dir, &logo_path) {
        Some(file) => file,
        None => {
            warn!("extract_uwp_logo_from_manifest: No logo file found for path: {}", logo_path);
            return None;
        }
    };
    debug!("extract_uwp_logo_from_manifest: Selected logo file: {:?}", logo_file);
    
    // Read and encode the logo file
    let logo_data = match fs::read(&logo_file) {
        Ok(data) => data,
        Err(e) => {
            warn!("extract_uwp_logo_from_manifest: Failed to read logo file {:?}: {}", logo_file, e);
            return None;
        }
    };
    debug!("extract_uwp_logo_from_manifest: Successfully read {} bytes from logo file", logo_data.len());
    Some(STANDARD.encode(logo_data))
}

// Extract logo path from manifest XML
fn extract_logo_from_manifest(manifest: &str) -> Option<String> {
    // Try multiple patterns for logo paths
    let patterns = [
        r#"<Logo>([^<]+)</Logo>"#,
        r#"Square44x44Logo="([^"]+)""#,
        r#"Square150x150Logo="([^"]+)""#,
        r#"StoreLogo="([^"]+)""#,
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(manifest) {
                if let Some(logo_path) = caps.get(1) {
                    let path = logo_path.as_str().to_string();
                    return Some(path);
                }
            }
        }
    }
    
    None
}

// Find the largest scale variant of a logo file
fn find_largest_scale_logo(app_dir: &Path, logo_path: &str) -> Option<PathBuf> {
    let logo_path = Path::new(logo_path);
    let extension = match logo_path.extension() {
        Some(ext) => match ext.to_str() {
            Some(s) => s,
            None => {
                return None;
            }
        },
        None => {
            return None;
        }
    };
    
    let name_no_ext = match logo_path.file_stem() {
        Some(stem) => match stem.to_str() {
            Some(s) => s,
            None => {
                return None;
            }
        },
        None => {
            return None;
        }
    };
    
    // Try to find scale variants
    let mut best_file = None;
    let mut best_size = 0;
    let mut best_scale = 0;
    
    // First check if the exact file exists
    // The logo_path is relative to the app directory (where AppxManifest.xml is)
    let exact_path = app_dir.join(logo_path);
    if exact_path.exists() {
        if let Ok(metadata) = fs::metadata(&exact_path) {
            best_file = Some(exact_path.clone());
            best_size = metadata.len();
        }
    }
    
    // Look for scale variants in the same directory as the logo file
    // For example, if logo_path is "Images\System\StoreLogo.png", 
    // we should look in app_dir\Images\System\ for StoreLogo.scale-*.png
    let search_dir = if let Some(parent_path) = logo_path.parent() {
        // Has a parent directory (e.g., "Images\System")
        app_dir.join(parent_path)
    } else {
        // No parent directory, file is in root
        app_dir.to_path_buf()
    };
    
    if !search_dir.exists() {
        return best_file;
    }
    
    // Search for scale variants
    if let Ok(entries) = fs::read_dir(&search_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => match name.to_str() {
                    Some(s) => s,
                    None => continue,
                },
                None => continue,
            };
            
            // Check if it matches our base name and has the same extension
            // This handles patterns like:
            // - StoreLogo.scale-100.png
            // - StoreLogo.scale-200.png  
            // - StoreLogo.targetsize-48.png
            // - StoreLogo.png (the base file we already checked)
            if file_name.starts_with(name_no_ext) && file_name.ends_with(extension) {
                // Skip the exact file we already checked (compare just the filename)
                if let Some(logo_filename) = logo_path.file_name() {
                    if let Some(logo_filename_str) = logo_filename.to_str() {
                        if file_name == logo_filename_str {
                            continue;
                        }
                    }
                }
                
                // Skip contrast variants
                if file_name.contains("contrast") {
                    continue;
                }
                
                // Extract scale number from filename
                let scale = extract_scale_from_filename(file_name).unwrap_or(100);
                
                if let Ok(metadata) = fs::metadata(&path) {
                    let size = metadata.len();
                    
                    // Prefer higher scale or larger file size
                    if scale > best_scale || (scale == best_scale && size > best_size) {
                        best_file = Some(path);
                        best_size = size;
                        best_scale = scale;
                    }
                }
            }
        }
    }
    
    if let Some(ref file) = best_file {
        debug!("find_largest_scale_logo: Final selection: {:?} (scale: {}, size: {} bytes)", file, best_scale, best_size);
    }
    
    best_file
}

// Extract scale number from filename like "logo.scale-200.png" or "logo.targetsize-48.png"
fn extract_scale_from_filename(filename: &str) -> Option<u32> {
    if let Some(scale_pos) = filename.find(".scale-") {
        let after_scale = &filename[scale_pos + 7..];
        if let Some(dot_pos) = after_scale.find('.') {
            let scale_str = &after_scale[..dot_pos];
            return scale_str.parse().ok();
        }
    }
    
    if let Some(size_pos) = filename.find(".targetsize-") {
        let after_size = &filename[size_pos + 12..];
        if let Some(dot_pos) = after_size.find('.') {
            let size_str = &after_size[..dot_pos];
            // Convert targetsize to approximate scale (48 -> 100, 96 -> 200, etc.)
            if let Ok(size) = size_str.parse::<u32>() {
                return Some((size * 100) / 48);
            }
        }
    }
    
    None
}