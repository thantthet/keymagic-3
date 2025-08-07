use anyhow::Result;
use crate::commands::AppInfo;
use std::collections::HashSet;
use std::path::Path;
use windows::Win32::Foundation::{CloseHandle, HANDLE, BOOL};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, 
    TH32CS_SNAPPROCESS, PROCESSENTRY32W
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindowVisible, EnumWindows};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub fn get_running_apps() -> Result<Vec<AppInfo>> {
    let mut apps = Vec::new();
    let mut seen_identifiers = HashSet::new();
    
    // Get all visible window process IDs first
    let visible_pids = get_visible_window_pids()?;
    
    unsafe {
        // Create snapshot of all processes
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        
        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        
        // Get first process
        if Process32FirstW(snapshot, &mut process_entry).is_ok() {
            loop {
                let pid = process_entry.th32ProcessID;
                
                // Only process if it has a visible window
                if visible_pids.contains(&pid) {
                    // Get exe name from process entry
                    let exe_name = get_exe_name_from_entry(&process_entry);
                    
                    // Skip if we've already seen this exe
                    if !exe_name.is_empty() && !seen_identifiers.contains(&exe_name) {
                        // Try to get full path and extract icon
                        let (display_name, full_path_opt, icon_base64) = if let Ok(handle) = OpenProcess(
                            PROCESS_QUERY_INFORMATION,
                            false,
                            pid
                        ) {
                            let full_path = get_process_full_path(handle);
                            let _ = CloseHandle(handle);
                            
                            if let Ok(path) = full_path {
                                let display = extract_display_name(&path);
                                let icon = extract_icon_as_base64(&path);
                                (display, Some(path), icon)
                            } else {
                                (clean_display_name(&exe_name), None, None)
                            }
                        } else {
                            (clean_display_name(&exe_name), None, None)
                        };
                        
                        // If we couldn't get icon from full path, try with just exe name
                        let icon_base64 = icon_base64.or_else(|| {
                            // Try to find the exe in common locations
                            let common_paths = vec![
                                format!("C:\\Program Files\\{}\\{}", display_name, exe_name),
                                format!("C:\\Program Files (x86)\\{}\\{}", display_name, exe_name),
                                format!("C:\\Windows\\System32\\{}", exe_name),
                                format!("C:\\Windows\\{}", exe_name),
                            ];
                            
                            for path in common_paths {
                                if std::path::Path::new(&path).exists() {
                                    if let Some(icon) = extract_icon_as_base64(&path) {
                                        return Some(icon);
                                    }
                                }
                            }
                            
                            // Last resort: try with just the exe name (system might find it)
                            full_path_opt.as_ref().and_then(|p| extract_icon_as_base64(p))
                        });
                        
                        apps.push(AppInfo {
                            display_name,
                            identifier: exe_name.clone(),
                            icon_base64,
                            is_running: true,
                        });
                        
                        seen_identifiers.insert(exe_name);
                    }
                }
                
                // Get next process
                if Process32NextW(snapshot, &mut process_entry).is_err() {
                    break;
                }
            }
        }
        
        let _ = CloseHandle(snapshot);
    }
    
    // Sort apps alphabetically by display name
    apps.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    
    Ok(apps)
}

fn get_visible_window_pids() -> Result<HashSet<u32>> {
    let mut pids = HashSet::new();
    
    unsafe {
        // Enumerate all top-level windows
        EnumWindows(Some(enum_window_callback), windows::Win32::Foundation::LPARAM(&mut pids as *mut _ as isize))?
    }
    
    Ok(pids)
}

unsafe extern "system" fn enum_window_callback(hwnd: windows::Win32::Foundation::HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
    let pids = &mut *(lparam.0 as *mut HashSet<u32>);
    
    // Check if window is visible
    if IsWindowVisible(hwnd).into() {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        
        if pid != 0 {
            pids.insert(pid);
        }
    }
    
    BOOL::from(true) // Continue enumeration
}

fn get_exe_name_from_entry(entry: &PROCESSENTRY32W) -> String {
    // Convert wide string to String
    let exe_file: Vec<u16> = entry.szExeFile
        .iter()
        .take_while(|&&c| c != 0)
        .copied()
        .collect();
    
    String::from_utf16_lossy(&exe_file).to_lowercase()
}

fn get_process_full_path(handle: HANDLE) -> Result<String> {
    unsafe {
        let mut buffer = vec![0u16; 1024];
        let len = GetModuleFileNameExW(
            handle,
            None,
            &mut buffer
        );
        
        if len == 0 {
            return Err(anyhow::anyhow!("Failed to get module filename"));
        }
        
        buffer.truncate(len as usize);
        Ok(String::from_utf16_lossy(&buffer))
    }
}

fn extract_display_name(path: &str) -> String {
    // Try to extract product name from file version info
    // For now, just use the filename without extension
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| clean_display_name(s))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn clean_display_name(name: &str) -> String {
    // Remove common suffixes and clean up the name
    let cleaned = name
        .replace(".exe", "")
        .replace("-", " ")
        .replace("_", " ");
    
    // Capitalize first letter of each word
    cleaned.split_whitespace()
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
        GetDIBits, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
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
            PCWSTR::from_raw(path_wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        
        if result == 0 || file_info.hIcon.is_invalid() {
            return None;
        }
        
        // Get icon info
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(file_info.hIcon, &mut icon_info).is_err() {
            DestroyIcon(file_info.hIcon);
            return None;
        }
        
        // Use color bitmap if available, otherwise use mask
        let bitmap_handle = if !icon_info.hbmColor.is_invalid() {
            icon_info.hbmColor
        } else if !icon_info.hbmMask.is_invalid() {
            icon_info.hbmMask
        } else {
            DestroyIcon(file_info.hIcon);
            return None;
        };
        
        // Get bitmap info
        let mut bitmap = BITMAP::default();
        if GetObjectW(
            bitmap_handle,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _)
        ) == 0 {
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(file_info.hIcon);
            return None;
        }
        
        // Create device contexts
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);
        
        // Prepare bitmap info for 32-bit RGBA
        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight, // Negative for top-down bitmap
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };
        
        // Create DIB section
        let mut bits_ptr = std::ptr::null_mut();
        let dib = CreateDIBSection(
            mem_dc,
            &bmp_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        );
        
        if dib.is_err() || bits_ptr.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(None, screen_dc);
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(file_info.hIcon);
            return None;
        }
        
        let dib = dib.unwrap();
        let old_bmp = SelectObject(mem_dc, dib);
        
        // Create source DC and select the icon bitmap
        let src_dc = CreateCompatibleDC(screen_dc);
        let old_src_bmp = SelectObject(src_dc, bitmap_handle);
        
        // Copy the icon bitmap to our DIB
        BitBlt(
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
        DeleteDC(src_dc);
        SelectObject(mem_dc, old_bmp);
        DeleteObject(dib);
        DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        if !icon_info.hbmColor.is_invalid() {
            DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_invalid() {
            DeleteObject(icon_info.hbmMask);
        }
        DestroyIcon(file_info.hIcon);
        
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