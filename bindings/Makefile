STATIC_LIB_NAME := libbindings.a
TARGET_DIR := ../target
CARGO_BUILD_CMD := cargo build --release

apple:
	@make remove-output
	@make build-targets
	@make bindgen-swift
	@make assemble-frameworks
	@make xcframework
	@make cp-xcframework-source


remove-output:
	rm -rf ../output/Sources/Bindings ../output/Sources/BindingsFFI.xcframework

build-targets:
	$(CARGO_BUILD_CMD) --lib --target x86_64-apple-ios
	$(CARGO_BUILD_CMD) --lib --target aarch64-apple-ios-sim
	$(CARGO_BUILD_CMD) --lib --target aarch64-apple-ios
	$(CARGO_BUILD_CMD) --lib --target aarch64-apple-darwin
	$(CARGO_BUILD_CMD) --lib --target x86_64-apple-darwin

bindgen-swift:
	$(CARGO_BUILD_CMD)
	cargo run --release -p uniffi-bindgen generate --language swift --lib-file $(TARGET_DIR)/release/libbindings.dylib src/bindings.udl
	sed -i '' 's/module\ BindingsFFI/framework\ module\ BindingsFFI/' src/BindingsFFI.modulemap

assemble-frameworks:
	cd $(TARGET_DIR) && find . -type d -name BindingsFFI.framework -exec rm -rf {} \; || echo "rm failed"
	cd $(TARGET_DIR)/x86_64-apple-ios/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework && mkdir Headers Modules Resources && pwd && cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap && cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h && cp ../$(STATIC_LIB_NAME) ./BindingsFFI && cp ../../../../misc/apple/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-ios-sim/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework && mkdir Headers Modules Resources && cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap && cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h && cp ../$(STATIC_LIB_NAME) ./BindingsFFI && cp ../../../../misc/apple/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-ios/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework && mkdir Headers Modules Resources && cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap && cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h && cp ../$(STATIC_LIB_NAME) ./BindingsFFI && cp ../../../../misc/apple/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-darwin/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework && mkdir Headers Modules Resources && cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap && cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h && cp ../$(STATIC_LIB_NAME) ./BindingsFFI && cp ../../../../misc/apple/Info.plist ./Resources
	cd $(TARGET_DIR)/x86_64-apple-darwin/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework && mkdir Headers Modules Resources && cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap && cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h && cp ../$(STATIC_LIB_NAME) ./BindingsFFI && cp ../../../../misc/apple/Info.plist ./Resources

xcframework:
  # lipo used to create universal(aarch64 + x86_64) files
	lipo -create $(TARGET_DIR)/x86_64-apple-ios/release/BindingsFFI.framework/BindingsFFI \
		$(TARGET_DIR)/aarch64-apple-ios-sim/release/BindingsFFI.framework/BindingsFFI \
		-output $(TARGET_DIR)/aarch64-apple-ios-sim/release/BindingsFFI.framework/BindingsFFI
	lipo -create $(TARGET_DIR)/aarch64-apple-darwin/release/BindingsFFI.framework/BindingsFFI \
		$(TARGET_DIR)/x86_64-apple-darwin/release/BindingsFFI.framework/BindingsFFI \
		-output $(TARGET_DIR)/aarch64-apple-darwin/release/BindingsFFI.framework/BindingsFFI

	rm -rf $(TARGET_DIR)/BindingsFFI.xcframework || echo "skip removing"

	xcodebuild -create-xcframework \
		-framework $(TARGET_DIR)/aarch64-apple-ios/release/BindingsFFI.framework \
		-framework $(TARGET_DIR)/aarch64-apple-ios-sim/release/BindingsFFI.framework \
		-framework $(TARGET_DIR)/aarch64-apple-darwin/release/BindingsFFI.framework \
		-output $(TARGET_DIR)/BindingsFFI.xcframework

cp-xcframework-source:
	mkdir -p ../output/Sources/Bindings
	cp -r $(TARGET_DIR)/BindingsFFI.xcframework ../output/Sources
	cp src/Bindings.swift ../output/Sources/Bindings
