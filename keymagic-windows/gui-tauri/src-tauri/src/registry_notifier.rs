#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_A, GetAsyncKeyState, VK_MENU};
use anyhow::Result;
use std::thread;
use std::time::Duration;

// SendInput signature for registry reload notification
const KEYMAGIC_REGISTRY_RELOAD_SIGNATURE: usize = 0x4B4D5252; // "KMRR" in hex

#[cfg(target_os = "windows")]
fn send_notification_internal() -> Result<()> {
    unsafe {
        println!("[RegistryNotifier] Sending notification via SendInput");
        
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                    wVk: VK_A,
                    wScan: 0,
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: KEYMAGIC_REGISTRY_RELOAD_SIGNATURE,
                },
            },
        };
        
        // Send key down
        let result1 = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result1 == 0 {
            eprintln!("[RegistryNotifier] ERROR: Failed to send key down event");
            return Err(anyhow::anyhow!("SendInput failed for key down"));
        }
        
        // Send key up to maintain proper keyboard state
        input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
        let result2 = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result2 == 0 {
            eprintln!("[RegistryNotifier] ERROR: Failed to send key up event");
            return Err(anyhow::anyhow!("SendInput failed for key up"));
        }
        
        println!("[RegistryNotifier] Notification sent successfully");
        Ok(())
    }
}

pub struct RegistryNotifier;

impl RegistryNotifier {
    /// Notify all TSF instances to reload registry settings using SendInput
    pub fn notify_registry_changed() -> Result<()> {
        println!("[RegistryNotifier] Sending registry reload notification to TSF instances");
        
        #[cfg(target_os = "windows")]
        {
            unsafe {
                // Check if ALT key is currently pressed
                let alt_pressed = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
                
                if alt_pressed {
                    println!("[RegistryNotifier] ALT key is pressed, spawning delayed notification thread");
                    
                    // Spawn a thread to send the notification after ALT is released
                    thread::spawn(|| {
                        // Wait for ALT key to be released (max 2 seconds)
                        let mut wait_count = 0;
                        while wait_count < 40 {  // 40 * 50ms = 2 seconds max
                            thread::sleep(Duration::from_millis(50));
                            let alt_still_pressed = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
                            if !alt_still_pressed {
                                println!("[RegistryNotifier] ALT key released after {} ms", wait_count * 50);
                                break;
                            }
                            wait_count += 1;
                        }
                        
                        // Small additional delay to ensure system is ready
                        thread::sleep(Duration::from_millis(100));
                        
                        // Now send the notification
                        if let Err(e) = send_notification_internal() {
                            eprintln!("[RegistryNotifier] Failed to send delayed notification: {}", e);
                        }
                    });
                    
                    // Return immediately - the notification will be sent asynchronously
                    return Ok(());
                }
                
                // ALT not pressed, send immediately
                send_notification_internal()?;
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            println!("[RegistryNotifier] Registry notification skipped (not Windows)");
        }
        
        Ok(())
    }
}

// Helper function to notify after registry changes
pub fn save_settings_and_notify<F>(save_fn: F) -> Result<()>
where
    F: FnOnce() -> Result<()>
{
    println!("[RegistryNotifier] save_settings_and_notify: Starting registry update");
    
    // 1. Write to registry
    save_fn()?;
    println!("[RegistryNotifier] save_settings_and_notify: Registry write completed");
    
    // 2. Notify all TSF instances via SendInput
    RegistryNotifier::notify_registry_changed()?;
    println!("[RegistryNotifier] save_settings_and_notify: Notification sent");
    
    Ok(())
}