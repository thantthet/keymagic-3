import Foundation

/// KeyMagic configuration management for macOS IMK
/// Reads configuration from the same location as GUI: ~/Library/Preferences/net.keymagic/config.plist
public class KMConfiguration {
    // Pure Swift structs with Codable for automatic serialization
    private struct Config: Codable {
        var general: GeneralConfig
        var keyboards: KeyboardsConfig
        var compositionMode: CompositionModeConfig?
        var directMode: DirectModeConfig?
        
        private enum CodingKeys: String, CodingKey {
            case general
            case keyboards
            case compositionMode = "composition_mode"
            case directMode = "direct_mode"
        }
    }
    
    private struct GeneralConfig: Codable {
        var startWithSystem: Bool
        var checkForUpdates: Bool
        var lastUpdateCheck: String?
        var lastScannedVersion: String?
        var updateRemindAfter: String?
        
        private enum CodingKeys: String, CodingKey {
            case startWithSystem = "start_with_system"
            case checkForUpdates = "check_for_updates"
            case lastUpdateCheck = "last_update_check"
            case lastScannedVersion = "last_scanned_version"
            case updateRemindAfter = "update_remind_after"
        }
    }
    
    private struct KeyboardsConfig: Codable {
        var active: String?
        var lastUsed: [String]
        var installed: [InstalledKeyboard]
        
        private enum CodingKeys: String, CodingKey {
            case active
            case lastUsed = "last_used"
            case installed
        }
    }
    
    private struct InstalledKeyboard: Codable {
        var id: String
        var name: String
        var filename: String
        var hotkey: String?
        var hash: String
    }
    
    private struct CompositionModeConfig: Codable {
        var enabledHosts: [String]
        
        private enum CodingKeys: String, CodingKey {
            case enabledHosts = "enabled_hosts"
        }
    }
    
    private struct DirectModeConfig: Codable {
        var enabledHosts: [String]
        
        private enum CodingKeys: String, CodingKey {
            case enabledHosts = "enabled_hosts"
        }
    }
    
    // MARK: - Singleton
    public static let shared = KMConfiguration()
    
    // MARK: - Properties
    private let configDir: URL
    private let dataDir: URL
    private let keyboardsDir: URL
    private let configPath: URL
    private var config: Config?
    
    // MARK: - Public Properties
    public var activeKeyboardId: String? {
        return config?.keyboards.active
    }
    
    public var installedKeyboards: [[String: String]] {
        guard let keyboards = config?.keyboards.installed else { return [] }
        return keyboards.map { keyboard in
            [
                "id": keyboard.id,
                "name": keyboard.name,
                "filename": keyboard.filename,
                "hash": keyboard.hash
            ]
        }
    }
    
    // MARK: - Composition Mode Management
    public func shouldUseCompositionMode(for bundleId: String) -> Bool {
        NSLog("KeyMagic: Bundle ID: \(bundleId)")
        NSLog("KeyMagic: Composition mode: \(String(describing: config?.compositionMode))")
        // If no composition mode config, default to direct mode
        guard let compositionMode = config?.compositionMode else {
            return false
        }
        
        // Check if the bundle ID is in the enabled list (case-insensitive)
        let lowercaseBundleId = bundleId.lowercased()
        return compositionMode.enabledHosts.contains { enabledHost in
            enabledHost.lowercased() == lowercaseBundleId
        }
    }
    
    // MARK: - Direct Mode Management
    public func shouldUseDirectMode(for bundleId: String) -> Bool {
        NSLog("KeyMagic: Bundle ID: \(bundleId)")
        NSLog("KeyMagic: Direct mode: \(String(describing: config?.directMode))")
        // If no direct mode config, default to composition mode (return false)
        guard let directMode = config?.directMode else {
            return false
        }
        
        // Check if the bundle ID is in the enabled list (case-insensitive)
        let lowercaseBundleId = bundleId.lowercased()
        return directMode.enabledHosts.contains { enabledHost in
            enabledHost.lowercased() == lowercaseBundleId
        }
    }
    
    // MARK: - Initialization
    private init() {
        // Setup directories following GUI convention
        let libraryDir = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first!
        
        // Config: ~/Library/Preferences/net.keymagic/
        self.configDir = libraryDir.appendingPathComponent("Preferences/net.keymagic")
        self.configPath = configDir.appendingPathComponent("config.plist")
        
        // Data: ~/Library/Application Support/KeyMagic/
        self.dataDir = libraryDir.appendingPathComponent("Application Support/KeyMagic")
        self.keyboardsDir = dataDir.appendingPathComponent("Keyboards")
        
        
        // Create directories if needed
        createDirectoriesIfNeeded()
        
        // Load initial config
        loadConfig()
        
        // Monitor config file changes
        startMonitoringConfigChanges()
    }
    
