# How to View KeyMagic IMK Logs on macOS

## Overview
When the KeyMagic Input Method Kit (IMK) runs, it's executed by the system as part of the Input Method services. The logs written using `NSLog()` are captured by the system logging facility.

## Viewing Logs

### Method 1: Using Console.app (GUI)
1. Open **Console.app** (found in `/Applications/Utilities/`)
2. In the search field, type: `KeyMagic`
3. You'll see all log messages that start with "KeyMagic:"
4. You can filter further by:
   - Process: Look for your IMK process name
   - Subsystem: com.apple.InputMethodKit

### Method 2: Using Terminal with `log` command (Real-time)
```bash
# Stream logs in real-time
log stream --predicate 'eventMessage contains "KeyMagic"'

# Stream logs with more detail
log stream --predicate 'eventMessage contains "KeyMagic"' --info --debug

# Stream logs from specific process
log stream --predicate 'process == "KeyMagic" OR eventMessage contains "KeyMagic"'
```

### Method 3: Query Historical Logs
```bash
# Show logs from the last hour
log show --predicate 'eventMessage contains "KeyMagic"' --last 1h

# Show logs from the last 30 minutes with timestamps
log show --predicate 'eventMessage contains "KeyMagic"' --last 30m --info

# Export logs to a file
log show --predicate 'eventMessage contains "KeyMagic"' --last 1h > keymagic_logs.txt
```

### Method 4: Using Traditional System Logs
```bash
# Check system.log (older macOS versions)
tail -f /var/log/system.log | grep KeyMagic

# Check for crash logs
ls ~/Library/Logs/DiagnosticReports/ | grep KeyMagic
```

## Log Levels Added

The logging added to `KMInputController.swift` includes:

1. **Key Event Details**
   - Keycode, characters, and modifier flags
   - Virtual key conversion status

2. **Engine Processing**
   - Whether engine is available
   - Processing success/failure
   - Output details (text, composing text, processed flag)

3. **Composition Management**
   - When text is being committed
   - Marked text updates
   - Engine resets

4. **Special Key Handling**
   - Command key skipping
   - Printable key eating when no engine
   - Special key resets (Return, Tab, Escape)

## Debugging Tips

1. **Start streaming before testing**: Run the log stream command before switching to your IMK to capture all events.

2. **Use specific predicates**: Combine predicates for more targeted logging:
   ```bash
   log stream --predicate '(eventMessage contains "KeyMagic") AND (eventMessage contains "process")'
   ```

3. **Enable debug logging**: For more verbose output during development, you can add:
   ```swift
   // Add at class level
   private let debugLogging = true
   
   // Use conditional logging
   if debugLogging {
       NSLog("KeyMagic DEBUG: Detailed info here")
   }
   ```

4. **Check IMK registration**: Verify your IMK is properly registered:
   ```bash
   # List all installed input methods
   defaults read /Library/Preferences/com.apple.HIToolbox.plist AppleEnabledInputSources
   ```

## Common Issues

- **No logs appearing**: Make sure the IMK is properly installed and selected in System Preferences > Keyboard > Input Sources
- **Logs delayed**: System logging can have a slight delay; wait a few seconds
- **Too many logs**: Use more specific predicates or grep to filter