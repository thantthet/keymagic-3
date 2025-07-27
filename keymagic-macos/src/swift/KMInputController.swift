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
    private var currentBundleId: String = "unknown"
    private var useCompositionMode: Bool = true
    private var supportsTSMDocumentAccess: Bool = false
    private var deleteFailedLastTime: Bool = false
    private var metadataCache: [String: KeyboardMetadata] = [:]  // Cache keyboard metadata by ID
    
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
        
        // Get keycode and modifiers
        let keycode = event.keyCode
        let modifiers = event.modifierFlags.rawValue
        
        // Clear delete failure flag on each key event
        deleteFailedLastTime = false

        guard event.type == .keyDown else {
            
            if keycode == kVK_Delete, !useCompositionMode {
                LOG_DEBUG("Backspace key up event - eating (direct mode)")
                // some app like safari will process backspace key up event, so we need to eat it
                return true
            }

            LOG_DEBUG("Ignoring key up event")

            return false
        }
        
        guard let client = sender as? (IMKTextInput & NSObjectProtocol) else {
            LOG_DEBUG("Invalid client type")
            return false
        }
        
        // Get character string
        let chars = event.characters ?? ""
        
        // Log key event details
        LOG_KEY_EVENT(keycode, chars, modifiers)
        
        // Skip processing for Command key combinations
        if (modifiers & NSEvent.ModifierFlags.command.rawValue) != 0 {
            LOG_DEBUG("Skipping Command key combination")

            // Commit and reset if in composition mode
            if useCompositionMode {
                commitAndReset(client: client)
            } else if let engine = engine {
                // Just reset engine if nothing to commit
                keymagic_engine_reset(engine)
            }
            
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

            // Commit and reset if in composition mode
            if useCompositionMode {
                commitAndReset(client: client)
            } else {
                // Just reset engine if nothing to commit
                keymagic_engine_reset(engine)
            }
            
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
        
        // Synchronize engine state with client text before processing in direct mode
        if !useCompositionMode {
            synchronizeEngineWithClient(client: client)
        }
        
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
            LOG_TEXT("Output - composing_text", String(cString: output.composing_text!))
            
            // Log output details
            if let textPtr = output.text {
                let text = String(cString: textPtr)
                LOG_TEXT("Output text", text)
            }
            
            // Process output if the key was handled by the engine
            if output.is_processed != 0 {
                LOG_DEBUG("Engine processed the key - handling output")
                processOutput(&output, keycode: keycode, client: client)
            } else {
                // Engine didn't process the key - ensure clean state
                LOG_DEBUG("Engine did not process the key")
                
                // Clear any existing preedit in composition mode
                if useCompositionMode {
                    commitAndReset(client: client)
                }
                
                // Reset engine for special keys that weren't processed
                switch Int(keycode) {
                case kVK_Return,
                     kVK_Tab,
                     kVK_Escape:
                    LOG_DEBUG("Resetting engine for unprocessed special key")
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
            
            // In direct mode, insert ZWS for backspace and return false to prevent double deletion
            // BUT skip the workaround if the last delete operation failed (let the app handle it)
            if keycode == kVK_Delete && !useCompositionMode && event.type == .keyDown && processed && !deleteFailedLastTime {
                LOG_DEBUG("Direct mode backspace - inserting ZWS and returning false")
                client.insertText("\u{200B}", replacementRange: NSRange(location: NSNotFound, length: 0))
                LOG_DEBUG("Key processing complete - consumed=FALSE (ZWS workaround)")
                return false
            } else if keycode == kVK_Delete && deleteFailedLastTime {
                LOG_DEBUG("Last delete failed - letting backspace pass through without workaround")
                return false
            }
            
            LOG_DEBUG("Key processing complete - consumed=\(processed ? "TRUE" : "FALSE")")
            return processed
        }

        LOG_DEBUG("Engine process failed with result: \(result)")
        return false
    }
    
    // MARK: - Output Processing
    
    private func processOutput(_ output: inout ProcessKeyOutput, keycode: UInt16, client sender: (IMKTextInput & NSObjectProtocol)) {
        LOG_DEBUG("processOutput called")
        
        if useCompositionMode {
            processOutputCompositionMode(&output, keycode: keycode, client: sender)
        } else {
            processOutputDirectMode(&output, keycode: keycode, client: sender)
        }
    }
    
    private func processOutputDirectMode(_ output: inout ProcessKeyOutput, keycode: UInt16, client sender: (IMKTextInput & NSObjectProtocol)) {
        LOG_DEBUG("Direct mode - committing immediately (TSMDocumentAccess: \(supportsTSMDocumentAccess))")
        
        let client = sender
        
        // Handle text replacement (delete + insert)
        if output.delete_count > 0 || output.text != nil {
            let textToInsert = output.text != nil ? String(cString: output.text!) : ""
            
            if output.delete_count > 0 {
                LOG_DEBUG("Direct mode - replacing \(output.delete_count) characters with '\(textToInsert)'")
                
                // Get current selection/cursor position
                let currentRange = client.selectedRange()
                LOG_DEBUG("Current range: location=\(currentRange.location), length=\(currentRange.length)")
                
                if currentRange.location != NSNotFound && currentRange.location >= output.delete_count {
                    // Calculate range to replace (delete characters before cursor)
                    let replacementRange = NSRange(
                        location: currentRange.location - Int(output.delete_count),
                        length: Int(output.delete_count)
                    )
                    LOG_DEBUG("Replacement range: location=\(replacementRange.location), length=\(replacementRange.length)")
                    
                    if textToInsert.isEmpty {
                        // For pure deletion, use our helper method
                        deleteCharacters(count: UInt32(output.delete_count), 
                                       currentRange: currentRange, 
                                       replacementRange: replacementRange, 
                                       client: client)
                    } else {
                        // For replacement (delete + insert), this works well
                        client.insertText(textToInsert, replacementRange: replacementRange)
                    }
                } else {
                    // Fallback: just insert text if we can't determine proper range
                    LOG_DEBUG("Cannot determine proper range, just inserting text")
                    if !textToInsert.isEmpty {
                        client.insertText(textToInsert, replacementRange: NSRange(location: NSNotFound, length: 0))
                    }
                    // Set flag since we couldn't perform the deletion properly
                    if output.delete_count > 0 && textToInsert.isEmpty {
                        deleteFailedLastTime = true
                    }
                }
            } else {
                // Just insert text without deletion
                LOG_TEXT("Direct mode - inserting text", textToInsert)
                client.insertText(textToInsert, replacementRange: NSRange(location: NSNotFound, length: 0))
            }
        }
    }
    
    private func processOutputCompositionMode(_ output: inout ProcessKeyOutput, keycode: UInt16, client sender: (IMKTextInput & NSObjectProtocol)) {
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
                
                // Commit and reset
                commitAndReset(client: sender)
            }
        } else {
            // Just update preedit display
            LOG_TEXT("Updating marked text with", composingText)
            updateMarkedText(composingText, client: sender)
        }
    }
    
    private func deleteCharacters(count: UInt32, currentRange: NSRange, replacementRange: NSRange, client: IMKTextInput & NSObjectProtocol) {
        LOG_DEBUG("Pure deletion - attempting to delete \(count) UTF-16 units")
        
        var remainingUtf16ToDelete = Int(count)
        var totalUtf16Deleted = 0
        
        // Primary method: Get text before cursor and replace with truncated version
        while remainingUtf16ToDelete > 0 && currentRange.location > totalUtf16Deleted {
            // Calculate how much text to request (in UTF-16 units)
            let requestLength = min(remainingUtf16ToDelete + 20, Int(currentRange.location) - totalUtf16Deleted)
            if requestLength <= 0 {
                LOG_DEBUG("No more text available to delete")
                break
            }
            
            let requestRange = NSRange(
                location: currentRange.location - totalUtf16Deleted - requestLength,
                length: requestLength
            )
            
            var actualRange = NSRange()
            if let textBeforeCursor = client.string(from: requestRange, actualRange: &actualRange),
               !textBeforeCursor.isEmpty {
                LOG_DEBUG("Got text: '\(textBeforeCursor)' (chars: \(textBeforeCursor.count), utf16: \(textBeforeCursor.utf16.count), actual range: \(actualRange))")
                
                // Work directly with UTF-16 representation
                let utf16Array = Array(textBeforeCursor.utf16)
                let utf16Count = utf16Array.count
                
                // Calculate how many UTF-16 units we can delete
                let utf16ToDelete = min(remainingUtf16ToDelete, utf16Count)
                
                // Prevent deleting all UTF-16 units (would result in empty insertText)
                let safeUtf16ToDelete = (utf16ToDelete == utf16Count && utf16Count > 0) 
                    ? max(0, utf16ToDelete - 1)
                    : utf16ToDelete
                
                if safeUtf16ToDelete > 0 {
                    // Create truncated UTF-16 array
                    let truncatedUtf16 = Array(utf16Array.dropLast(safeUtf16ToDelete))
                    
                    // Convert back to String
                    let truncatedText = String(utf16CodeUnits: truncatedUtf16, count: truncatedUtf16.count)
                    let truncatedUtf16Count = truncatedUtf16.count
                    
                    LOG_DEBUG("Replacing with truncated text: '\(truncatedText)' (deleted \(safeUtf16ToDelete) utf16 units, remaining utf16 length: \(truncatedUtf16Count))")
                    
                    // Store the expected cursor position after deletion
                    let expectedLocation = actualRange.location + truncatedUtf16Count
                    
                    // Replace the actual range with truncated text
                    client.insertText(truncatedText, replacementRange: actualRange)
                    
                    // Verify the deletion happened by checking cursor position
                    let newRange = client.selectedRange()
                    if newRange.location == expectedLocation {
                        // Success - cursor is where we expect
                        remainingUtf16ToDelete -= safeUtf16ToDelete
                        totalUtf16Deleted += safeUtf16ToDelete
                        
                        LOG_DEBUG("Deletion verified - cursor moved to expected position \(expectedLocation)")
                        LOG_DEBUG("Deleted \(safeUtf16ToDelete) utf16 units, remaining: \(remainingUtf16ToDelete)")
                    } else {
                        // Deletion might have failed
                        LOG_DEBUG("Deletion verification failed - expected cursor at \(expectedLocation), but got \(newRange.location)")
                        
                        // Try to calculate actual deletion based on cursor movement (in UTF-16 units)
                        let actuallyDeletedUtf16 = Int(currentRange.location) - Int(newRange.location) - totalUtf16Deleted
                        if actuallyDeletedUtf16 > 0 {
                            remainingUtf16ToDelete -= actuallyDeletedUtf16
                            totalUtf16Deleted += actuallyDeletedUtf16
                            LOG_DEBUG("Partial deletion detected - actually deleted \(actuallyDeletedUtf16) utf16 units")
                        } else {
                            LOG_DEBUG("No deletion detected - stopping")
                            break
                        }
                    }
                    
                    // Small delay to let the change propagate
                    Thread.sleep(forTimeInterval: 0.001)
                } else {
                    LOG_DEBUG("Cannot delete from this text chunk (would result in empty string)")
                    break
                }
            } else {
                LOG_DEBUG("Failed to get text from client or got empty text")
                break
            }
        }
        
        if totalUtf16Deleted > 0 {
            LOG_DEBUG("Successfully deleted \(totalUtf16Deleted) UTF-16 units using text replacement")
        }
        
        if remainingUtf16ToDelete > 0 {
            LOG_DEBUG("Could not delete all requested UTF-16 units. Deleted: \(totalUtf16Deleted), Failed: \(remainingUtf16ToDelete)")
            // Set flag to indicate deletion failure
            deleteFailedLastTime = true
        }
    }
    
    // MARK: - Engine State Synchronization
    
    private func synchronizeEngineWithClient(client: IMKTextInput & NSObjectProtocol) {
        guard let engine = engine else { return }
        
        // Get engine's current composing text
        let composingTextPtr = keymagic_engine_get_composition(engine)
        if composingTextPtr == nil {
            // Engine has no composing text, nothing to synchronize
            return
        }
        
        defer {
            keymagic_free_string(composingTextPtr)
        }
        
        let engineComposingText = String(cString: composingTextPtr!)
        if engineComposingText.isEmpty {
            return
        }
        
        LOG_DEBUG("Engine composing text: '\(engineComposingText)'")
        
        // Get text from client before cursor
        let selectedRange = client.selectedRange()
        if selectedRange.location == NSNotFound || selectedRange.location == 0 {
            // No valid cursor position or cursor at beginning
            LOG_DEBUG("No text before cursor to compare")
            keymagic_engine_reset(engine)
            return
        }
        
        // First, try to get text matching engine composing length
        let engineUtf16Length = engineComposingText.utf16.count
        
        // Request based on UTF-16 length (add some extra to ensure we get enough)
        let lengthToGet = min(engineUtf16Length + 10, Int(selectedRange.location))
        let rangeBeforeCursor = NSRange(location: selectedRange.location - lengthToGet, length: lengthToGet)
        
        // Get the text from client
        var actualRange = NSRange()
        if let textBeforeCursor = client.string(from: rangeBeforeCursor, actualRange: &actualRange) {
            LOG_DEBUG("Text before cursor: '\(textBeforeCursor)' (chars: \(textBeforeCursor.count), utf16: \(textBeforeCursor.utf16.count), actual range: \(actualRange))")
            
            // For comparison, we need to check if we got enough text
            let textUtf16Length = textBeforeCursor.utf16.count
            
            // Check if we got text that could match the engine text
            if textUtf16Length >= engineUtf16Length {
                // Extract the suffix that matches engine text UTF-16 length
                let suffixStartUtf16 = textBeforeCursor.utf16.index(textBeforeCursor.utf16.endIndex, offsetBy: -engineUtf16Length)
                let suffixText = String(textBeforeCursor.utf16[suffixStartUtf16...])!
                
                if suffixText == engineComposingText {
                    LOG_DEBUG("Engine state matches client text (suffix match)")
                    return  // No sync needed
                } else {
                    LOG_DEBUG("Engine state does not match (full UTF-16 comparison)")
                    // Will sync below
                }
            } else if textBeforeCursor == engineComposingText {
                // Got less text, but it exactly matches engine text
                LOG_DEBUG("Engine state matches client text (exact match)")
                return  // No sync needed
            } else if engineComposingText.hasSuffix(textBeforeCursor) {
                // Engine text has document text as suffix
                LOG_DEBUG("Engine state has document text as suffix - keeping engine state")
                return  // Keep current engine state
            } else {
                LOG_DEBUG("Engine state does not match (partial comparison)")
                // Will sync below
            }
            
            // If we reach here, texts don't match and we need to sync
            // Get up to 40 UTF-16 units before cursor for context (roughly 20 chars, but safe for emoji)
            let contextLength = min(40, Int(selectedRange.location))
            let contextRange = NSRange(location: selectedRange.location - contextLength, length: contextLength)
            
            var contextActualRange = NSRange()
            if let contextText = client.string(from: contextRange, actualRange: &contextActualRange) {
                LOG_DEBUG("Syncing engine with document context: '\(contextText)'")
                contextText.withCString { textPtr in
                    _ = keymagic_engine_set_composition(engine, textPtr)
                }
            } else {
                // If we can't get context, just reset
                LOG_DEBUG("Could not get document context - resetting engine")
                keymagic_engine_reset(engine)
            }
        } else if selectedRange.location > 0 {
            // Engine has composing text but we couldn't get text from document
            // Try to sync with whatever text is available
            let availableLength = min(40, Int(selectedRange.location))  // 40 UTF-16 units
            let availableRange = NSRange(location: selectedRange.location - availableLength, length: availableLength)
            
            var availableActualRange = NSRange()
            if let availableText = client.string(from: availableRange, actualRange: &availableActualRange) {
                LOG_DEBUG("Syncing engine with available document text: '\(availableText)'")
                availableText.withCString { textPtr in
                    _ = keymagic_engine_set_composition(engine, textPtr)
                }
            } else {
                LOG_DEBUG("No text available - resetting engine")
                keymagic_engine_reset(engine)
            }
        } else {
            // Cursor at beginning, reset engine
            LOG_DEBUG("Cursor at beginning - resetting engine")
            keymagic_engine_reset(engine)
        }
    }
    
    
    // MARK: - Composition Management
    
    private func commitAndReset(client sender: (IMKTextInput & NSObjectProtocol)) {
        LOG_DEBUG("Committing and resetting engine")

        // Commit the text
        commitText(composingText, client: sender)

        // Clear composing text
        composingText = ""
        
        // Reset engine
        if let engine = engine {
            keymagic_engine_reset(engine)
        }
    }
    
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
    
    private func updateMarkedText(_ text: String, client sender: (IMKTextInput & NSObjectProtocol)) {
        let client = sender
        
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
    
    private func commitText(_ text: String, client sender: (IMKTextInput & NSObjectProtocol)) {
        let client = sender
        
        LOG_TEXT("Attempting to commit marked text", text)
        
        client.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
        clearMarkedText(client: sender)
        
        LOG_TEXT("Committed marked text", text)
    }
    
    private func clearMarkedText(client sender: (IMKTextInput & NSObjectProtocol)) {
        let client = sender
        
        if !composingText.isEmpty {
            LOG_DEBUG("Cleared marked text")
        }
        client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: NSRange(location: NSNotFound, length: 0))
        composingText = ""
    }
    
    // MARK: - Process Detection
    
    private func getClientBundleIdentifier(_ client: (IMKTextInput & NSObjectProtocol)) -> String {
        // Get bundle identifier directly from client
        if let bundleId = client.bundleIdentifier() {
            LOG_DEBUG("Got bundle identifier from client: \(bundleId)")
            return bundleId
        }
        
        // Fallback: Use the frontmost application
        if let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier {
            LOG_DEBUG("Using frontmost app bundle ID: \(bundleId)")
            return bundleId
        }
        
        LOG_DEBUG("Could not determine client bundle identifier")
        return "unknown"
    }
    
    private func checkTSMDocumentAccess(_ client: (IMKTextInput & NSObjectProtocol)) -> Bool {
        // Check if the client supports TSMDocumentAccess using the proper API
        let supportsAccess = client.supportsProperty(TSMDocumentPropertyTag(kTSMDocumentSupportDocumentAccessPropertyTag))
        
        if supportsAccess {
            LOG_DEBUG("Client supports TSMDocumentAccess property")
        } else {
            LOG_DEBUG("Client does not support TSMDocumentAccess")
        }
        
        // Also check other useful properties for debugging
        let supportsTextService = client.supportsProperty(TSMDocumentPropertyTag(kTSMDocumentTextServicePropertyTag))
        let supportsUnicode = client.supportsProperty(TSMDocumentPropertyTag(kTSMDocumentUnicodePropertyTag))
        
        LOG_DEBUG("Client TSM properties - Access: \(supportsAccess), TextService: \(supportsTextService), Unicode: \(supportsUnicode)")
        
        return supportsAccess
    }
    
    // MARK: - State Management
    
    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        
        LOG_DEBUG("Focus in")
        
        guard let client = sender as? (IMKTextInput & NSObjectProtocol) else {
            // Reset engine state even if no valid client
            if let engine = engine {
                keymagic_engine_reset(engine)
            }
            return
        }
        
        // Detect client bundle ID and set input mode
        currentBundleId = getClientBundleIdentifier(client)
        useCompositionMode = !KMConfiguration.shared.shouldUseDirectMode(for: currentBundleId)
        supportsTSMDocumentAccess = checkTSMDocumentAccess(client)
        
        LOG_DEBUG("Activated for bundle: \(currentBundleId), mode: \(useCompositionMode ? "Composition" : "Direct"), TSMDocumentAccess: \(supportsTSMDocumentAccess)")
        
        // Reset engine state
        if let engine = engine {
            keymagic_engine_reset(engine)
        }
        
        // Clear any existing composition
        clearMarkedText(client: client)
    }
    
    override func deactivateServer(_ sender: Any!) {
        LOG_DEBUG("Focus out")

        guard let client = sender as? (IMKTextInput & NSObjectProtocol) else {
            // Still reset engine even without valid client
            if let engine = engine {
                keymagic_engine_reset(engine)
            }
            super.deactivateServer(sender)
            return
        }
        
        // Commit and reset if in composition mode
        if useCompositionMode {
            commitAndReset(client: client)
        } else if let engine = engine {
            // Just reset engine if nothing to commit
            keymagic_engine_reset(engine)
        }
        
        super.deactivateServer(sender)
    }
    
    override func commitComposition(_ sender: Any!) {
        LOG_DEBUG("Reset")

        guard let client = sender as? (IMKTextInput & NSObjectProtocol) else {
            // Still reset engine even without valid client
            if let engine = engine {
                keymagic_engine_reset(engine)
            }
            return
        }
        
        // Commit and reset if in composition mode
        if useCompositionMode {
            commitAndReset(client: client)
        } else if let engine = engine {
            // Just reset engine if nothing to commit
            keymagic_engine_reset(engine)
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
            self?.clearMetadataCache()  // Clear cache when config changes
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
    
    /// Keyboard metadata from KM2 file
    private struct KeyboardMetadata {
        let name: String?
        let description: String?
        let hotkey: String?
    }
    
    /// Get metadata from a KM2 file with caching
    private func getMetadataFromKM2(id: String, path: String) -> KeyboardMetadata? {
        // Check cache first
        if let cached = metadataCache[id] {
            return cached
        }
        
        // Load from file
        guard let km2Handle = path.withCString({ keymagic_km2_load($0) }) else {
            return nil
        }
        defer { keymagic_km2_free(km2Handle) }
        
        var name: String?
        var description: String?
        var hotkey: String?
        
        // Get name
        if let namePtr = keymagic_km2_get_name(km2Handle) {
            name = String(cString: namePtr)
            keymagic_free_string(namePtr)
        }
        
        // Get description
        if let descPtr = keymagic_km2_get_description(km2Handle) {
            description = String(cString: descPtr)
            keymagic_free_string(descPtr)
        }
        
        // Get hotkey
        if let hotkeyPtr = keymagic_km2_get_hotkey(km2Handle) {
            hotkey = String(cString: hotkeyPtr)
            keymagic_free_string(hotkeyPtr)
        }
        
        let metadata = KeyboardMetadata(name: name, description: description, hotkey: hotkey)
        
        // Cache the result
        metadataCache[id] = metadata
        
        return metadata
    }
    
    /// Clear metadata cache (e.g., when keyboards are installed/uninstalled)
    private func clearMetadataCache() {
        metadataCache.removeAll()
    }
    
    override func menu() -> NSMenu! {
        LOG_DEBUG("Creating menu")
        let menu = NSMenu(title: "KeyMagic")
        
        // Load available keyboards from config
        let config = KMConfiguration.shared
        let keyboards = config.installedKeyboards
        LOG_DEBUG("Found \(keyboards.count) keyboards")
        
        // Add keyboards directly to the main menu
        if keyboards.isEmpty {
            let noKeyboardsItem = NSMenuItem(title: "No Keyboards Installed", action: nil, keyEquivalent: "")
            noKeyboardsItem.isEnabled = false
            menu.addItem(noKeyboardsItem)
        } else {
            for (index, keyboard) in keyboards.enumerated() {
                guard let id = keyboard["id"] else { continue }
                
                var displayName = keyboard["name"] ?? id
                var hotkeyString = keyboard["hotkey"]
                var description: String?
                
                // Get metadata from KM2 file if available
                if let keyboardPath = KMConfiguration.shared.getKeyboardPath(for: id),
                   let metadata = getMetadataFromKM2(id: id, path: keyboardPath) {
                    // Use name from KM2 if not in config
                    if keyboard["name"] == nil, let km2Name = metadata.name {
                        displayName = km2Name
                    }
                    
                    // Use hotkey from KM2 if not in config
                    if hotkeyString == nil, let km2Hotkey = metadata.hotkey {
                        hotkeyString = km2Hotkey
                        LOG_DEBUG("Got hotkey from KM2 file for \(displayName): \(km2Hotkey)")
                    }
                    
                    // Get description for tooltip
                    description = metadata.description
                }
                
                LOG_DEBUG("Adding keyboard menu item: \(displayName) (id: \(id))")
                
                let menuItem = NSMenuItem(title: displayName, action: #selector(selectionChanged(_:)), keyEquivalent: "")
                menuItem.target = self
                menuItem.representedObject = id
                menuItem.tag = index
                menuItem.isEnabled = true
                
                // Set hotkey if available
                if let hotkeyStr = hotkeyString,
                   hotkeyStr != "",
                   let hotkey = MacHotkey.parse(hotkeyStr) {
                    LOG_DEBUG("Setting hotkey for \(displayName): \(hotkey.debugDescription)")
                    hotkey.applyTo(menuItem)
                }
                
                // Set tooltip with description if available
                if let desc = description {
                    menuItem.toolTip = desc
                }
                
                // Check current keyboard
                if id == currentKeyboardId {
                    menuItem.state = .on
                }
                
                menu.addItem(menuItem)
            }
        }
        
        menu.addItem(NSMenuItem.separator())
        
        // Add preferences item
        let preferencesItem = NSMenuItem(title: "Preferences...", action: #selector(showKeyMagicPreferences), keyEquivalent: ",")
        preferencesItem.target = self
        menu.addItem(preferencesItem)
        
        return menu
    }
    
    @objc private func selectionChanged(_ sender: Any) {
        LOG_DEBUG("selectionChanged called with sender: \(type(of: sender))")
        
        // IMK passes a dictionary with the menu item
        guard let dict = sender as? [String: Any],
              let menuItem = dict["IMKCommandMenuItem"] as? NSMenuItem else {
            LOG_DEBUG("Could not get menu item from sender")
            return
        }
        
        // Get the keyboard ID from representedObject
        guard let keyboardId = menuItem.representedObject as? String else {
            LOG_DEBUG("No keyboard ID in menu item")
            return
        }
        
        LOG_DEBUG("Selected keyboard ID: \(keyboardId)")
        selectKeyboardById(keyboardId)
    }
    
    private func selectKeyboardById(_ keyboardId: String) {
        // Commit any ongoing composition before switching keyboards
        if useCompositionMode, let currentClient = client() {
            LOG_DEBUG("Committing composition before keyboard switch")
            commitAndReset(client: currentClient)
        } else if let engine = engine {
            // Just reset engine if not in composition mode
            LOG_DEBUG("Resetting engine before keyboard switch (direct mode)")
            keymagic_engine_reset(engine)
        }
        
        // Update configuration
        let config = KMConfiguration.shared
        if let keyboardPath = config.getKeyboardPath(for: keyboardId) {
            if loadKeyboard(id: keyboardId, path: keyboardPath) {
                LOG_DEBUG("Switched to keyboard: \(keyboardId)")
                
                // Save the active keyboard to config
                config.setActiveKeyboard(keyboardId)
                
                // Show notification
                showKeyboardSwitchNotification(keyboardId: keyboardId)
            }
        }
    }
    
    
    private func showKeyboardSwitchNotification(keyboardId: String) {
        // Get keyboard name for display
        var keyboardName = keyboardId
        
        // Try to get the actual keyboard name from metadata
        if let keyboardPath = KMConfiguration.shared.getKeyboardPath(for: keyboardId),
           let metadata = getMetadataFromKM2(id: keyboardId, path: keyboardPath),
           let name = metadata.name {
            keyboardName = name
        }
        
        // For Input Methods, we'll use the old NSUserNotification API which doesn't require permissions
        // This works well for transient notifications that don't need user interaction
        let notification = NSUserNotification()
        notification.title = "KeyMagic 3"
        notification.informativeText = "Switched to: \(keyboardName)"
        notification.soundName = nil // No sound for keyboard switches
        
        // Set notification to disappear automatically after a short time
        notification.hasActionButton = false
        
        // Deliver the notification
        NSUserNotificationCenter.default.deliver(notification)
        
        // Remove the notification after 2 seconds
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
            NSUserNotificationCenter.default.removeDeliveredNotification(notification)
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