# Android Migration & Setup Log

This document records the exact steps taken to migrate NodeChat from a desktop-only Slint app to a fully-functional, signed Android application.

## **1. Environment Setup**
To build Slint for Android on Linux, we installed the following system-level dependencies:

- **Java JDK 21**: Required for Slint's Java bridge and `keytool`.
- **Android SDK Platform 30 & 33**: Required for its `android.jar` and system libraries.
- **Android Build-Tools 35.0.0**: Essential to fix a known crash in older `d8` (Dex) compilers when running on Java 21.

## **2. Project Configuration (`Cargo.toml`)**
We updated the project's metadata to support the `cargo-apk` tool:

- **Enabled Slint Android Backend**: Added `features = ["backend-android-activity-06"]` to the `slint` dependency.
- **Library Configuration**: Added a `[lib]` section with `crate-type = ["cdylib", "rlib"]` so Android can load the project as a native library.
- **Android Metadata**: Named the APK `NodeChat`, defined the package as `com.nodechat.app`, and set `target_sdk_version` to 33.

## **3. Code Refactoring**
To handle the transition from a standard `main()` entry point to the Android-specific entry point:

- **Unified Entry Point (`run_app`)**: Moved all UI and Core initialization into a shareable `run_app()` function.
- **Android Entry Point**: Added `#[unsafe(no_mangle)] fn android_main(app: AndroidApp)` to handle the Android life-cycle and automatically initialize the Slint backend.
- **Rust 2024 Compliance**: Switched from `#[no_mangle]` to the safer `#[unsafe(no_mangle)]` as required by the latest Rust compiler editions in use.

## **4. Signing & Release**
To enable fully optimized release builds for distribution:

- **Generated Production Keystore**: Created a 2,048-bit RSA keystore named `release.keystore` valid for 10,000 days.
- **Integrated Signing**: Configured `Cargo.toml` to automatically sign the APK using this file during the build process.

## **5. Final Build Commands**
We verified the builds with the following environment-pinned commands:

- **Debug Build**:
  ```bash
  ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk build --target aarch64-linux-android
  ```

- **Signed Production Release**:
  ```bash
  ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk build --target aarch64-linux-android --release
  ```

- **Output Location**:
  All generated APKs are placed in `target/(debug|release)/apk/NodeChat.apk`.