    // MARK: - Directory Management
    private func createDirectoriesIfNeeded() {
        do {
            try FileManager.default.createDirectory(at: configDir, withIntermediateDirectories: true)
            try FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)
            try FileManager.default.createDirectory(at: keyboardsDir, withIntermediateDirectories: true)
        } catch {
            NSLog("KeyMagic: Failed to create directories: \(error)")
        }
    }
    
    // MARK: - Configuration Loading
    public func loadConfig() {
        if FileManager.default.fileExists(atPath: configPath.path) {
            do {
                let data = try Data(contentsOf: configPath)
                let decoder = PropertyListDecoder()
                config = try decoder.decode(Config.self, from: data)
                NSLog("KeyMagic: Loaded config with active keyboard: \(activeKeyboardId ?? "none")")
            } catch {
                NSLog("KeyMagic: Failed to load config: \(error)")
                createDefaultConfig()
            }
        } else {
            // Create default config if it doesn't exist
            createDefaultConfig()
        }
    }
    
    private func saveConfig() {
        guard let config = config else { return }
        
        do {
            let encoder = PropertyListEncoder()
            encoder.outputFormat = .binary
            let data = try encoder.encode(config)
            try data.write(to: configPath)
        } catch {
            NSLog("KeyMagic: Failed to save config: \(error)")
        }
    }
    
    private func createDefaultConfig() {
        config = Config(
            general: GeneralConfig(
                startWithSystem: false,
                checkForUpdates: true,
                lastUpdateCheck: nil,
                lastScannedVersion: nil,
                updateRemindAfter: nil
            ),
            keyboards: KeyboardsConfig(
                active: nil,
                lastUsed: [],
                installed: []
            ),
            compositionMode: CompositionModeConfig(
                enabledHosts: []
            ),
            directMode: DirectModeConfig(
                enabledHosts: [
                    "com.apple.Spotlight",
                    "com.apple.finder",
                    "com.apple.TextEdit",
                    "com.microsoft.Word",
                    "com.apple.Dictionary",
                    "ru.keepcoder.Telegram",
                    "com.tencent.xinWeChat",
                    "com.tinyspeck.slackmacgap",
                    "com.apple.Safari",
                    "com.google.Chrome",
                    "us.zoom.xos",
                    "com.apple.dt.Xcode",
                    "com.apple.AppStore"
                ]
            )
        )
        saveConfig()
    }
    
    // MARK: - Keyboard File Management
    public func getKeyboardPath(for keyboardId: String) -> String? {
        // First check if there's a matching installed keyboard
        if let keyboard = config?.keyboards.installed.first(where: { $0.id == keyboardId }) {
            let path = keyboardsDir.appendingPathComponent(keyboard.filename).path
            if FileManager.default.fileExists(atPath: path) {
                return path
            }
        }
        
        // Try direct match: keyboardId.km2
        let directPath = keyboardsDir.appendingPathComponent("\(keyboardId).km2").path
        if FileManager.default.fileExists(atPath: directPath) {
            return directPath
        }
        
        // Search through all .km2 files
        do {
            let files = try FileManager.default.contentsOfDirectory(at: keyboardsDir, 
                                                                   includingPropertiesForKeys: nil)
            for file in files where file.pathExtension == "km2" {
                let basename = file.deletingPathExtension().lastPathComponent
                if basename == keyboardId {
                    return file.path
                }
            }
        } catch {
            NSLog("KeyMagic: Failed to search keyboards directory: \(error)")
        }
        
        return nil
    }
    
    public func getAllKeyboardFiles() -> [String] {
        do {
            let files = try FileManager.default.contentsOfDirectory(at: keyboardsDir,
                                                                   includingPropertiesForKeys: nil)
            return files
                .filter { $0.pathExtension == "km2" }
                .map { $0.path }
        } catch {
            NSLog("KeyMagic: Failed to list keyboard files: \(error)")
            return []
        }
    }
    
    // MARK: - Active Keyboard Management
    public func setActiveKeyboard(_ keyboardId: String) {
        guard var config = config else { return }
        
        // Update active keyboard
        config.keyboards.active = keyboardId
        
        // Update last used list
        if !config.keyboards.lastUsed.contains(keyboardId) {
            config.keyboards.lastUsed.insert(keyboardId, at: 0)
            // Keep only last 5 keyboards
            if config.keyboards.lastUsed.count > 5 {
                config.keyboards.lastUsed = Array(config.keyboards.lastUsed.prefix(5))
            }
        }
        
        // Save updated config
        self.config = config
        saveConfig()
        
        NSLog("KeyMagic: Set active keyboard to: \(keyboardId)")
    }
    
    // MARK: - Config File Monitoring
    private var configMonitor: DispatchSourceFileSystemObject?
    
    private func startMonitoringConfigChanges() {
        let fd = open(configPath.path, O_EVTONLY)
        guard fd >= 0 else { return }
        
        configMonitor = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename],
            queue: .main
        )
        
        configMonitor?.setEventHandler { [weak self] in
            NSLog("KeyMagic: Config file changed, reloading...")
            self?.loadConfig()
            
            // Post notification for other components
            NotificationCenter.default.post(
                name: NSNotification.Name("KMConfigurationChanged"),
                object: nil
            )
        }
        
        configMonitor?.setCancelHandler {
            close(fd)
        }
        
        configMonitor?.resume()
    }
}