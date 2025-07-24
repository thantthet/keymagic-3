//
//  main.swift
//  KeyMagic
//
//  Main entry point for KeyMagic Input Method
//

import InputMethodKit
import Foundation

// Bundle identifier for the input method
let kConnectionName = "org.keymagic.inputmethod.KeyMagic3_Connection"

// Main entry point
autoreleasepool {
    // Create the server
    guard IMKServer(name: kConnectionName, bundleIdentifier: Bundle.main.bundleIdentifier) != nil else {
        NSLog("Failed to create IMK server")
        exit(1)
    }
    
    // Load the main nib file if it exists
    if Bundle.main.loadNibNamed("MainMenu", owner: NSApplication.shared, topLevelObjects: nil) {
        NSLog("Loaded MainMenu nib")
    }
    
    // Run the application
    NSApplication.shared.run()
}