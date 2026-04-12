#!/usr/bin/env bash
set -euo pipefail

# Simple Android build wrapper for cargo-apk.
# Defaults align with the previous project: build-tools 35.0.0, platform 30, arm64 target.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

export ANDROID_BUILD_TOOLS_VERSION="${ANDROID_BUILD_TOOLS_VERSION:-35.0.0}"
export ANDROID_PLATFORM="${ANDROID_PLATFORM:-30}"
export ANDROID_NDK="${ANDROID_NDK:-/opt/android-sdk/ndk/27d}"
export SLINT_BACKEND="android-activity"

TARGET="${TARGET:-aarch64-linux-android}"

echo "Building NodeChat for Android..."
echo "  ANDROID_BUILD_TOOLS_VERSION=${ANDROID_BUILD_TOOLS_VERSION}"
echo "  ANDROID_PLATFORM=${ANDROID_PLATFORM}"
echo "  TARGET=${TARGET}"

cargo apk build --target "${TARGET}" --release "$@"

echo "Done: target/${TARGET}/release"
