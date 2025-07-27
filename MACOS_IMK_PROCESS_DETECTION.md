# macOS IMK Process-Based Input Mode Selection

Yes, IMK provides enough information to implement process-based input mode selection (direct vs composition mode) similar to the Windows TSF implementation!

## Getting Client Process Information

```objc
// In KMInputController.m

- (NSString *)getClientProcessName:(id)client {
    // Get the client's process ID
    pid_t clientPID = 0;
    
    // Try to get PID from client proxy (undocumented but widely used)
    if ([client respondsToSelector:@selector(_pid)]) {
        clientPID = [(id)client _pid];
    }
    
    // Get NSRunningApplication from PID
    if (clientPID > 0) {
        NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:clientPID];
        if (app) {
            return [app.executableURL.lastPathComponent lowercaseString];
        }
    }
    
    return @"unknown";
}
```

## Simple Input Mode Configuration

```objc
// KMInputModeConfig.h
typedef NS_ENUM(NSInteger, KMInputMode) {
    KMInputModeComposition,  // Show marked text (default)
    KMInputModeDirect        // Direct insertion
};

@interface KMInputModeConfig : NSObject

+ (KMInputMode)inputModeForProcess:(NSString *)processName;
+ (void)setInputMode:(KMInputMode)mode forProcess:(NSString *)processName;
+ (void)loadConfiguration;
+ (void)saveConfiguration;

@end

// KMInputModeConfig.m
@implementation KMInputModeConfig

static NSMutableDictionary<NSString *, NSNumber *> *processModesDict = nil;
static NSString *const kConfigKey = @"ProcessInputModes";

+ (void)initialize {
    if (self == [KMInputModeConfig class]) {
        [self loadConfiguration];
    }
}

+ (void)loadConfiguration {
    processModesDict = [[[NSUserDefaults standardUserDefaults] 
        dictionaryForKey:kConfigKey] mutableCopy] ?: [NSMutableDictionary dictionary];
    
    // Default configurations
    if (processModesDict.count == 0) {
        // Terminal apps typically work better with direct mode
        processModesDict[@"terminal.app"] = @(KMInputModeDirect);
        processModesDict[@"iterm.app"] = @(KMInputModeDirect);
        processModesDict[@"hyper.app"] = @(KMInputModeDirect);
    }
}

+ (void)saveConfiguration {
    [[NSUserDefaults standardUserDefaults] setObject:processModesDict forKey:kConfigKey];
}

+ (KMInputMode)inputModeForProcess:(NSString *)processName {
    NSNumber *mode = processModesDict[processName.lowercaseString];
    return mode ? mode.integerValue : KMInputModeComposition;
}

+ (void)setInputMode:(KMInputMode)mode forProcess:(NSString *)processName {
    processModesDict[processName.lowercaseString] = @(mode);
    [self saveConfiguration];
}

@end
```

## Integration in Input Controller

```objc
// In KMInputController.m

@interface KMInputController ()
@property (nonatomic) KMInputMode currentInputMode;
@property (nonatomic, copy) NSString *currentProcessName;
@end

@implementation KMInputController

- (void)activateServer:(id)sender {
    [super activateServer:sender];
    
    // Get client process info and set input mode
    self.currentProcessName = [self getClientProcessName:sender];
    self.currentInputMode = [KMInputModeConfig inputModeForProcess:self.currentProcessName];
    
    DEBUG_LOG(@"Activated for process: %@, mode: %@", 
              self.currentProcessName, 
              self.currentInputMode == KMInputModeDirect ? @"Direct" : @"Composition");
}

- (BOOL)handleEvent:(NSEvent *)event client:(id)sender {
    // Quick check if process changed
    NSString *processName = [self getClientProcessName:sender];
    if (![processName isEqualToString:self.currentProcessName]) {
        self.currentProcessName = processName;
        self.currentInputMode = [KMInputModeConfig inputModeForProcess:processName];
    }
    
    // Process key event...
    KeyMagicResult result = [self processKey:event];
    
    // Apply result based on input mode
    [self applyResult:result client:sender];
    
    return result.isProcessed;
}

- (void)applyResult:(KeyMagicResult)result client:(id)sender {
    if (self.currentInputMode == KMInputModeDirect) {
        // Direct mode - immediately commit any text
        if (result.backspaceCount > 0) {
            for (int i = 0; i < result.backspaceCount; i++) {
                [sender deleteBackward:nil];
            }
        }
        if (result.textToInsert.length > 0) {
            [sender insertText:result.textToInsert 
             replacementRange:NSMakeRange(NSNotFound, 0)];
        }
    } else {
        // Composition mode - use marked text
        if (result.composingText.length > 0) {
            NSAttributedString *markedText = [[NSAttributedString alloc] 
                initWithString:result.composingText
                attributes:@{NSUnderlineStyleAttributeName: @(NSUnderlineStyleSingle)}];
            
            [sender setMarkedText:markedText 
                   selectionRange:NSMakeRange(result.composingText.length, 0)
                 replacementRange:NSMakeRange(NSNotFound, 0)];
        } else {
            // Clear marked text
            [sender setMarkedText:@"" 
                   selectionRange:NSMakeRange(0, 0)
                 replacementRange:NSMakeRange(NSNotFound, 0)];
        }
    }
}

@end
```

## Special Case: Web View Processes

```objc
- (NSString *)getEffectiveProcessName:(id)client {
    NSString *processName = [self getClientProcessName:client];
    
    // Handle web content processes
    if ([processName isEqualToString:@"com.apple.webkit.webcontent"]) {
        // For web content, default to composition mode
        // since we can't reliably get the parent browser
        return @"webcontent";
    }
    
    return processName;
}
```

## Usage Example

```objc
// User preference UI or configuration
[KMInputModeConfig setInputMode:KMInputModeDirect forProcess:@"vscode.app"];
[KMInputModeConfig setInputMode:KMInputModeComposition forProcess:@"pages.app"];
```

## Key Points

1. **Simple Configuration**: Just store process name → input mode mapping
2. **User Defaults**: Use NSUserDefaults for persistence (no separate config file needed)
3. **Automatic Detection**: Process is detected when IMK activates for a client
4. **Quick Switching**: Mode changes immediately when switching between apps
5. **Sensible Defaults**: Terminal apps default to direct mode, others to composition mode

This simplified approach focuses only on what matters - selecting between direct and composition modes per application, just like the Windows TSF implementation.