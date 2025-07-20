use std::sync::Mutex;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use anyhow::{Result, anyhow};
use log::{debug, error};

static HUD_INSTANCE: OnceLock<Mutex<HWND>> = OnceLock::new();

const HUD_TIMER_ID: usize = 1;
const HUD_DISPLAY_TIME_MS: u32 = 1500;
const WM_SHOW_HUD: u32 = WM_USER + 1;
const AC_SRC_OVER: u8 = 0x00;
const AC_SRC_ALPHA: u8 = 0x01;

/// Initialize the HUD window
pub fn initialize_hud() -> Result<()> {
    debug!("Initializing HUD window");
    unsafe {
        let instance = GetModuleHandleW(None)?;
        debug!("Got module handle");
        
        // Register window class
        let class_name = w!("KeyMagicHUD");
        
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(hud_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: std::mem::size_of::<*mut c_void>() as i32,
            hInstance: instance.into(),
            hIcon: HICON::default(),
            hCursor: LoadCursorW(HINSTANCE::default(), IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            let err = GetLastError();
            if err != Ok(()) {
                // Check if it's not already registered
                // For now, continue even if registration failed
            }
        }

        // Create window
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            w!(""),
            WS_POPUP,
            0, 0, 0, 0,
            HWND::default(),
            HMENU::default(),
            instance,
            None,
        );

        if hwnd.0 == 0 {
            return Err(anyhow!("Failed to create HUD window"));
        }
        
        debug!("HUD window created successfully: {:?}", hwnd);

        // Store window handle
        HUD_INSTANCE.set(Mutex::new(hwnd))
            .map_err(|_| anyhow!("Failed to store HUD window handle"))?;
            
        debug!("HUD initialization complete");

        Ok(())
    }
}

/// Show the HUD with keyboard name
pub fn show_keyboard_hud(keyboard_name: &str) -> Result<()> {
    debug!("show_keyboard_hud called with: {}", keyboard_name);
    if let Some(hwnd_mutex) = HUD_INSTANCE.get() {
        if let Ok(hwnd) = hwnd_mutex.lock() {
            unsafe {
                debug!("Sending HUD message for: {}", keyboard_name);
                // Allocate and copy string
                let text_wide: Vec<u16> = keyboard_name.encode_utf16().chain(std::iter::once(0)).collect();
                let text_ptr = Box::into_raw(Box::new(text_wide));
                
                // Send message to show HUD
                let result = PostMessageW(*hwnd, WM_SHOW_HUD, WPARAM(0), LPARAM(text_ptr as isize));
                if result.is_err() {
                    error!("Failed to post message: {:?}", result);
                }
            }
        } else {
            error!("Failed to lock HUD mutex");
        }
    } else {
        error!("HUD not initialized");
    }
    Ok(())
}


/// Show tray minimize notification
pub fn show_tray_minimize_notification() -> Result<()> {
    show_keyboard_hud("KeyMagic is running in the system tray")
}

/// Window procedure for HUD
extern "system" fn hud_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_SHOW_HUD => {
                debug!("WM_SHOW_HUD received");
                // Get keyboard name from lparam
                let text_ptr = lparam.0 as *mut Vec<u16>;
                if !text_ptr.is_null() {
                    let text_vec = Box::from_raw(text_ptr);
                    let text_str = String::from_utf16_lossy(&text_vec);
                    debug!("Showing HUD for: {}", text_str);
                    
                    // Show HUD
                    show_hud_internal(hwnd, &text_vec);
                    
                    // Set timer
                    let _ = SetTimer(hwnd, HUD_TIMER_ID, HUD_DISPLAY_TIME_MS, None);
                }
                LRESULT(0)
            }
            WM_TIMER => {
                if wparam.0 == HUD_TIMER_ID {
                    hide_hud(hwnd);
                    let _ = KillTimer(hwnd, HUD_TIMER_ID);
                }
                LRESULT(0)
            }
            WM_NCHITTEST => {
                // Make window click-through
                LRESULT(-1)  // HTNOWHERE
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Internal function to show HUD
fn show_hud_internal(hwnd: HWND, text: &[u16]) {
    debug!("show_hud_internal called");
    unsafe {
        // First make the window visible
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        let hdc = GetDC(HWND_DESKTOP);
        let mem_dc = CreateCompatibleDC(hdc);
        
        // Create font
        let font = CreateFontW(
            -mul_div(20, GetDeviceCaps(hdc, LOGPIXELSY), 72),
            0, 0, 0,
            FW_NORMAL.0 as i32,
            FALSE.0 as u32,
            FALSE.0 as u32,
            FALSE.0 as u32,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            DEFAULT_QUALITY.0 as u32,
            DEFAULT_PITCH.0 as u32 | FF_SWISS.0 as u32,
            w!("Segoe UI"),
        );
        
        let _old_font = SelectObject(mem_dc, font);
        
        // Measure text
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(mem_dc, text, &mut size);
        
        let padding = 20;
        let width = size.cx + (padding * 2);
        let height = size.cy + (padding * 2);
        
        // Create bitmap
        let bitmap = CreateCompatibleBitmap(hdc, width, height);
        let _old_bitmap = SelectObject(mem_dc, bitmap);
        
        // Define colors
        let transparent_color = rgb(255, 0, 255); // Magenta for transparency
        let bg_color = rgb(0, 0, 0); // Black background
        let text_color = rgb(255, 255, 255); // White text
        
        // First fill entire bitmap with transparent color
        let transparent_pen = CreatePen(PS_SOLID, 0, transparent_color);
        let _ = SelectObject(mem_dc, transparent_pen);
        let transparent_brush = CreateSolidBrush(transparent_color);
        let _ = SelectObject(mem_dc, transparent_brush);
        let _ = Rectangle(mem_dc, 0, 0, width, height);
        
        // Draw rounded rectangle with black background
        let bg_brush = CreateSolidBrush(bg_color);
        let _ = SelectObject(mem_dc, bg_brush);
        let _ = RoundRect(mem_dc, 0, 0, width, height, 22, 22);
        
        // Draw text
        let _ = SetBkMode(mem_dc, TRANSPARENT);
        let _ = SetTextColor(mem_dc, text_color);
        let rect = RECT { left: 0, top: 0, right: width, bottom: height };
        let mut text_rect = rect.clone();
        let mut text_copy: Vec<u16> = text.to_vec();
        let _ = DrawTextW(mem_dc, &mut text_copy[..], &mut text_rect as *mut _, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
        
        // Apply alpha channel similar to C++ sample
        set_bitmap_alpha(mem_dc, bitmap, transparent_color, text_color);
        
        // Cleanup
        let _ = DeleteObject(transparent_pen);
        let _ = DeleteObject(transparent_brush);
        let _ = DeleteObject(bg_brush);
        
        // Update layered window
        update_layered_window(hwnd, mem_dc, width, height);
        
        // Cleanup
        let _ = DeleteObject(font);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(HWND_DESKTOP, hdc);
    }
}

/// Update layered window with bitmap
fn update_layered_window(hwnd: HWND, mem_dc: HDC, width: i32, height: i32) {
    debug!("update_layered_window called with size: {}x{}", width, height);
    unsafe {
        // Get monitor info
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut monitor_info);
        
        // Calculate position - bottom-right corner
        let work_area = monitor_info.rcWork;
        let margin = 20;
        let x = work_area.right - width - margin;
        let y = work_area.bottom - height - margin - 50;
        
        let size = SIZE { cx: width, cy: height };
        let src_point = POINT { x: 0, y: 0 };
        let dst_point = POINT { x, y };
        
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,  // Use per-pixel alpha
            AlphaFormat: AC_SRC_ALPHA as u8,  // Important: use per-pixel alpha
        };
        
        let transparent_color = rgb(255, 0, 255); // Magenta for transparency
        
        let _ = UpdateLayeredWindow(
            hwnd,
            HDC::default(),
            Some(&dst_point),
            Some(&size),
            mem_dc,
            Some(&src_point),
            transparent_color,  // Use transparent color like C++ sample
            Some(&blend),
            ULW_ALPHA,  // Use ULW_ALPHA like C++ sample
        );
    }
}

