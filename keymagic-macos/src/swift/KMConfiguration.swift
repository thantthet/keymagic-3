import Foundation

/// KeyMagic configuration management for macOS IMK
/// Reads configuration from the same location as GUI: ~/Library/Preferences/net.keymagic/config.toml
@objc public class KMConfiguration: NSObject {
    private struct Config: Codable {
        var general: GeneralConfig
        var keyboards: KeyboardsConfig
        var composition_mode: CompositionModeConfig?
    }
    
    private struct GeneralConfig: Codable {
        var start_with_system: Bool
        var check_for_updates: Bool
        var last_update_check: String?
        var last_scanned_version: String?
    }
    
    private struct KeyboardsConfig: Codable {
        var active: String?
        var last_used: [String]
        var installed: [InstalledKeyboard]
    }
    
    private struct InstalledKeyboard: Codable {
        var id: String
        var name: String
        var filename: String
        var hotkey: String?
        var hash: String
    }
    
    private struct CompositionModeConfig: Codable {
        var enabled_hosts: [String]
    }
    
    // MARK: - Singleton
    @objc public static let shared = KMConfiguration()
    
    // MARK: - Properties
    private let configDir: URL
    private let dataDir: URL
    private let keyboardsDir: URL
    private let configPath: URL
    private var config: Config?
    
    // MARK: - Public Properties
    @objc public var activeKeyboardId: String? {
        return config?.keyboards.active
    }
    
