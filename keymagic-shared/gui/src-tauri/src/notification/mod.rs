use anyhow::Result;
use base64::Engine;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub struct NotificationManager {
    app_handle: AppHandle,
}

impl NotificationManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn show_keyboard_switch(
        &self,
        keyboard_name: &str,
        icon_data: Option<Vec<u8>>,
    ) -> Result<()> {
        // Use native Windows HUD if available
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = crate::hud_win32::show_keyboard_hud(&format!("Switched to {}", keyboard_name)) {
                log::warn!("Failed to show Windows HUD, falling back to Tauri HUD: {}", e);
                return self.show_hud(
                    &format!("Switched to {}", keyboard_name),
                    icon_data,
                    Some(1500),
                )
                .await;
            }
            return Ok(());
        }
        
        // Use Tauri HUD on other platforms
        #[cfg(not(target_os = "windows"))]
        {
            self.show_hud(
                &format!("Switched to {}", keyboard_name),
                icon_data,
                Some(1500),
            )
            .await
        }
    }

    pub async fn show_tray_notification(&self) -> Result<()> {
        // Use native Windows HUD if available
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = crate::hud_win32::show_tray_minimize_notification() {
                log::warn!("Failed to show Windows HUD, falling back to Tauri HUD: {}", e);
                return self.show_hud(
                    "KeyMagic is running in the system tray",
                    None,
                    Some(2000),
                )
                .await;
            }
            return Ok(());
        }
        
        // Use Tauri HUD on other platforms
        #[cfg(not(target_os = "windows"))]
        {
            self.show_hud(
                "KeyMagic is running in the system tray",
                None,
                Some(2000),
            )
            .await
        }
    }

    pub async fn show_error(&self, message: &str) -> Result<()> {
        self.show_hud(message, None, Some(3000)).await
    }

    async fn show_hud(
        &self,
        text: &str,
        icon_data: Option<Vec<u8>>,
        duration_ms: Option<u32>,
    ) -> Result<()> {
        // Get or create HUD window
        let window = match self.app_handle.get_webview_window("hud") {
            Some(w) => w,
            None => {
                // Get primary monitor info for positioning
                let monitor = self
                    .app_handle
                    .primary_monitor()?
                    .or_else(|| {
                        self.app_handle
                            .available_monitors()
                            .ok()
                            .and_then(|monitors| monitors.into_iter().next())
                    })
                    .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;

                let scale_factor = monitor.scale_factor();
                let size = monitor.size();
                let hud_width = 320.0;
                let hud_height = 80.0;

                // Position at top center with some margin
                let x = (size.width as f64 / scale_factor - hud_width) / 2.0;
                let y = 50.0; // 50px from top

                // Create the HUD window
                let mut window_builder = WebviewWindowBuilder::new(
                    &self.app_handle,
                    "hud",
                    WebviewUrl::App("hud.html".into()),
                );
                
                // Configure window
                window_builder = window_builder
                    .title("")
                    .decorations(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .resizable(false)
                    .inner_size(hud_width, hud_height)
                    .position(x, y)
                    .visible(false);
                
                // Build window
                window_builder.build()?
            }
        };

        // Prepare icon data as base64
        let icon_base64 = icon_data.map(|data| base64::engine::general_purpose::STANDARD.encode(&data));

        // Emit show event to the HUD window
        window.emit(
            "show-notification",
            json!({
                "text": text,
                "icon": icon_base64,
                "duration": duration_ms.unwrap_or(1500)
            }),
        )?;

        Ok(())
    }

    // Platform-specific fallback notifications
    #[cfg(target_os = "linux")]
    pub fn show_system_notification(&self, title: &str, body: &str) -> Result<()> {
        // Use notify-rust as fallback on Linux
        use notify_rust::Notification;
        
        Notification::new()
            .summary(title)
            .body(body)
            .timeout(notify_rust::Timeout::Milliseconds(2000))
            .show()?;
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn show_system_notification(&self, title: &str, body: &str) -> Result<()> {
        // On Windows, we could use native toast notifications
        // For now, just use the HUD
        futures::executor::block_on(self.show_hud(body, None, Some(2000)))
    }

    #[cfg(target_os = "macos")]
    pub fn show_system_notification(&self, title: &str, body: &str) -> Result<()> {
        // On macOS, we could use NSUserNotification
        // For now, just use the HUD
        futures::executor::block_on(self.show_hud(body, None, Some(2000)))
    }
}

// Settings for future customization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HudSettings {
    pub enabled: bool,
    pub duration_ms: u32,
    pub position: HudPosition,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HudPosition {
    TopCenter,
    TopRight,
    Center,
    BottomRight,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_ms: 1500,
            position: HudPosition::TopCenter,
        }
    }
}