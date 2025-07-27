# macOS KeyMagic Distribution Strategy

## Recommended Approach: Separate GUI App + IMK Bundle

Based on common macOS input method patterns and technical constraints, the recommended distribution strategy is:

### 1. **Two Separate Components**

```
KeyMagic.app (GUI Application)
├── Contents/
│   ├── MacOS/
│   │   └── KeyMagic (main GUI executable)
│   ├── Resources/
│   │   ├── KeyMagicInputMethod.app (embedded IMK bundle)
│   │   ├── keyboards/ (bundled .km2 files)
│   │   └── icon.icns
│   └── Info.plist

KeyMagicInputMethod.app (IMK Bundle - installed separately)
├── Contents/
│   ├── MacOS/
│   │   └── KeyMagicInputMethod (IMK server)
│   ├── Resources/
│   │   └── icon.icns
│   └── Info.plist (with IMK-specific keys)
```

### 2. **Why Separate Components?**

**Technical Reasons:**
- IMK bundles must have `LSBackgroundOnly = YES` (no UI)
- GUI apps need `LSBackgroundOnly = NO` (has UI)
- Different bundle ID requirements (IMK needs `.inputmethod`)
- Different lifecycle management (IMK runs on-demand)

**User Experience:**
- GUI app for configuration, keyboard management
- IMK bundle stays minimal for performance
- Clean separation of concerns

### 3. **Installation Flow**

```objc
// In GUI app's AppDelegate or installer
- (void)installInputMethod {
    NSString *sourcePath = [[NSBundle mainBundle] pathForResource:@"KeyMagicInputMethod" 
                                                            ofType:@"app"];
    NSString *destPath = [@"~/Library/Input Methods/KeyMagicInputMethod.app" 
                           stringByExpandingTildeInPath];
    
    // Copy IMK bundle to Input Methods folder
    [[NSFileManager defaultManager] copyItemAtPath:sourcePath 
                                             toPath:destPath 
                                              error:nil];
    
    // Register with system
    NSString *registerCmd = [NSString stringWithFormat:
        @"/System/Library/Frameworks/Carbon.framework/Versions/A/Support/TISRegisterInputSource '%@'", 
        destPath];
    system([registerCmd UTF8String]);
    
    // Enable the input method
    [self enableInputMethod];
}
```

### 4. **Communication Between Components**

**Shared Preferences:**
```objc
// Use app group for sharing data
NSUserDefaults *sharedDefaults = [[NSUserDefaults alloc] 
    initWithSuiteName:@"group.com.keymagic.shared"];

// GUI app writes:
[sharedDefaults setObject:keyboardPaths forKey:@"keyboards"];
[sharedDefaults setObject:processConfigs forKey:@"processInputModes"];

// IMK reads:
NSArray *keyboards = [sharedDefaults arrayForKey:@"keyboards"];
```

**File-Based Configuration:**
```
~/Library/Application Support/KeyMagic/
├── config.json          # Main configuration
├── keyboards/          # User keyboards
│   ├── myanmar-unicode.km2
│   └── shan.km2
└── process-modes.json  # Process-specific settings
```

### 5. **Distribution Package Structure**

**Option A: DMG with Installer**
```
KeyMagic.dmg
├── KeyMagic.app
├── Install KeyMagic (script/app)
└── README.txt
```

**Option B: PKG Installer**
```xml
<!-- distribution.xml -->
<installer-gui-script minSpecVersion="2.0">
    <title>KeyMagic</title>
    <choices-outline>
        <line choice="app"/>
        <line choice="imk"/>
    </choices-outline>
    <choice id="app" title="KeyMagic Application">
        <pkg-ref id="com.keymagic.app"/>
    </choice>
    <choice id="imk" title="KeyMagic Input Method">
        <pkg-ref id="com.keymagic.inputmethod"/>
    </choice>
</installer-gui-script>
```

### 6. **GUI App Responsibilities**

```objc
// KeyMagicApp features:
- (void)applicationDidFinishLaunching {
    // Check if IMK is installed
    if (![self isInputMethodInstalled]) {
        [self showInstallPrompt];
    }
    
    // Set up menu bar item
    [self setupStatusItem];
    
    // Load keyboard manager
    [self setupKeyboardManager];
}

// Main features:
- Keyboard installation/management
- Process input mode configuration
- IMK installation/updates
- Preferences window
- Menu bar quick access
```

### 7. **Bundle Identifiers**

```
GUI App: com.keymagic.app
IMK Bundle: com.keymagic.inputmethod
App Group: group.com.keymagic.shared
```

### 8. **Code Signing & Notarization**

```bash
# Sign both components
codesign --deep --force --verify --verbose \
    --sign "Developer ID Application: Your Name" \
    --options runtime \
    --entitlements KeyMagic.entitlements \
    KeyMagic.app

codesign --deep --force --verify --verbose \
    --sign "Developer ID Application: Your Name" \
    --options runtime \
    KeyMagicInputMethod.app

# Notarize the DMG/PKG
xcrun altool --notarize-app \
    --primary-bundle-id "com.keymagic.installer" \
    --file KeyMagic.dmg
```

## Alternative: Single App Bundle (Not Recommended)

While technically possible to embed everything in one bundle, this approach has limitations:

```
KeyMagic.app
├── Contents/
│   ├── MacOS/
│   │   ├── KeyMagic (launcher)
│   │   └── KeyMagicIMK (IMK server)
│   ├── Resources/
│   └── Info.plist (complex configuration)
```

**Problems:**
- Complex Info.plist configuration
- Difficult to update components independently
- Confusing lifecycle management
- May cause issues with system integration

## Examples from Other IMEs

1. **Google Japanese IME**: Separate GoogleJapaneseInput.app (GUI) and GoogleJapaneseInputMethod.app (IMK)
2. **Squirrel (RIME)**: Separate Squirrel.app (config) and SquirrelInputMethod.app (IMK)
3. **vChewing**: Single bundle but with complex internal structure

## Recommendation

Use the **separate GUI app + IMK bundle** approach because:
- Cleaner architecture
- Easier to maintain and update
- Better user experience
- Follows macOS input method conventions
- Allows independent component updates