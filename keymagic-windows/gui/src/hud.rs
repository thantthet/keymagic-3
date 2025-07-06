use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
        UI::HiDpi::*,
    },
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::ffi::c_void;

const HUD_CLASS_NAME: PCWSTR = w!("KeyMagicHUD");
const HUD_DISPLAY_DURATION: u64 = 800; // 0.8 seconds
const HUD_FADE_DURATION: u64 = 150; // 150ms fade
const HUD_INITIAL_OPACITY: u8 = 200; // Initial opacity (0-255)

// Constants for UpdateLayeredWindow
const ULW_ALPHA: UPDATE_LAYERED_WINDOW_FLAGS = UPDATE_LAYERED_WINDOW_FLAGS(2);
const AC_SRC_OVER: u32 = 0;
const AC_SRC_ALPHA: u32 = 1;

pub struct HudNotification {
    hwnd: HWND,
}

impl HudNotification {
    pub fn new() -> Result<Arc<Self>> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            
            // Register HUD window class
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: std::mem::size_of::<*const HudNotification>() as i32,
                hInstance: instance.into(),
                hIcon: HICON::default(),
                hCursor: HCURSOR::default(),
                hbrBackground: HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: HUD_CLASS_NAME,
                hIconSm: HICON::default(),
            };
            
            let atom = RegisterClassExW(&wc);
            if atom == 0 {
                return Err(Error::from_win32());
            }
            
            let hud = Arc::new(HudNotification {
                hwnd: HWND::default(),
            });
            
            let hud_ptr = Arc::as_ptr(&hud) as *const c_void;
            
            // Create HUD window
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                HUD_CLASS_NAME,
                PCWSTR::null(),
                WS_POPUP,
                0, 0, 0, 0,
                None,
                None,
                instance,
                Some(hud_ptr),
            );
            
            if hwnd.0 == 0 {
                return Err(Error::from_win32());
            }
            
            Ok(hud)
        }
    }
    
    pub fn show(&self, message: &str, enabled: bool) -> Result<()> {
        unsafe {
            // Get primary monitor info
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            
            let hmonitor = MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTOPRIMARY);
            GetMonitorInfoW(hmonitor, &mut mi);
            
            let screen_width = mi.rcWork.right - mi.rcWork.left;
            let screen_height = mi.rcWork.bottom - mi.rcWork.top;
            
            // Calculate HUD size based on DPI
            let dpi = GetDpiForWindow(self.hwnd);
            let hud_width = Self::scale_for_dpi(260, dpi);
            let hud_height = Self::scale_for_dpi(70, dpi);
            
            // Center the HUD on screen
            let x = mi.rcWork.left + (screen_width - hud_width) / 2;
            let y = mi.rcWork.top + (screen_height - hud_height) / 2;
            
            // Store the message and enabled state
            let msg_string = message.to_string();
            let hwnd = self.hwnd;
            
            // Clean up any previous message
            let msg_prop_name = w!("HudMessage");
            let old_msg_ptr = GetPropW(hwnd, msg_prop_name);
            if old_msg_ptr.0 != 0 {
                let _ = Box::from_raw(old_msg_ptr.0 as *mut String);
            }
            
            // Store as window properties instead
            let msg_prop_name = w!("HudMessage");
            let enabled_prop_name = w!("HudEnabled");
            SetPropW(hwnd, msg_prop_name, HANDLE(Box::into_raw(Box::new(msg_string)) as isize))?;
            SetPropW(hwnd, enabled_prop_name, HANDLE(if enabled { 1 } else { 0 }))?;
            
            // Position and show the window
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                x,
                y,
                hud_width,
                hud_height,
                SWP_SHOWWINDOW | SWP_NOACTIVATE,
            )?;
            
            // Force paint message to update layered window
            InvalidateRect(self.hwnd, None, false);
            UpdateWindow(self.hwnd);
            
            // Start fade out timer
            let hwnd = self.hwnd;
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(HUD_DISPLAY_DURATION));
                let _ = Self::fade_out(hwnd);
            });
        }
        
        Ok(())
    }
    
    fn fade_out(hwnd: HWND) -> Result<()> {
        unsafe {
            let steps = 8u64;
            let step_duration = HUD_FADE_DURATION / steps;
            
            for i in 0..steps {
                let alpha = HUD_INITIAL_OPACITY - ((HUD_INITIAL_OPACITY as u64 * i / steps) as u8);
                
                // Force repaint with updated alpha
                SetPropW(hwnd, w!("FadeAlpha"), HANDLE(alpha as isize))?;
                InvalidateRect(hwnd, None, false);
                UpdateWindow(hwnd);
                
                thread::sleep(Duration::from_millis(step_duration));
            }
            
            ShowWindow(hwnd, SW_HIDE);
            RemovePropW(hwnd, w!("FadeAlpha"));
        }
        
        Ok(())
    }
    
    fn scale_for_dpi(value: i32, dpi: u32) -> i32 {
        (value as f32 * dpi as f32 / 96.0) as i32
    }
    
    unsafe fn set_bitmap_alpha(hdc: HDC, bitmap: HBITMAP, transparent_color: COLORREF, text_color: COLORREF) {
        // Get bitmap info
        let mut bm = BITMAP::default();
        GetObjectW(bitmap, std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut _));
        
        // Create BITMAPINFO structure
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bm.bmWidth,
                biHeight: -bm.bmHeight, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default(); 1],
        };
        
        // Allocate buffer for pixel data
        let pixel_count = (bm.bmWidth * bm.bmHeight) as usize;
        let mut pixels = vec![0u8; pixel_count * 4];
        
        // Get bitmap bits
        GetDIBits(hdc, bitmap, 0, bm.bmHeight as u32, Some(pixels.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
        
        // Process alpha channel
        for i in 0..pixel_count {
            let offset = i * 4;
            let b = pixels[offset];
            let g = pixels[offset + 1];
            let r = pixels[offset + 2];
            let color = COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16));
            
            if color == transparent_color {
                // Transparent pixels
                pixels[offset + 3] = 0;
            } else if color == text_color {
                // Text pixels - fully opaque
                pixels[offset + 3] = 255;
            } else {
                // Background pixels - 80% opaque
                pixels[offset + 3] = (255.0 * 0.8) as u8;
            }
            
            // Premultiply alpha
            pixels[offset] = ((pixels[offset] as u32 * pixels[offset + 3] as u32) / 255) as u8;
            pixels[offset + 1] = ((pixels[offset + 1] as u32 * pixels[offset + 3] as u32) / 255) as u8;
            pixels[offset + 2] = ((pixels[offset + 2] as u32 * pixels[offset + 3] as u32) / 255) as u8;
        }
        
        // Set the modified bits back
        SetDIBits(hdc, bitmap, 0, bm.bmHeight as u32, pixels.as_ptr() as *const _, &bmi, DIB_RGB_COLORS);
    }
    
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                let hud_ptr = (*create_struct).lpCreateParams as *const HudNotification;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, hud_ptr as isize);
                
                if !hud_ptr.is_null() {
                    let _hud = &*hud_ptr;
                    let hud_mut = hud_ptr as *mut HudNotification;
                    (*hud_mut).hwnd = hwnd;
                }
                
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                
                if hdc.0 != 0 {
                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect);
                    
                    // Create memory DC
                    let screen_dc = GetDC(None);
                    let mem_dc = CreateCompatibleDC(screen_dc);
                    
                    // Create bitmap
                    let bitmap = CreateCompatibleBitmap(screen_dc, rect.right, rect.bottom);
                    let old_bitmap = SelectObject(mem_dc, bitmap);
                    
                    // Define colors
                    let transparent_color = COLORREF(0xFF0000); // Red for transparency
                    let bg_color = COLORREF(0x000000); // Black background
                    let text_color = COLORREF(0xFFFFFF); // White text
                    
                    // Fill with transparent color first
                    let transparent_brush = CreateSolidBrush(transparent_color);
                    FillRect(mem_dc, &rect, transparent_brush);
                    DeleteObject(transparent_brush);
                    
                    // Draw rounded rectangle with black background
                    let corner_radius = Self::scale_for_dpi(15, GetDpiForWindow(hwnd));
                    let bg_brush = CreateSolidBrush(bg_color);
                    let bg_pen = CreatePen(PS_SOLID, 0, bg_color);
                    let old_brush = SelectObject(mem_dc, bg_brush);
                    let old_pen = SelectObject(mem_dc, bg_pen);
                    
                    RoundRect(mem_dc, rect.left, rect.top, rect.right, rect.bottom, corner_radius, corner_radius);
                    
                    SelectObject(mem_dc, old_brush);
                    SelectObject(mem_dc, old_pen);
                    DeleteObject(bg_brush);
                    DeleteObject(bg_pen);
                    
                    // Get message and enabled state from window properties
                    let msg_prop_name = w!("HudMessage");
                    let enabled_prop_name = w!("HudEnabled");
                    let msg_ptr = GetPropW(hwnd, msg_prop_name);
                    let enabled = GetPropW(hwnd, enabled_prop_name).0 != 0;
                    
                    if msg_ptr.0 != 0 {
                        let msg_box = Box::from_raw(msg_ptr.0 as *mut String);
                        let message = msg_box.as_ref().clone();
                        // Don't drop the box, we still need it
                        Box::into_raw(msg_box);
                        // Draw icon (circle with checkmark or X)
                        let icon_size = Self::scale_for_dpi(32, GetDpiForWindow(hwnd));
                        let icon_x = rect.left + Self::scale_for_dpi(20, GetDpiForWindow(hwnd));
                        let icon_y = (rect.bottom - icon_size) / 2;
                        
                        // Draw circle
                        let icon_brush = CreateSolidBrush(COLORREF(if enabled { 0x4CAF50 } else { 0xF44336 }));
                        let old_brush = SelectObject(mem_dc, icon_brush);
                        Ellipse(mem_dc, icon_x, icon_y, icon_x + icon_size, icon_y + icon_size);
                        SelectObject(mem_dc, old_brush);
                        DeleteObject(icon_brush);
                        
                        // Draw checkmark or X
                        let icon_pen = CreatePen(PS_SOLID, 3, COLORREF(0xFFFFFF));
                        let old_pen = SelectObject(mem_dc, icon_pen);
                        
                        if enabled {
                            // Draw checkmark
                            MoveToEx(mem_dc, icon_x + icon_size / 4, icon_y + icon_size / 2, None);
                            LineTo(mem_dc, icon_x + icon_size / 2 - 2, icon_y + 3 * icon_size / 4 - 2);
                            LineTo(mem_dc, icon_x + 3 * icon_size / 4, icon_y + icon_size / 4);
                        } else {
                            // Draw X
                            MoveToEx(mem_dc, icon_x + icon_size / 4, icon_y + icon_size / 4, None);
                            LineTo(mem_dc, icon_x + 3 * icon_size / 4, icon_y + 3 * icon_size / 4);
                            MoveToEx(mem_dc, icon_x + 3 * icon_size / 4, icon_y + icon_size / 4, None);
                            LineTo(mem_dc, icon_x + icon_size / 4, icon_y + 3 * icon_size / 4);
                        }
                        
                        SelectObject(mem_dc, old_pen);
                        DeleteObject(icon_pen);
                        
                        // Draw text
                        let text_rect = RECT {
                            left: icon_x + icon_size + Self::scale_for_dpi(15, GetDpiForWindow(hwnd)),
                            top: rect.top,
                            right: rect.right - Self::scale_for_dpi(20, GetDpiForWindow(hwnd)),
                            bottom: rect.bottom,
                        };
                        
                        SetTextColor(mem_dc, COLORREF(0xFFFFFF)); // White text
                        SetBkMode(mem_dc, TRANSPARENT);
                        
                        // Create font
                        let font_height = Self::scale_for_dpi(18, GetDpiForWindow(hwnd));
                        let font = CreateFontW(
                            font_height,
                            0, 0, 0,
                            FW_NORMAL.0 as i32,
                            FALSE.0 as u32,
                            FALSE.0 as u32,
                            FALSE.0 as u32,
                            DEFAULT_CHARSET.0 as u32,
                            OUT_DEFAULT_PRECIS.0 as u32,
                            CLIP_DEFAULT_PRECIS.0 as u32,
                            CLEARTYPE_QUALITY.0 as u32,
                            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                            w!("Segoe UI"),
                        );
                        
                        let old_font = SelectObject(mem_dc, font);
                        
                        let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
                        
                        let mut msg_slice: Vec<u16> = msg_wide[..msg_wide.len()-1].to_vec();
                        DrawTextW(
                            mem_dc,
                            &mut msg_slice,
                            &text_rect as *const _ as *mut _,
                            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                        );
                        
                        SelectObject(mem_dc, old_font);
                        DeleteObject(font);
                    }
                    
                    // Now set alpha values manually like the original KeyMagic
                    Self::set_bitmap_alpha(mem_dc, bitmap, transparent_color, text_color);
                    
                    // Get fade alpha if we're fading out
                    let fade_alpha_prop = GetPropW(hwnd, w!("FadeAlpha"));
                    let fade_alpha = if fade_alpha_prop.0 != 0 {
                        fade_alpha_prop.0 as u8
                    } else {
                        HUD_INITIAL_OPACITY
                    };
                    
                    // Use UpdateLayeredWindow for proper alpha blending
                    let blend = BLENDFUNCTION {
                        BlendOp: AC_SRC_OVER as u8,
                        BlendFlags: 0,
                        SourceConstantAlpha: fade_alpha,
                        AlphaFormat: AC_SRC_ALPHA as u8,
                    };
                    
                    let window_pos = POINT { x: 0, y: 0 };
                    let size = SIZE { cx: rect.right, cy: rect.bottom };
                    
                    UpdateLayeredWindow(
                        hwnd,
                        None,
                        None,
                        Some(&size),
                        mem_dc,
                        Some(&window_pos),
                        transparent_color,  // Use transparent color for color key
                        Some(&blend),
                        UPDATE_LAYERED_WINDOW_FLAGS(ULW_ALPHA.0 | 1), // ULW_ALPHA | ULW_COLORKEY
                    );
                    
                    // Cleanup
                    SelectObject(mem_dc, old_bitmap);
                    DeleteObject(bitmap);
                    DeleteDC(mem_dc);
                    ReleaseDC(None, screen_dc);
                    
                    EndPaint(hwnd, &ps);
                }
                
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1), // Prevent flicker
            WM_DESTROY => {
                // Clean up the allocated string
                let msg_prop_name = w!("HudMessage");
                let msg_ptr = GetPropW(hwnd, msg_prop_name);
                if msg_ptr.0 != 0 {
                    let _ = Box::from_raw(msg_ptr.0 as *mut String);
                }
                RemovePropW(hwnd, msg_prop_name);
                RemovePropW(hwnd, w!("HudEnabled"));
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for HudNotification {
    fn drop(&mut self) {
        unsafe {
            if self.hwnd.0 != 0 {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}