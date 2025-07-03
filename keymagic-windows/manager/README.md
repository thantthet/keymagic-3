# KeyMagic Keyboard Manager

A Windows GUI application for managing KeyMagic keyboard layouts.

## Features

- Add/remove keyboard layouts (.km2 files)
- Activate keyboards for use with the TSF
- Keyboard layouts are stored in Windows registry
- Simple and intuitive interface

## Building

1. Open Visual Studio Developer Command Prompt
2. Navigate to the manager directory
3. Run: `build_manager.bat`

## Usage

1. Run `KeyMagicManager.exe`
2. Click "Add..." to browse and add .km2 keyboard files
3. Select a keyboard and click "Activate" to make it active
4. The active keyboard will be loaded automatically when the TSF starts

## Registry Storage

Keyboards are stored in: `HKEY_CURRENT_USER\Software\KeyMagic\Keyboards`

- Each keyboard is stored as a registry value with the name as key and file path as value
- The active keyboard index is stored in the `ActiveKeyboard` DWORD value

## Integration with TSF

When the KeyMagic TSF DLL starts, it reads the active keyboard from the registry and loads it automatically. This allows users to manage keyboards without manually editing configuration files.

## Screenshot

```
+--------------------------------------------------+
| KeyMagic Keyboard Manager                    [X] |
+--------------------------------------------------+
| Name         | Description           | Status    |
|--------------|----------------------|-----------|
| Myanmar3     | KeyMagic Keyboard... | Active    |
| ZawGyi       | KeyMagic Keyboard... |           |
| Shan         | KeyMagic Keyboard... |           |
|              |                      |           |
|              |                      | [Add...]  |
|              |                      | [Remove]  |
|              |                      | [Activate]|
|              |                      | [Settings]|
+--------------------------------------------------+
| Activated: Myanmar3                              |
+--------------------------------------------------+
```