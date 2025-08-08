use anyhow::{Context, Result};
use windows::core::PWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL, ERROR_SUCCESS};
use windows::Win32::Security::{
    AllocateAndInitializeSid, FreeSid,
    PSECURITY_DESCRIPTOR, PSID, ACL, ACE_FLAGS,
    DACL_SECURITY_INFORMATION,
    SECURITY_APP_PACKAGE_AUTHORITY, SECURITY_WORLD_SID_AUTHORITY,
    SECURITY_MANDATORY_LABEL_AUTHORITY,
    SID_IDENTIFIER_AUTHORITY,
    SUB_CONTAINERS_AND_OBJECTS_INHERIT,
};
use windows::Win32::Security::Authorization::{
    GetSecurityInfo, SetSecurityInfo,
    SetEntriesInAclW,
    SE_REGISTRY_KEY,
    EXPLICIT_ACCESS_W, TRUSTEE_W,
    SET_ACCESS,
    TRUSTEE_IS_SID, TRUSTEE_IS_GROUP,
};
use windows::Win32::System::SystemServices::{
    SECURITY_WORLD_RID,
    SECURITY_APP_PACKAGE_BASE_RID,
    SECURITY_BUILTIN_PACKAGE_ANY_PACKAGE,
    SECURITY_MANDATORY_LOW_RID,
    SECURITY_MANDATORY_UNTRUSTED_RID,
};
use windows::Win32::System::Registry::HKEY;
use winreg::RegKey;
use std::ptr::null_mut;

/// Add a specific permission to the registry key's existing DACL
fn add_permission_to_key(
    hkey: HKEY,
    sid_authority: &SID_IDENTIFIER_AUTHORITY,
    sid_sub_authorities: &[u32],
    access_mask: u32,
) -> Result<()> {
    unsafe {
        let mut psid = PSID::default();
        let mut old_dacl: *mut ACL = null_mut();
        let mut new_dacl: *mut ACL = null_mut();
        let mut psd = PSECURITY_DESCRIPTOR::default();
        
        // Create the SID
        let result = match sid_sub_authorities.len() {
            1 => AllocateAndInitializeSid(
                sid_authority,
                1,
                sid_sub_authorities[0],
                0, 0, 0, 0, 0, 0, 0,
                &mut psid,
            ),
            2 => AllocateAndInitializeSid(
                sid_authority,
                2,
                sid_sub_authorities[0],
                sid_sub_authorities[1],
                0, 0, 0, 0, 0, 0,
                &mut psid,
            ),
            _ => return Err(anyhow::anyhow!("Unsupported number of SID sub-authorities")),
        };
        
        if result.is_err() {
            return Err(anyhow::anyhow!("Failed to allocate SID"));
        }
        
        // Get the existing DACL
        let result = GetSecurityInfo(
            windows::Win32::Foundation::HANDLE(hkey.0 as *mut std::ffi::c_void),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut old_dacl),
            None,
            Some(&mut psd),
        );
        
        if result != ERROR_SUCCESS {
            FreeSid(psid);
            return Err(anyhow::anyhow!("Failed to get existing security info: {}", result.0));
        }
        
        // Create explicit access entry
        let ea = EXPLICIT_ACCESS_W {
            grfAccessPermissions: access_mask,
            grfAccessMode: SET_ACCESS,
            grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
            Trustee: TRUSTEE_W {
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_GROUP,
                ptstrName: PWSTR::from_raw(psid.0.cast()),
                ..Default::default()
            },
        };
        
        // Add the new entry to the existing DACL
        let result = SetEntriesInAclW(
            Some(&[ea]),
            Some(old_dacl),
            &mut new_dacl,
        );
        
        if result != ERROR_SUCCESS {
            FreeSid(psid);
            LocalFree(HLOCAL(psd.0));
            return Err(anyhow::anyhow!("Failed to set entries in ACL: {}", result.0));
        }
        
        // Apply the new DACL to the registry key
        let result = SetSecurityInfo(
            windows::Win32::Foundation::HANDLE(hkey.0 as *mut std::ffi::c_void),
            SE_REGISTRY_KEY,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(new_dacl),
            None,
        );
        
        // Clean up
        FreeSid(psid);
        LocalFree(HLOCAL(psd.0));
        if !new_dacl.is_null() {
            LocalFree(HLOCAL(new_dacl.cast()));
        }
        
        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("Failed to set security info: {}", result.0));
        }
    }
    
    Ok(())
}