    @objc public var installedKeyboards: [[String: String]] {
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
    @objc public func shouldUseCompositionMode(for bundleId: String) -> Bool {
        NSLog("KeyMagic: Bundle ID: \(bundleId)")
        NSLog("KeyMagic: Composition mode: \(config?.composition_mode)")
        // If no composition mode config, default to direct mode
        guard let compositionMode = config?.composition_mode else {
            return false
        }
        
        // Check if the bundle ID is in the enabled list (case-insensitive)
        let lowercaseBundleId = bundleId.lowercased()
        return compositionMode.enabled_hosts.contains { enabledHost in
            enabledHost.lowercased() == lowercaseBundleId
        }
    }
    
    // MARK: - Initialization
    private override init() {
        // Setup directories following GUI convention
        let libraryDir = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first!
        
        // Config: ~/Library/Preferences/net.keymagic/
        self.configDir = libraryDir.appendingPathComponent("Preferences/net.keymagic")
        self.configPath = configDir.appendingPathComponent("config.toml")
        
        // Data: ~/Library/Application Support/KeyMagic/
        self.dataDir = libraryDir.appendingPathComponent("Application Support/KeyMagic")
        self.keyboardsDir = dataDir.appendingPathComponent("Keyboards")
        
        super.init()
        
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
    @objc public func loadConfig() {
        do {
            if FileManager.default.fileExists(atPath: configPath.path) {
                let data = try Data(contentsOf: configPath)
                if let tomlString = String(data: data, encoding: .utf8) {
                    config = try parseToml(tomlString)
                    NSLog("KeyMagic: Loaded config with active keyboard: \(activeKeyboardId ?? "none")")
                }
            } else {
                // Create default config if it doesn't exist
                createDefaultConfig()
            }
        } catch {
            NSLog("KeyMagic: Failed to load config: \(error)")
            createDefaultConfig()
        }
    }
    
    private func createDefaultConfig() {
        config = Config(
            general: GeneralConfig(
                start_with_system: false,
                check_for_updates: true,
                last_update_check: nil,
                last_scanned_version: nil
            ),
            keyboards: KeyboardsConfig(
                active: nil,
                last_used: [],
                installed: []
            ),
            composition_mode: CompositionModeConfig(
                enabled_hosts: []
            )
        )
    }
    
    // MARK: - Keyboard File Management
    @objc public func getKeyboardPath(for keyboardId: String) -> String? {
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
    
    @objc public func getAllKeyboardFiles() -> [String] {
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
    
    // MARK: - TOML Parsing
    // Simple TOML parser for our specific config structure
    // In production, we should use a proper TOML library
    private func parseToml(_ toml: String) throws -> Config {
        // This is a simplified parser - in production use a proper TOML library
        var config = Config(
            general: GeneralConfig(
                start_with_system: false,
                check_for_updates: true,
                last_update_check: nil,
                last_scanned_version: nil
            ),
            keyboards: KeyboardsConfig(
                active: nil,
                last_used: [],
                installed: []
            ),
            composition_mode: CompositionModeConfig(enabled_hosts: [])
        )
        
        // Parse active keyboard
        if let match = toml.range(of: #"active = "(.+?)""#, options: .regularExpression) {
            let value = String(toml[match])
            if let activeMatch = value.range(of: #""(.+?)""#, options: .regularExpression) {
                let active = String(value[activeMatch])
                config.keyboards.active = active.replacingOccurrences(of: "\"", with: "")
            }
        }
        
        // Parse installed keyboards array
        if let installedStart = toml.range(of: "[[keyboards.installed]]") {
            var installedKeyboards: [InstalledKeyboard] = []
            var searchRange = installedStart.upperBound..<toml.endIndex
            
            while let nextStart = toml.range(of: "[[keyboards.installed]]", range: searchRange) {
                // Parse one keyboard entry
                let entryEnd = toml.range(of: "[[keyboards.installed]]", 
                                         range: nextStart.upperBound..<toml.endIndex)?.lowerBound ?? toml.endIndex
                let entry = String(toml[nextStart.lowerBound..<entryEnd])
                
                if let keyboard = parseKeyboardEntry(entry) {
                    installedKeyboards.append(keyboard)
                }
                
                searchRange = nextStart.upperBound..<toml.endIndex
            }
            
            // Parse the last entry if exists
            if searchRange.lowerBound < toml.endIndex {
                let lastEntry = String(toml[searchRange])
                if let keyboard = parseKeyboardEntry(lastEntry) {
                    installedKeyboards.append(keyboard)
                }
            }
            
            config.keyboards.installed = installedKeyboards
        }
        
        // Parse composition mode enabled hosts
        if let compositionModeStart = toml.range(of: "[composition_mode]") {
            var enabledHosts: [String] = []
            
            let searchStart = compositionModeStart.upperBound
            
            // Look for enabled_hosts field
            if let enabledStart = toml.range(of: "enabled_hosts = [", range: searchStart..<toml.endIndex) {
                // Find the closing bracket
                if let closingBracket = toml.range(of: "]", range: enabledStart.upperBound..<toml.endIndex) {
                    let arrayContent = String(toml[enabledStart.upperBound..<closingBracket.lowerBound])
                    
                    // Parse the array content
                    let hostNames = arrayContent.components(separatedBy: ",")
                    for hostName in hostNames {
                        let trimmed = hostName.trimmingCharacters(in: .whitespacesAndNewlines)
                        if trimmed.hasPrefix("\"") && trimmed.hasSuffix("\"") {
                            let name = String(trimmed.dropFirst().dropLast())
                            if !name.isEmpty {
                                enabledHosts.append(name)
                            }
                        }
                    }
                }
            }
            
            if !enabledHosts.isEmpty {
                config.composition_mode = CompositionModeConfig(enabled_hosts: enabledHosts)
            }
        }
        
        return config
    }
    
    private func parseKeyboardEntry(_ entry: String) -> InstalledKeyboard? {
        var id: String?
        var name: String?
        var filename: String?
        var hash: String?
        
        // Extract fields using simple regex
        if let match = entry.range(of: #"id = "(.+?)""#, options: .regularExpression) {
            let value = String(entry[match])
            if let valueMatch = value.range(of: #""(.+?)""#, options: .regularExpression) {
                id = String(value[valueMatch]).replacingOccurrences(of: "\"", with: "")
            }
        }
        
        if let match = entry.range(of: #"name = "(.+?)""#, options: .regularExpression) {
            let value = String(entry[match])
            if let valueMatch = value.range(of: #""(.+?)""#, options: .regularExpression) {
                name = String(value[valueMatch]).replacingOccurrences(of: "\"", with: "")
            }
        }
        
        if let match = entry.range(of: #"filename = "(.+?)""#, options: .regularExpression) {
            let value = String(entry[match])
            if let valueMatch = value.range(of: #""(.+?)""#, options: .regularExpression) {
                filename = String(value[valueMatch]).replacingOccurrences(of: "\"", with: "")
            }
        }
        
        if let match = entry.range(of: #"hash = "(.+?)""#, options: .regularExpression) {
            let value = String(entry[match])
            if let valueMatch = value.range(of: #""(.+?)""#, options: .regularExpression) {
                hash = String(value[valueMatch]).replacingOccurrences(of: "\"", with: "")
            }
        }
        
        guard let id = id, let name = name, let filename = filename, let hash = hash else {
            return nil
        }
        
        return InstalledKeyboard(id: id, name: name, filename: filename, hotkey: nil, hash: hash)
    }
}