import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var menu: NSMenu!

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        if let button = statusItem.button {
            button.image = NSImage(systemSymbolName: "music.note", accessibilityDescription: "MSSIC")
            button.image?.size = NSSize(width: 18, height: 18)
            button.image?.isTemplate = true
            button.action = #selector(handleClick(_:))
            button.target = self
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }

        // Menu only shows on right-click
        menu = NSMenu()
        let openItem = NSMenuItem(title: "Open MSSIC", action: #selector(openTerminal), keyEquivalent: "o")
        openItem.target = self
        menu.addItem(openItem)
        menu.addItem(NSMenuItem.separator())
        let quitItem = NSMenuItem(title: "Quit", action: #selector(quitApp), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)
    }

    @objc private func handleClick(_ sender: NSStatusBarButton) {
        guard let event = NSApp.currentEvent else { return }

        if event.type == .rightMouseUp {
            // Right-click: show menu
            statusItem.menu = menu
            statusItem.button?.performClick(nil)
            // Reset so left-click works again next time
            DispatchQueue.main.async { self.statusItem.menu = nil }
        } else {
            // Left-click: open TUI directly
            openTerminal()
        }
    }

    @objc private func openTerminal() {
        let binaryPath = findBinary()

        let script = """
        tell application "Terminal"
            activate
            do script "\(binaryPath)"
        end tell
        """

        if let appleScript = NSAppleScript(source: script) {
            var error: NSDictionary?
            appleScript.executeAndReturnError(&error)
            if let error = error {
                NSLog("MSSIC: AppleScript error: \(error)")
            }
        }
    }

    private func findBinary() -> String {
        // 1. Same directory as this app's bundle
        if let bundlePath = Bundle.main.bundlePath as String? {
            let bundleDir = (bundlePath as NSString).deletingLastPathComponent
            let candidate = (bundleDir as NSString).appendingPathComponent("mssic-player")
            if FileManager.default.isExecutableFile(atPath: candidate) {
                return candidate
            }
        }

        // 2. Inside the app bundle
        if let bundlePath = Bundle.main.bundlePath as String? {
            let candidate = (bundlePath as NSString)
                .appendingPathComponent("Contents/MacOS/mssic-player")
            if FileManager.default.isExecutableFile(atPath: candidate) {
                return candidate
            }
        }

        // 3. Common install paths
        let candidates = [
            "/usr/local/bin/mssic-player",
            "\(NSHomeDirectory())/.cargo/bin/mssic",
        ]
        for path in candidates {
            if FileManager.default.isExecutableFile(atPath: path) {
                return path
            }
        }

        return "mssic-player"
    }

    @objc private func quitApp() {
        NSApplication.shared.terminate(nil)
    }
}

// --- Entry Point ---
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let delegate = AppDelegate()
app.delegate = delegate
app.run()
