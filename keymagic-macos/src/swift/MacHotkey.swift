//
//  MacHotkey.swift
//  KeyMagic
//
//  Represents a parsed hotkey for macOS
//

import Foundation
import AppKit
import Carbon.HIToolbox

/// Represents a parsed hotkey for macOS
struct MacHotkey {
    let keyEquivalent: String
    let modifierMask: NSEvent.ModifierFlags
    
    /// Original hotkey string (e.g., "CTRL+SHIFT+K")
    let originalString: String?
    
    /// Virtual key code from keymagic-core
    let virtualKeyCode: Int32
    
    /// Individual modifier flags
    let hasCtrl: Bool
    let hasAlt: Bool
    let hasShift: Bool
    let hasMeta: Bool
    
    /// Initialize from HotkeyInfo and optional original string
    init?(from info: HotkeyInfo, originalString: String? = nil) {
        // Convert VirtualKey to macOS key equivalent
        guard let keyEquiv = MacHotkey.virtualKeyToKeyEquivalent(info.key_code) else {
            return nil
        }
        
        // Build modifier flags
        var modifiers = NSEvent.ModifierFlags()
        if info.ctrl != 0 { modifiers.insert(.control) }
        if info.alt != 0 { modifiers.insert(.option) }
        if info.shift != 0 { modifiers.insert(.shift) }
        if info.meta != 0 { modifiers.insert(.command) }
        
        self.keyEquivalent = keyEquiv
        self.modifierMask = modifiers
        self.originalString = originalString
        self.virtualKeyCode = info.key_code
        self.hasCtrl = info.ctrl != 0
        self.hasAlt = info.alt != 0
        self.hasShift = info.shift != 0
        self.hasMeta = info.meta != 0
    }
    
    /// Parse a hotkey string like "CTRL+SHIFT+K" using the keymagic-core FFI
    static func parse(_ hotkeyString: String) -> MacHotkey? {
        LOG_DEBUG("Parsing hotkey: \(hotkeyString)")
        
        var info = HotkeyInfo()
        let result = hotkeyString.withCString { hotkeyStr in
            keymagic_parse_hotkey(hotkeyStr, &info)
        }
        
        guard result == 1 else {
            LOG_DEBUG("Failed to parse hotkey: \(hotkeyString)")
            return nil
        }
        
        guard let hotkey = MacHotkey(from: info, originalString: hotkeyString) else {
            LOG_DEBUG("Unknown virtual key code: \(info.key_code)")
            return nil
        }
        
        LOG_DEBUG("Parsed hotkey: \(hotkey.debugDescription)")
        return hotkey
    }
    
    /// Apply this hotkey to a menu item
    func applyTo(_ menuItem: NSMenuItem) {
        menuItem.keyEquivalent = keyEquivalent
        menuItem.keyEquivalentModifierMask = modifierMask
    }
    
    /// Debug description
    var debugDescription: String {
        if let original = originalString {
            return original
        }
        
        var parts: [String] = []
        if hasMeta { parts.append("CMD") }
        if hasCtrl { parts.append("CTRL") }
        if hasAlt { parts.append("ALT") }
        if hasShift { parts.append("SHIFT") }
        
        // Format key for display
        let displayKey: String
        switch keyEquivalent {
        case " ": displayKey = "SPACE"
        case "\r": displayKey = "ENTER"
        case "\t": displayKey = "TAB"
        case "\u{08}": displayKey = "BACKSPACE"
        case "\u{1B}": displayKey = "ESC"
        case "\u{7F}": displayKey = "DELETE"
        default:
            if keyEquivalent.count == 1 {
                displayKey = keyEquivalent.uppercased()
            } else {
                displayKey = "KEY(\(virtualKeyCode))"
            }
        }
        
        parts.append(displayKey)
        return parts.joined(separator: "+")
    }
    
    /// Convert VirtualKey enum value to macOS key equivalent string
    private static func virtualKeyToKeyEquivalent(_ vkCode: Int32) -> String? {
        guard let vk = VirtualKey(rawValue: vkCode) else { return nil }
        
        switch vk {
        // Control keys
        case .back: return "\u{08}"      // Backspace
        case .tab: return "\t"           // Tab
        case .return: return "\r"        // Return/Enter
        case .escape: return "\u{1B}"    // Escape
        case .space: return " "          // Space
        case .delete: return "\u{7F}"    // Forward Delete
        
        // Number keys
        case .key0: return "0"
        case .key1: return "1"
        case .key2: return "2"
        case .key3: return "3"
        case .key4: return "4"
        case .key5: return "5"
        case .key6: return "6"
        case .key7: return "7"
        case .key8: return "8"
        case .key9: return "9"
        
        // Letter keys
        case .keyA: return "a"
        case .keyB: return "b"
        case .keyC: return "c"
        case .keyD: return "d"
        case .keyE: return "e"
        case .keyF: return "f"
        case .keyG: return "g"
        case .keyH: return "h"
        case .keyI: return "i"
        case .keyJ: return "j"
        case .keyK: return "k"
        case .keyL: return "l"
        case .keyM: return "m"
        case .keyN: return "n"
        case .keyO: return "o"
        case .keyP: return "p"
        case .keyQ: return "q"
        case .keyR: return "r"
        case .keyS: return "s"
        case .keyT: return "t"
        case .keyU: return "u"
        case .keyV: return "v"
        case .keyW: return "w"
        case .keyX: return "x"
        case .keyY: return "y"
        case .keyZ: return "z"
        
        // Function keys
        case .f1: return NSString(format: "%c", NSF1FunctionKey) as String
        case .f2: return NSString(format: "%c", NSF2FunctionKey) as String
        case .f3: return NSString(format: "%c", NSF3FunctionKey) as String
        case .f4: return NSString(format: "%c", NSF4FunctionKey) as String
        case .f5: return NSString(format: "%c", NSF5FunctionKey) as String
        case .f6: return NSString(format: "%c", NSF6FunctionKey) as String
        case .f7: return NSString(format: "%c", NSF7FunctionKey) as String
        case .f8: return NSString(format: "%c", NSF8FunctionKey) as String
        case .f9: return NSString(format: "%c", NSF9FunctionKey) as String
        case .f10: return NSString(format: "%c", NSF10FunctionKey) as String
        case .f11: return NSString(format: "%c", NSF11FunctionKey) as String
        case .f12: return NSString(format: "%c", NSF12FunctionKey) as String
        
        // OEM keys
        case .oem1: return ";"           // Semicolon
        case .oemPlus: return "="        // Equal/Plus
        case .oemComma: return ","       // Comma
        case .oemMinus: return "-"       // Minus
        case .oemPeriod: return "."      // Period
        case .oem2: return "/"           // Slash
        case .oem3: return "`"           // Grave/Tilde
        case .oem4: return "["           // Left Bracket
        case .oem5: return "\\"          // Backslash
        case .oem6: return "]"           // Right Bracket
        case .oem7: return "'"           // Quote
        
        // Keys that don't have direct key equivalents
        case .shift, .control, .menu, .capital, .pause, .prior, .next,
             .numpad0, .numpad1, .numpad2, .numpad3, .numpad4,
             .numpad5, .numpad6, .numpad7, .numpad8, .numpad9,
             .multiply, .add, .separator, .subtract, .decimal, .divide,
             .kanji:
            return nil
        }
    }
}

// MARK: - Logging Support

private func LOG_DEBUG(_ message: String) {
    NSLog("KeyMagicEngine: \(message)")
}