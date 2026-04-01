# Android Build Instructions

Use the following command to build the Android APK anytime.

### **Current Build Command**
To build the APK in Debug mode:
```bash
ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk build --target aarch64-linux-android
```

To build a signed Release APK:
```bash
ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk build --target aarch64-linux-android --release
```

To build and run on a connected device:
```bash
ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk run --target aarch64-linux-android
```

### **Output Path**
Debug APK: `target/debug/apk/NodeChat.apk`
Release APK: `target/release/apk/NodeChat.apk`

### **Persistent Environment (Optional)**
You can add these to your `~/.zshrc` or `~/.bashrc` to simplify the command:
```bash
alias build-android="ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk build --target aarch64-linux-android"
alias run-android="ANDROID_BUILD_TOOLS_VERSION=35.0.0 ANDROID_PLATFORM=30 cargo apk run --target aarch64-linux-android"
```
Then you can just type `build-android`.

### **Environment Requirements**
This project requires:
- **Java 17+** (Installed in `/usr/lib/jvm/default-java`).
- **Android SDK Level 30 & 33** (Installed in `/opt/android-sdk/platforms`).
- **Build Tools 35.0.0** (Installed in `/opt/android-sdk/build-tools`).
- **NDK 27+** (Installed in `/opt/android-sdk/ndk`).