/// Set permissions on a registry key to allow access from low integrity processes
/// This preserves existing permissions and adds new ones for broad compatibility
pub fn set_registry_permissions_for_low_integrity(key: &RegKey) -> Result<()> {
    // Get the raw HKEY handle from winreg
    let hkey = unsafe { std::mem::transmute::<isize, HKEY>(key.raw_handle()) };
    
    // Key Read access mask (KEY_READ = 0x20019)
    const KEY_READ: u32 = 0x20019;
    
    // Add permission for EVERYONE
    let world_sid_authority = SECURITY_WORLD_SID_AUTHORITY;
    add_permission_to_key(
        hkey,
        &world_sid_authority,
        &[SECURITY_WORLD_RID as u32],
        KEY_READ,
    ).context("Failed to add EVERYONE permission")?;
    
    // Add permission for ALL APPLICATION PACKAGES
    let app_package_authority = SECURITY_APP_PACKAGE_AUTHORITY;
    add_permission_to_key(
        hkey,
        &app_package_authority,
        &[SECURITY_APP_PACKAGE_BASE_RID as u32, SECURITY_BUILTIN_PACKAGE_ANY_PACKAGE as u32],
        KEY_READ,
    ).context("Failed to add ALL APPLICATION PACKAGES permission")?;
    
    // Add permission for Low Integrity Level (S-1-16-4096)
    let mandatory_label_authority = SECURITY_MANDATORY_LABEL_AUTHORITY;
    add_permission_to_key(
        hkey,
        &mandatory_label_authority,
        &[SECURITY_MANDATORY_LOW_RID as u32],
        KEY_READ,
    ).context("Failed to add Low Integrity permission")?;
    
    // Add permission for Untrusted Integrity Level (S-1-16-0)
    add_permission_to_key(
        hkey,
        &mandatory_label_authority,
        &[SECURITY_MANDATORY_UNTRUSTED_RID as u32],
        KEY_READ,
    ).context("Failed to add Untrusted Integrity permission")?;
    
    Ok(())
}

/// Apply low integrity permissions to all KeyMagic registry keys
pub fn apply_keymagic_registry_permissions() -> Result<()> {
    use winreg::enums::*;
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    // Set permissions on main KeyMagic key
    if let Ok(keymagic_key) = hkcu.open_subkey_with_flags(
        r"Software\KeyMagic",
        KEY_ALL_ACCESS
    ) {
        set_registry_permissions_for_low_integrity(&keymagic_key)
            .context("Failed to set permissions on KeyMagic root key")?;
        
        // Set permissions on Settings subkey
        if let Ok(settings_key) = keymagic_key.open_subkey_with_flags(
            "Settings",
            KEY_ALL_ACCESS
        ) {
            set_registry_permissions_for_low_integrity(&settings_key)
                .context("Failed to set permissions on Settings key")?;
        }
        
        // Set permissions on Keyboards subkey
        if let Ok(keyboards_key) = keymagic_key.open_subkey_with_flags(
            "Keyboards",
            KEY_ALL_ACCESS
        ) {
            set_registry_permissions_for_low_integrity(&keyboards_key)
                .context("Failed to set permissions on Keyboards key")?;
            
            // Set permissions on each keyboard entry
            for kb_name in keyboards_key.enum_keys().filter_map(Result::ok) {
                if let Ok(kb_key) = keyboards_key.open_subkey_with_flags(
                    &kb_name,
                    KEY_ALL_ACCESS
                ) {
                    let _ = set_registry_permissions_for_low_integrity(&kb_key);
                }
            }
        }
    }
    
    Ok(())
}