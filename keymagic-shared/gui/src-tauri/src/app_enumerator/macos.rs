use anyhow::Result;
use crate::commands::AppInfo;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSString};
use objc::{class, msg_send, sel, sel_impl};
use std::collections::HashSet;
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn get_running_apps() -> Result<Vec<AppInfo>> {
    let mut apps = Vec::new();
    let mut seen_identifiers = HashSet::new();
    
    unsafe {
        // Create autorelease pool
        let pool = NSAutoreleasePool::new(nil);
        
        // Get NSWorkspace shared instance
        let workspace_class = class!(NSWorkspace);
        let workspace: id = msg_send![workspace_class, sharedWorkspace];
        
        // Get running applications
        let running_apps: id = msg_send![workspace, runningApplications];
        let count: usize = msg_send![running_apps, count];
        
        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex: i];
            
            // Check if app has regular activation policy (user apps)
            let activation_policy: i64 = msg_send![app, activationPolicy];
            if activation_policy != 0 { // NSApplicationActivationPolicyRegular = 0
                continue;
            }
            
            // Get bundle identifier
            let bundle_id_nsstring: id = msg_send![app, bundleIdentifier];
            if bundle_id_nsstring == nil {
                continue;
            }
            
            let bundle_id = nsstring_to_string(bundle_id_nsstring);
            
            // Skip if we've already seen this bundle ID
            if seen_identifiers.contains(&bundle_id) {
                continue;
            }
            
            // Get localized name
            let name_nsstring: id = msg_send![app, localizedName];
            let display_name = if name_nsstring != nil {
                nsstring_to_string(name_nsstring)
            } else {
                bundle_id.split('.').last().unwrap_or("Unknown").to_string()
            };
            
            // Get icon
            let icon_base64 = get_app_icon_base64(app);
            
            apps.push(AppInfo {
                display_name,
                identifier: bundle_id.clone(),
                icon_base64,
                is_running: true,
            });
            
            seen_identifiers.insert(bundle_id);
        }
        
        pool.drain();
    }
    
    // Sort apps alphabetically by display name
    apps.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    
    Ok(apps)
}

unsafe fn nsstring_to_string(nsstring: id) -> String {
    let bytes: *const u8 = msg_send![nsstring, UTF8String];
    let len: usize = msg_send![nsstring, lengthOfBytesUsingEncoding: 4]; // NSUTF8StringEncoding = 4
    
    if bytes.is_null() {
        return String::new();
    }
    
    let slice = std::slice::from_raw_parts(bytes, len);
    std::str::from_utf8_unchecked(slice).to_string()
}

unsafe fn get_app_icon_base64(app: id) -> Option<String> {
    // Get the app icon
    let icon: id = msg_send![app, icon];
    if icon == nil {
        return None;
    }
    
    // Create a 32x32 representation
    let size = cocoa::foundation::NSSize { 
        width: 32.0, 
        height: 32.0 
    };
    
    // Get representations array
    let representations: id = msg_send![icon, representations];
    if representations == nil {
        return None;
    }
    
    // Try to find a bitmap representation
    let count: usize = msg_send![representations, count];
    for i in 0..count {
        let rep: id = msg_send![representations, objectAtIndex: i];
        
        // Check if it's NSBitmapImageRep
        let class_name: id = msg_send![rep, className];
        let class_str = nsstring_to_string(class_name);
        
        if class_str.contains("NSBitmapImageRep") {
            // Get PNG data
            let png_data: id = msg_send![rep, representationUsingType: 4 properties: nil]; // NSBitmapImageFileTypePNG = 4
            
            if png_data != nil {
                let length: usize = msg_send![png_data, length];
                let bytes: *const u8 = msg_send![png_data, bytes];
                
                if !bytes.is_null() && length > 0 {
                    let data = std::slice::from_raw_parts(bytes, length);
                    return Some(STANDARD.encode(data));
                }
            }
        }
    }
    
    // Alternative: Try to get TIFF representation and convert
    let tiff_data: id = msg_send![icon, TIFFRepresentation];
    if tiff_data != nil {
        // Create NSBitmapImageRep from TIFF data
        let bitmap_class = class!(NSBitmapImageRep);
        let bitmap: id = msg_send![bitmap_class, imageRepWithData: tiff_data];
        
        if bitmap != nil {
            // Convert to PNG
            let png_data: id = msg_send![bitmap, representationUsingType: 4 properties: nil];
            
            if png_data != nil {
                let length: usize = msg_send![png_data, length];
                let bytes: *const u8 = msg_send![png_data, bytes];
                
                if !bytes.is_null() && length > 0 {
                    let data = std::slice::from_raw_parts(bytes, length);
                    return Some(STANDARD.encode(data));
                }
            }
        }
    }
    
    None
}