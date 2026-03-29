# Android & Desktop Rust (eframe/egui) Development Guide

This guide documents the critical configuration and code patterns required to successfully build and run an `eframe` application for both **Linux (Wayland/X11)** and **Android** using `cargo-apk` and standard `cargo`.

## 1. Toolchain & Prerequisites

### Patched `cargo-apk`
The standard `cargo-apk` may have issues with some workspace layouts. It is recommended to use the following patched version:

```bash
cargo install \
    --git https://github.com/parasyte/cargo-apk.git \
    --rev 282639508eeed7d73f2e1eaeea042da2716436d5 \
    cargo-apk
```

### Targets
Enable the Android targets for Rust:

```bash
rustup target add aarch64-linux-android
# Option for emulator:
rustup target add x86_64-linux-android
```

## 2. Cargo.toml Configuration

### Crate Type
To package as an Android app while maintaining desktop support, you MUST include both `cdylib` and `rlib` library types:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

### Target-Specific Dependencies (Crucial)
To avoid windowing backend conflicts (e.g., `Glutin` errors on Linux) and ensure Android symbols are correctly exported, use target-specific dependencies.

```toml
[dependencies]
eframe = { version = "0.34", default-features = false, features = ["glow", "default_fonts", "wayland", "x11"] }
log = "0.4"

[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.15"
# Explicitly enable native-activity to export ANativeActivity_onCreate
android-activity = { path = "vendor/android-activity", features = ["native-activity"] }
winit = { version = "0.30", features = ["android-native-activity"] }
# Redundant link here ensures android-native-activity is active for eframe's transitive winit
eframe = { version = "0.34", default-features = false, features = ["glow", "android-native-activity"] }
```

> [!IMPORTANT]
> **Android Symbol Fix**: If `android-native-activity` is not correctly enabled on both `winit` and `eframe` (for Android), the build will fail at runtime with `UnsatisfiedLinkError: undefined symbol: ANativeActivity_onCreate`.
> **Linux Window Fix**: If desktop execution fails with `NotSupported("provided native window is not supported")`, ensure you aren't forcing Android features on Linux and that all `egui` family crates (`egui-wgpu`, etc.) are updated to matching versions (e.g., run `cargo update`).

## 3. Implementation Patterns

### Entry Point (`src/lib.rs`)
The Android system calls `android_main`. It must be marked `#[unsafe(no_mangle)]` and match the `winit` signature.

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("PANIC: {}", panic_info);
    }));

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    );
}
```

### Desktop Entry Point (`src/main.rs`)
Standard `main` function for Linux/Windows/macOS:

```rust
use hello_android::MyApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "hello_android",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
```

## 4. Build & Run Commands

### Desktop (Linux/Wayland/X11)
```bash
cargo run
```

### Android Development Build & Run
```bash
# Automated build, install, and run
cargo apk run --target aarch64-linux-android
```

### Android Release Build (Signed)
```bash
cargo apk build --release --target aarch64-linux-android
# Resulting APK: target/release/apk/hello_android.apk
```

## 5. Troubleshooting Reference

- **`undefined symbol: ANativeActivity_onCreate`**: Ensure `native-activity` feature is explicitly enabled on `android-activity` and `android-native-activity` is on `winit`/`eframe` in the Android target section.
- **`Glutin: NotSupported("provided native window is not supported")`**: 
    - Check if you are on Wayland and running an X11-only build (or vice-versa).
    - Run `cargo update` to sync `egui-wgpu` versions.
    - Try forcing XWayland with `WINIT_UNIX_BACKEND=x11 cargo run`.
- **`Package not found` during install**: Ensure you are installing from the correct path (`target/debug/apk/` vs `target/release/apk/`) matching your build profile.
