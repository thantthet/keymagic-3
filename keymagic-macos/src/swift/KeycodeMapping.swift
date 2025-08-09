//
//  KeycodeMapping.swift
//  KeyMagic
//
//  Maps macOS virtual keycodes to KeyMagic VirtualKey values
//

import Foundation
import Carbon.HIToolbox

// KeyMagic VirtualKey enum values (from Rust)
enum VirtualKey: UInt32 {
    case back = 2
    case tab = 3
    case `return` = 4
    case shift = 5
    case control = 6
    case menu = 7       // Alt
    case pause = 8
    case capital = 9    // Caps Lock
    case kanji = 10
    case escape = 11
    case space = 12
    case prior = 13     // Page Up
    case next = 14      // Page Down
    case delete = 15
    
    // Numbers
    case key0 = 16
    case key1 = 17
    case key2 = 18
    case key3 = 19
    case key4 = 20
    case key5 = 21
    case key6 = 22
    case key7 = 23
    case key8 = 24
    case key9 = 25
    
    // Letters
    case keyA = 26
    case keyB = 27
    case keyC = 28
    case keyD = 29
    case keyE = 30
    case keyF = 31
    case keyG = 32
    case keyH = 33
    case keyI = 34
    case keyJ = 35
    case keyK = 36
    case keyL = 37
    case keyM = 38
    case keyN = 39
    case keyO = 40
    case keyP = 41
    case keyQ = 42
    case keyR = 43
    case keyS = 44
    case keyT = 45
    case keyU = 46
    case keyV = 47
    case keyW = 48
    case keyX = 49
    case keyY = 50
    case keyZ = 51
    
    // Numpad
    case numpad0 = 52
    case numpad1 = 53
    case numpad2 = 54
    case numpad3 = 55
    case numpad4 = 56
    case numpad5 = 57
    case numpad6 = 58
    case numpad7 = 59
    case numpad8 = 60
    case numpad9 = 61
    
    // Numpad operators
    case multiply = 62
    case add = 63
    case separator = 64
    case subtract = 65
    case decimal = 66
    case divide = 67
    
    // Function keys
    case f1 = 68
    case f2 = 69
    case f3 = 70
    case f4 = 71
    case f5 = 72
    case f6 = 73
    case f7 = 74
    case f8 = 75
    case f9 = 76
    case f10 = 77
    case f11 = 78
    case f12 = 79
    
    // OEM keys
    case oem1 = 86      // ;:
    case oemPlus = 87
    case oemComma = 88
    case oemMinus = 89
    case oemPeriod = 90
    case oem2 = 91      // /?
    case oem3 = 92      // `~
    case oem4 = 93      // [{
    case oem5 = 94      // \|
    case oem6 = 95      // ]}
    case oem7 = 96      // '"
}

// Extension to map macOS keycodes
extension UInt16 {
    var toVirtualKey: VirtualKey? {
        switch Int(self) {
        // Letters
        case kVK_ANSI_A: return .keyA
        case kVK_ANSI_B: return .keyB
        case kVK_ANSI_C: return .keyC
        case kVK_ANSI_D: return .keyD
        case kVK_ANSI_E: return .keyE
        case kVK_ANSI_F: return .keyF
        case kVK_ANSI_G: return .keyG
        case kVK_ANSI_H: return .keyH
        case kVK_ANSI_I: return .keyI
        case kVK_ANSI_J: return .keyJ
        case kVK_ANSI_K: return .keyK
        case kVK_ANSI_L: return .keyL
        case kVK_ANSI_M: return .keyM
        case kVK_ANSI_N: return .keyN
        case kVK_ANSI_O: return .keyO
        case kVK_ANSI_P: return .keyP
        case kVK_ANSI_Q: return .keyQ
        case kVK_ANSI_R: return .keyR
        case kVK_ANSI_S: return .keyS
        case kVK_ANSI_T: return .keyT
        case kVK_ANSI_U: return .keyU
        case kVK_ANSI_V: return .keyV
        case kVK_ANSI_W: return .keyW
        case kVK_ANSI_X: return .keyX
        case kVK_ANSI_Y: return .keyY
        case kVK_ANSI_Z: return .keyZ
        
        // Numbers
        case kVK_ANSI_1: return .key1
        case kVK_ANSI_2: return .key2
        case kVK_ANSI_3: return .key3
        case kVK_ANSI_4: return .key4
        case kVK_ANSI_5: return .key5
        case kVK_ANSI_6: return .key6
        case kVK_ANSI_7: return .key7
        case kVK_ANSI_8: return .key8
        case kVK_ANSI_9: return .key9
        case kVK_ANSI_0: return .key0
        
        // Control keys
        case kVK_Return: return .return
        case kVK_Escape: return .escape
        case kVK_Delete: return .back
        case kVK_Tab: return .tab
        case kVK_Space: return .space
        
        // Punctuation
        case kVK_ANSI_Minus: return .oemMinus           // - _
        case kVK_ANSI_Equal: return .oemPlus            // = +
        case kVK_ANSI_LeftBracket: return .oem4         // [ {
        case kVK_ANSI_RightBracket: return .oem6        // ] }
        case kVK_ANSI_Backslash: return .oem5           // \ |
        case kVK_ANSI_Semicolon: return .oem1           // ; :
        case kVK_ANSI_Quote: return .oem7               // ' "
        case kVK_ANSI_Grave: return .oem3               // ` ~
        case kVK_ANSI_Comma: return .oemComma           // , <
        case kVK_ANSI_Period: return .oemPeriod         // . >
        case kVK_ANSI_Slash: return .oem2               // / ?
        
        // Function keys
        case kVK_F1: return .f1
        case kVK_F2: return .f2
        case kVK_F3: return .f3
        case kVK_F4: return .f4
        case kVK_F5: return .f5
        case kVK_F6: return .f6
        case kVK_F7: return .f7
        case kVK_F8: return .f8
        case kVK_F9: return .f9
        case kVK_F10: return .f10
        case kVK_F11: return .f11
        case kVK_F12: return .f12
        
        // Navigation
        case kVK_PageUp: return .prior      // Page Up
        case kVK_PageDown: return .next     // Page Down
        case kVK_ForwardDelete: return .delete
        
        // Modifiers
        case kVK_CapsLock: return .capital      // Caps Lock
        case kVK_Shift: return .shift           // Left Shift
        case kVK_RightShift: return .shift      // Right Shift
        case kVK_Control: return .control       // Left Control
        case kVK_RightControl: return .control  // Right Control
        case kVK_Option: return .menu           // Left Option/Alt
        case kVK_RightOption: return .menu      // Right Option/Alt
        
        // Numpad
        case kVK_ANSI_Keypad0: return .numpad0
        case kVK_ANSI_Keypad1: return .numpad1
        case kVK_ANSI_Keypad2: return .numpad2
        case kVK_ANSI_Keypad3: return .numpad3
        case kVK_ANSI_Keypad4: return .numpad4
        case kVK_ANSI_Keypad5: return .numpad5
        case kVK_ANSI_Keypad6: return .numpad6
        case kVK_ANSI_Keypad7: return .numpad7
        case kVK_ANSI_Keypad8: return .numpad8
        case kVK_ANSI_Keypad9: return .numpad9
        
        default: return nil
        }
    }
}