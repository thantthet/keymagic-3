//
//  KMInputController.swift
//  KeyMagic
//
//  Input controller for KeyMagic IMK implementation
//

import InputMethodKit
import Foundation
import Carbon.HIToolbox

// MARK: - Logging Configuration

private let LOG_TAG = "KeyMagicEngine"

// Conditional logging for sensitive information
#if DEBUG
    // Debug build - show full information
    private func LOG_KEY_EVENT(_ keycode: UInt16, _ chars: String, _ modifiers: UInt) {
        NSLog("\(LOG_TAG): Processing key event - keycode=\(keycode), chars='\(chars)', modifiers=0x\(String(format: "%08X", modifiers))")
    }
    
    private func LOG_TEXT(_ label: String, _ text: String) {
        NSLog("\(LOG_TAG): \(label): \(text)")
    }
    
    private func LOG_DEBUG(_ message: String) {
        NSLog("\(LOG_TAG): \(message)")
    }
#else
    // Release build - redact sensitive key information
    private func LOG_KEY_EVENT(_ keycode: UInt16, _ chars: String, _ modifiers: UInt) {
        NSLog("\(LOG_TAG): Processing key event - [REDACTED]")
    }
    
    private func LOG_TEXT(_ label: String, _ text: String) {
        NSLog("\(LOG_TAG): \(label): [REDACTED]")
    }
    
    private func LOG_DEBUG(_ message: String) {
        NSLog("\(LOG_TAG): \(message)")
    }
#endif

class KMInputController: IMKInputController {
    private var engine: UnsafeMutablePointer<EngineHandle?>?
    private var currentKeyboardPath: String?
    private var currentKeyboardId: String?
    private var composingText: String = ""
    private var configObserver: NSObjectProtocol?
    
    // MARK: - Initialization
    
    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        
        // Create engine instance
        engine = keymagic_engine_new()
        
        // Load configuration and active keyboard
        loadActiveKeyboard()
        