/// Hide the HUD
fn hide_hud(hwnd: HWND) {
    unsafe {
        // Hide by moving off-screen
        let _ = SetWindowPos(
            hwnd,
            HWND_TOP,
            -1000, -1000, 0, 0,
            SWP_HIDEWINDOW | SWP_NOACTIVATE,
        );
    }
}

// MulDiv implementation for Windows
fn mul_div(n_number: i32, n_numerator: i32, n_denominator: i32) -> i32 {
    ((n_number as i64 * n_numerator as i64) / n_denominator as i64) as i32
}

// RGB macro to create COLORREF
fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

// Set bitmap alpha based on colors (matching C++ sample)
fn set_bitmap_alpha(hdc: HDC, bitmap: HBITMAP, transparent_color: COLORREF, text_color: COLORREF) {
    unsafe {
        let mut bm = BITMAP::default();
        GetObjectW(bitmap, std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut c_void));
        
        // Create BITMAPINFO
        let bmi_size = std::mem::size_of::<BITMAPINFOHEADER>() + (256 * std::mem::size_of::<RGBQUAD>());
        let mut bmi_buffer = vec![0u8; bmi_size];
        let bmi = bmi_buffer.as_mut_ptr() as *mut BITMAPINFO;
        (*bmi).bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        
        // Get bitmap info
        let _ = GetDIBits(hdc, bitmap, 0, bm.bmHeight as u32, None, bmi, DIB_RGB_COLORS);
        
        if (*bmi).bmiHeader.biBitCount != 32 {
            return;
        }
        
        // Allocate buffer for pixel data
        let pixel_count = (bm.bmWidth * bm.bmHeight) as usize;
        let mut pixel_data = vec![0u8; pixel_count * 4];
        
        // Get pixel data
        let _ = GetDIBits(hdc, bitmap, 0, bm.bmHeight as u32, Some(pixel_data.as_mut_ptr() as *mut c_void), bmi, DIB_RGB_COLORS);
        
        // Process each pixel
        for i in 0..pixel_count {
            let offset = i * 4;
            let b = pixel_data[offset];
            let g = pixel_data[offset + 1];
            let r = pixel_data[offset + 2];
            let color = rgb(r, g, b);
            
            let alpha = if color == transparent_color {
                0u8
            } else if color == text_color {
                255u8
            } else {
                (255.0 * 0.8) as u8 // 80% opacity for background
            };
            
            pixel_data[offset + 3] = alpha;
            // Premultiply alpha
            pixel_data[offset] = (b as u32 * alpha as u32 / 255) as u8;
            pixel_data[offset + 1] = (g as u32 * alpha as u32 / 255) as u8;
            pixel_data[offset + 2] = (r as u32 * alpha as u32 / 255) as u8;
        }
        
        // Set the modified pixels back
        let _ = SetDIBits(hdc, bitmap, 0, bm.bmHeight as u32, pixel_data.as_ptr() as *const c_void, bmi, DIB_RGB_COLORS);
    }
}