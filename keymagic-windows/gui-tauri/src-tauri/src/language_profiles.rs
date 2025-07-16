use anyhow::{anyhow, Result};

#[cfg(target_os = "windows")]
use windows::{
    core::{GUID, HSTRING},
    Win32::{
        Foundation::HMODULE,
        System::Com::{CoInitializeEx, CoCreateInstance, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED},
        UI::TextServices::{ITfInputProcessorProfiles, CLSID_TF_InputProcessorProfiles, TF_LANGUAGEPROFILE},
    },
};

// KeyMagic TSF GUIDs - must match the values in TSF C++ code
#[cfg(target_os = "windows")]
const CLSID_KEYMAGIC_TEXT_SERVICE: GUID = GUID::from_u128(0x094a562b_d08b_4caf_8e95_8f8031cfd24c);
#[cfg(target_os = "windows")]
const GUID_KEYMAGIC_PROFILE: GUID = GUID::from_u128(0x87654321_4321_4321_4321_cba987654321);

// Text service description
const TEXTSERVICE_DESC: &str = "KeyMagic";

// Icon resource ID (negative value for resource)
const IDI_KEYMAGIC: i32 = 101;

/// Updates TSF language profiles based on the enabled languages in the registry
#[cfg(target_os = "windows")]
pub fn update_language_profiles(enabled_languages: &[String]) -> Result<()> {
    unsafe {
        // Initialize COM
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            return Err(anyhow!("Failed to initialize COM: {:?}", hr));
        }
        
        // Create ITfInputProcessorProfiles instance
        let profiles: ITfInputProcessorProfiles = CoCreateInstance(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ).map_err(|e| anyhow!("Failed to create ITfInputProcessorProfiles: {:?}", e))?;
        
        // Get module handle for icon
        let h_module = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;
        let module_path = get_module_path(h_module)?;
        
        // Convert enabled languages to LANGIDs
        let mut new_langids = Vec::new();
        for lang in enabled_languages {
            if let Some(langid) = language_code_to_langid(lang) {
                new_langids.push(langid);
            }
        }
        
        // If no languages specified, use English as default
        if new_langids.is_empty() {
            new_langids.push(0x0409); // English (United States)
        }
        
        // Get currently registered language profiles
        let current_langids = get_registered_language_profiles(&profiles)?;
        
        // Add new languages that aren't currently registered
        for langid in &new_langids {
            if !current_langids.contains(langid) {
                add_language_profile(&profiles, *langid, &module_path)?;
            }
        }
        
        // Remove languages that are no longer in the enabled list
        for langid in &current_langids {
            if !new_langids.contains(langid) {
                remove_language_profile(&profiles, *langid)?;
            }
        }
        
        // Uninitialize COM
        CoUninitialize();
        
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn get_module_path(h_module: HMODULE) -> Result<String> {
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
    
    unsafe {
        let mut buffer = vec![0u16; 260]; // MAX_PATH
        let len = GetModuleFileNameW(h_module, &mut buffer);
        
        if len == 0 {
            return Err(anyhow!("Failed to get module path"));
        }
        
        buffer.truncate(len as usize);
        Ok(String::from_utf16_lossy(&buffer))
    }
}

#[cfg(target_os = "windows")]
fn get_registered_language_profiles(profiles: &ITfInputProcessorProfiles) -> Result<Vec<u16>> {
    use windows::Win32::UI::TextServices::IEnumTfLanguageProfiles;
    
    let mut registered_langids = Vec::new();
    
    // Check known possible languages
    let possible_langids = vec![
        0x0409, // English (United States)
        0x0455, // Myanmar
        0x041E, // Thai
        0x0453, // Khmer (Cambodia)
        0x0454, // Lao
        0x042A, // Vietnamese
        0x0804, // Chinese (Simplified)
        0x0404, // Chinese (Traditional)
        0x0411, // Japanese
        0x0412, // Korean
    ];
    
    unsafe {
        for langid in possible_langids {
            let enum_profiles: IEnumTfLanguageProfiles = profiles.EnumLanguageProfiles(langid)?;
            
            loop {
                let mut profiles_array: [TF_LANGUAGEPROFILE; 1] = std::mem::zeroed();
                let mut fetched = 0u32;
                
                if enum_profiles.Next(&mut profiles_array, &mut fetched).is_err() || fetched == 0 {
                    break;
                }
                
                let profile = &profiles_array[0];
                if profile.clsid == CLSID_KEYMAGIC_TEXT_SERVICE && 
                   profile.guidProfile == GUID_KEYMAGIC_PROFILE {
                    registered_langids.push(langid);
                    break;
                }
            }
        }
    }
    
    Ok(registered_langids)
}

#[cfg(target_os = "windows")]
fn add_language_profile(profiles: &ITfInputProcessorProfiles, langid: u16, module_path: &str) -> Result<()> {
    unsafe {
        let desc = HSTRING::from(TEXTSERVICE_DESC);
        let icon_path = HSTRING::from(module_path);
        
        profiles.AddLanguageProfile(
            &CLSID_KEYMAGIC_TEXT_SERVICE,
            langid,
            &GUID_KEYMAGIC_PROFILE,
            desc.as_wide(),
            icon_path.as_wide(),
            -IDI_KEYMAGIC as u32, // Negative for resource ID
        )?;
        
        // Enable the profile
        profiles.EnableLanguageProfile(
            &CLSID_KEYMAGIC_TEXT_SERVICE,
            langid,
            &GUID_KEYMAGIC_PROFILE,
            true,
        )?;
        
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn remove_language_profile(profiles: &ITfInputProcessorProfiles, langid: u16) -> Result<()> {
    unsafe {
        profiles.RemoveLanguageProfile(
            &CLSID_KEYMAGIC_TEXT_SERVICE,
            langid,
            &GUID_KEYMAGIC_PROFILE,
        )?;
        
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn language_code_to_langid(language_code: &str) -> Option<u16> {
    match language_code {
        "en-US" => Some(0x0409), // English (United States)
        "my-MM" => Some(0x0455), // Myanmar
        "th-TH" => Some(0x041E), // Thai
        "km-KH" => Some(0x0453), // Khmer (Cambodia)
        "lo-LA" => Some(0x0454), // Lao
        "vi-VN" => Some(0x042A), // Vietnamese
        "zh-CN" => Some(0x0804), // Chinese (Simplified)
        "zh-TW" => Some(0x0404), // Chinese (Traditional)
        "ja-JP" => Some(0x0411), // Japanese
        "ko-KR" => Some(0x0412), // Korean
        _ => None,
    }
}

// Non-Windows stub
#[cfg(not(target_os = "windows"))]
pub fn update_language_profiles(_enabled_languages: &[String]) -> Result<()> {
    Ok(())
}