        // Monitor configuration changes
        setupConfigurationObserver()
    }
    
    deinit {
        if let configObserver = configObserver {
            NotificationCenter.default.removeObserver(configObserver)
        }
        
        if let engine = engine {
            keymagic_engine_free(engine)
        }
    }
    
    // MARK: - Key Event Handling
    
    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard event.type == .keyDown else {
            return false
        }
        
        // Get keycode and modifiers
        let keycode = event.keyCode
        let modifiers = event.modifierFlags.rawValue
        
        // Get character string
        let chars = event.characters ?? ""
        
        // Log key event details
        LOG_KEY_EVENT(keycode, chars, modifiers)
        
        // Skip processing for Command key combinations
        if (modifiers & NSEvent.ModifierFlags.command.rawValue) != 0 {
            LOG_DEBUG("Skipping Command key combination")
            return false
        }
        
        // Check if we have a valid engine
        guard let engine = engine else {
            // No engine - eat printable keys
            if !chars.isEmpty,
               let ch = chars.first,
               ch.isASCII && ch.asciiValue! >= 0x21 && ch.asciiValue! <= 0x7E {
                LOG_DEBUG("No engine - eating printable key: '\(ch)'")
                return true // Eat the key
            }
            LOG_DEBUG("No engine - passing through non-printable key")
            return false
        }
        
        // Convert macOS keycode to VirtualKey
        guard let virtualKey = keycode.toVirtualKey else {
            LOG_DEBUG("Unknown keycode \(keycode) - cannot convert to VirtualKey")
            return false
        }
        
        #if DEBUG
        LOG_DEBUG("VirtualKey: \(virtualKey)")
        LOG_DEBUG("Modifiers - shift: \((modifiers & NSEvent.ModifierFlags.shift.rawValue) != 0), ctrl: \((modifiers & NSEvent.ModifierFlags.control.rawValue) != 0), alt: \((modifiers & NSEvent.ModifierFlags.option.rawValue) != 0), capsLock: \((modifiers & NSEvent.ModifierFlags.capsLock.rawValue) != 0)")
        #endif
        
        // Parse modifiers
        let shift = (modifiers & NSEvent.ModifierFlags.shift.rawValue) != 0 ? Int32(1) : Int32(0)
        let ctrl = (modifiers & NSEvent.ModifierFlags.control.rawValue) != 0 ? Int32(1) : Int32(0)
        let alt = (modifiers & NSEvent.ModifierFlags.option.rawValue) != 0 ? Int32(1) : Int32(0)
        let capsLock = (modifiers & NSEvent.ModifierFlags.capsLock.rawValue) != 0 ? Int32(1) : Int32(0)
        
        // Extract character
        let character: Int8 = {
            if ctrl == 0 && alt == 0 && !chars.isEmpty {
                let ch = chars.first!
                if ch.isASCII && ch.asciiValue! >= 0x20 && ch.asciiValue! <= 0x7E {
                    return Int8(ch.asciiValue!)
                }
            }
            return 0
        }()
        
        #if DEBUG
        LOG_DEBUG("Character: \(character) ('\(character > 0 ? String(UnicodeScalar(UInt8(character))) : "none")')")
        #endif
        
        // Prepare output structure
        var output = ProcessKeyOutput()
        
        // Process key
        LOG_DEBUG("Processing key with engine...")
        let result = keymagic_engine_process_key(
            engine,
            virtualKey.rawValue,
            character,
            shift,
            ctrl,
            alt,
            capsLock,
            &output
        )
        
        // Handle result
        if result == KeyMagicResult_Success {
            LOG_DEBUG("Engine process successful")
            LOG_DEBUG("Output - is_processed: \(output.is_processed)")
            
            // Log output details
            if let textPtr = output.text {
                let text = String(cString: textPtr)
                LOG_TEXT("Output text", text)
            }
            
            if let composingTextPtr = output.composing_text {
                let composingText = String(cString: composingTextPtr)
                LOG_TEXT("Output composing text", composingText)
            }
            
            // Handle output based on composing text
            if let composingTextPtr = output.composing_text, 
               String(cString: composingTextPtr).count > 0 {
                // Process output normally
                LOG_DEBUG("Processing output with composing text")
                processOutput(&output, keycode: keycode, client: sender)
            } else {
                // Engine has no composing text - clear preedit
                LOG_DEBUG("Engine has no composing text, clearing preedit")
                clearMarkedText(client: sender)
                composingText = ""
                
                // Reset engine for special keys
                switch Int(keycode) {
                case kVK_Return,
                     kVK_Tab,
                     kVK_Escape:
                    LOG_DEBUG("Resetting engine for special key")
                    keymagic_engine_reset(engine)
                default:
                    break
                }
            }
            
            // Free allocated strings
            if let text = output.text {
                keymagic_free_string(text)
            }
            if let composingText = output.composing_text {
                keymagic_free_string(composingText)
            }
            
            let processed = output.is_processed != 0
            LOG_DEBUG("Key processing complete - consumed=\(processed ? "TRUE" : "FALSE")")
            return processed
        }
        
        LOG_DEBUG("Engine process failed with result: \(result)")
        return false
    }
    
    // MARK: - Output Processing
    
    private func processOutput(_ output: inout ProcessKeyOutput, keycode: UInt16, client sender: Any!) {
        LOG_DEBUG("processOutput called")
        
        // Update composing text from engine
        if let composingTextPtr = output.composing_text {
            composingText = String(cString: composingTextPtr)
            LOG_TEXT("Updated composing text", composingText)
        }
        
        // Check if we should commit the composition
        let shouldCommitResult = shouldCommit(keycode: keycode, isProcessed: output.is_processed != 0, composingText: composingText)
        LOG_DEBUG("Should commit: \(shouldCommitResult)")
        
        if shouldCommitResult {
            if !composingText.isEmpty {
                LOG_DEBUG("Committing composition")
                
                // Update marked text with final composing text before committing
                updateMarkedText(composingText, client: sender)
                
                // Commit the composing text
                commitText(composingText, client: sender)
                
                // Reset engine after commit
                LOG_DEBUG("Resetting engine after commit")
                keymagic_engine_reset(engine)
                
                // For unprocessed space, commit space too
                if keycode == kVK_Space && output.is_processed == 0 {
                    LOG_DEBUG("Committing additional space for unprocessed space key")
                    commitText(" ", client: sender)
                }
            }
        } else {
            // Just update preedit display
            LOG_TEXT("Updating marked text with", composingText)
            updateMarkedText(composingText, client: sender)
        }
    }
    
    // MARK: - Composition Management
    
    private func shouldCommit(keycode: UInt16, isProcessed: Bool, composingText: String) -> Bool {
        // If engine didn't process the key, commit
        if !isProcessed {
            return true
        }
        
        // Check special keys that trigger commit
        switch Int(keycode) {
        case kVK_Space:
            // Commit if composing text ends with space
            if !composingText.isEmpty {
                return composingText.last == " "
            }
            return false
            
        case kVK_Return,
             kVK_Tab,
             kVK_Escape:
            // Always commit for these keys
            return true
            
        default:
            // Don't commit for other keys
            return false
        }
    }
    
    private func updateMarkedText(_ text: String, client sender: Any!) {
        guard let client = sender as? IMKTextInput & NSObjectProtocol else { return }
        
        let attributes: [NSAttributedString.Key: Any] = [
            .underlineStyle: NSUnderlineStyle.single.rawValue,
            .underlineColor: NSColor.systemBlue
        ]
        
        let markedText = NSAttributedString(string: text, attributes: attributes)
        // Use UTF-16 count for NSRange
        let utf16Count = text.utf16.count
        let selectionRange = NSRange(location: utf16Count, length: 0)
        let replacementRange = NSRange(location: NSNotFound, length: 0)
        
        client.setMarkedText(markedText, selectionRange: selectionRange, replacementRange: replacementRange)
        
        LOG_TEXT("Updated marked text", text)
        LOG_DEBUG("Cursor at \(utf16Count)")
    }
    
    private func commitText(_ text: String, client sender: Any!) {
        guard let client = sender as? IMKTextInput & NSObjectProtocol else { return }
        
        LOG_TEXT("Attempting to commit marked text", text)
        
        client.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
        clearMarkedText(client: sender)
        
        LOG_TEXT("Committed marked text", text)
    }
    
    private func clearMarkedText(client sender: Any!) {
        guard let client = sender as? IMKTextInput & NSObjectProtocol else { return }
        
        if !composingText.isEmpty {
            LOG_DEBUG("Cleared marked text")
        }
        client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: NSRange(location: NSNotFound, length: 0))
        composingText = ""
    }
    
    // MARK: - State Management
    
    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        
        LOG_DEBUG("Focus in")
        
        // Reset engine state
        if let engine = engine {
            keymagic_engine_reset(engine)
        }
        
        // Clear any existing composition
        clearMarkedText(client: sender)
    }
    
    override func deactivateServer(_ sender: Any!) {
        LOG_DEBUG("Focus out")
        
        // Commit any pending composition
        if !composingText.isEmpty {
            commitText(composingText, client: sender)
        }
        
        super.deactivateServer(sender)
    }
    
    override func commitComposition(_ sender: Any!) {
        LOG_DEBUG("Reset")
        if !composingText.isEmpty {
            commitText(composingText, client: sender)
        }
    }
    
    override func cancelComposition() {
        LOG_DEBUG("Cancel composition")
        if let client = client() {
            clearMarkedText(client: client)
        }
        
        if let engine = engine {
            keymagic_engine_reset(engine)
        }
    }
    
    // MARK: - Keyboard Management
    
    private func setupConfigurationObserver() {
        configObserver = NotificationCenter.default.addObserver(
            forName: NSNotification.Name("KMConfigurationChanged"),
            object: nil,
            queue: .main
        ) { [weak self] _ in
            LOG_DEBUG("Config file changed, reloading keyboard")
            self?.loadActiveKeyboard()
        }
        LOG_DEBUG("Config file monitoring enabled")
    }
    
    private func loadActiveKeyboard() {
        let config = KMConfiguration.shared
        
        // Get active keyboard ID from config
        guard let keyboardId = config.activeKeyboardId else {
            LOG_DEBUG("No active keyboard configured")
            return
        }
        
        // Skip if already loaded
        if keyboardId == currentKeyboardId {
            return
        }
        
        // Get keyboard file path
        guard let keyboardPath = config.getKeyboardPath(for: keyboardId) else {
            LOG_DEBUG("Keyboard file not found for ID: \(keyboardId)")
            return
        }
        
        // Load the keyboard
        if loadKeyboard(id: keyboardId, path: keyboardPath) {
            LOG_DEBUG("Successfully loaded keyboard: \(keyboardId) (\(keyboardPath))")
        } else {
            LOG_DEBUG("Failed to load keyboard: \(keyboardId)")
        }
    }
    
    func loadKeyboard(id: String, path: String) -> Bool {
        guard let engine = engine else { return false }
        
        let result = path.withCString { pathPtr in
            keymagic_engine_load_keyboard(engine, pathPtr)
        }
        
        if result == KeyMagicResult_Success {
            currentKeyboardPath = path
            currentKeyboardId = id
            return true
        }
        
        return false
    }
    
    func unloadKeyboard() {
        if let engine = engine {
            keymagic_engine_reset(engine)
        }
        currentKeyboardPath = nil
        currentKeyboardId = nil
        LOG_DEBUG("Keyboard unloaded")
    }
    
    // MARK: - Menu Support
    
    override func menu() -> NSMenu! {
        let menu = NSMenu(title: "KeyMagic")
        
        // Add keyboard selection items
        let keyboardsItem = NSMenuItem(title: "Keyboards", action: nil, keyEquivalent: "")
        let keyboardsSubmenu = NSMenu(title: "Keyboards")
        
        // Load available keyboards from config
        let config = KMConfiguration.shared
        let keyboards = config.installedKeyboards
        
        if keyboards.isEmpty {
            let noKeyboardsItem = NSMenuItem(title: "No Keyboards Installed", action: nil, keyEquivalent: "")
            noKeyboardsItem.isEnabled = false
            keyboardsSubmenu.addItem(noKeyboardsItem)
        } else {
            for keyboard in keyboards {
                guard let id = keyboard["id"],
                      let name = keyboard["name"] else { continue }
                
                let menuItem = NSMenuItem(title: name, action: #selector(selectKeyboard(_:)), keyEquivalent: "")
                menuItem.target = self
                menuItem.representedObject = id
                
                // Check current keyboard
                if id == currentKeyboardId {
                    menuItem.state = .on
                }
                
                keyboardsSubmenu.addItem(menuItem)
            }
        }
        
        keyboardsItem.submenu = keyboardsSubmenu
        menu.addItem(keyboardsItem)
        
        menu.addItem(NSMenuItem.separator())
        
        // Add preferences item
        let preferencesItem = NSMenuItem(title: "Preferences...", action: #selector(showKeyMagicPreferences), keyEquivalent: ",")
        preferencesItem.target = self
        menu.addItem(preferencesItem)
        
        return menu
    }
    
    @objc private func selectKeyboard(_ sender: NSMenuItem) {
        guard let keyboardId = sender.representedObject as? String else { return }
        
        // Update configuration
        let config = KMConfiguration.shared
        if let keyboardPath = config.getKeyboardPath(for: keyboardId) {
            if loadKeyboard(id: keyboardId, path: keyboardPath) {
                LOG_DEBUG("Switched to keyboard: \(keyboardId)")
                
                // TODO: Update config file to save the active keyboard
                // This would require implementing a method in KMConfiguration to save changes
            }
        }
    }
    
    @objc private func showKeyMagicPreferences() {
        // Launch the GUI application
        if #available(macOS 11.0, *) {
            if let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "net.keymagic.KeyMagic3") {
                let config = NSWorkspace.OpenConfiguration()
                NSWorkspace.shared.openApplication(at: url, configuration: config) { _, _ in }
            }
        } else {
            // Fallback for older macOS versions
            NSWorkspace.shared.launchApplication(withBundleIdentifier: "net.keymagic.KeyMagic3", 
                                               options: .default, 
                                               additionalEventParamDescriptor: nil, 
                                               launchIdentifier: nil)
        }
    }
}