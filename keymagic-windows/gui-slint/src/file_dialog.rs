use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::Shell::Common::*,
        UI::Shell::*,
        System::Com::*,
    },
};
use std::path::PathBuf;

pub fn show_open_file_dialog(hwnd: Option<HWND>) -> Option<PathBuf> {
    unsafe {
        // Initialize COM
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        
        // Create the file dialog
        let dialog: IFileOpenDialog = match CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL) {
            Ok(d) => d,
            Err(_) => return None,
        };
        
        // Set file type filter
        let file_types = [
            COMDLG_FILTERSPEC {
                pszName: w!("KeyMagic Keyboard Files (*.km2)"),
                pszSpec: w!("*.km2"),
            },
            COMDLG_FILTERSPEC {
                pszName: w!("All Files (*.*)"),
                pszSpec: w!("*.*"),
            },
        ];
        
        let _ = dialog.SetFileTypes(&file_types);
        let _ = dialog.SetFileTypeIndex(1);
        let _ = dialog.SetTitle(w!("Select KeyMagic Keyboard"));
        
        // Set options
        if let Ok(mut options) = dialog.GetOptions() {
            options |= FOS_FILEMUSTEXIST | FOS_PATHMUSTEXIST;
            let _ = dialog.SetOptions(options);
        }
        
        // Show the dialog
        let parent_hwnd = hwnd.unwrap_or(HWND::default());
        if dialog.Show(parent_hwnd).is_err() {
            return None;
        }
        
        // Get the result
        let item = match dialog.GetResult() {
            Ok(item) => item,
            Err(_) => return None,
        };
        
        let path = match item.GetDisplayName(SIGDN_FILESYSPATH) {
            Ok(path) => path,
            Err(_) => return None,
        };
        
        let path_string = path.to_string().ok()?;
        CoTaskMemFree(Some(path.0 as _));
        
        Some(PathBuf::from(path_string))
    }
}