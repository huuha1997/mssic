.PHONY: all build-rust build-menubar bundle clean install

APP_NAME = MSSIC.app
BUNDLE_DIR = build/$(APP_NAME)

all: bundle

# Build the Rust TUI binary
build-rust:
	cargo build --release

# Build the Swift menu bar app
build-menubar:
	@mkdir -p build
	swiftc -O \
		-o build/mssic-menubar \
		-framework AppKit \
		menubar/MssicMenuBar.swift

# Create macOS .app bundle
bundle: build-rust build-menubar
	@echo "Creating $(APP_NAME) bundle..."
	@mkdir -p "$(BUNDLE_DIR)/Contents/MacOS"
	@mkdir -p "$(BUNDLE_DIR)/Contents/Resources"
	@cp menubar/Info.plist "$(BUNDLE_DIR)/Contents/"
	@cp build/mssic-menubar "$(BUNDLE_DIR)/Contents/MacOS/MssicMenuBar"
	@cp target/release/mssic "$(BUNDLE_DIR)/Contents/MacOS/mssic-player"
	@echo "Built: $(BUNDLE_DIR)"

# Install to /Applications
install: bundle
	@cp -r "$(BUNDLE_DIR)" /Applications/
	@echo "Installed to /Applications/$(APP_NAME)"

# Clean build artifacts
clean:
	cargo clean
	rm -rf build
