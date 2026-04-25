#!/usr/bin/env bash
# tauri-ios-build.sh — Replaces `pnpm tauri ios xcode-script`
# Called by Xcode's "Build Rust Code" pre-build script phase.
# Compiles the Rust static library for iOS without requiring rustup.
set -euo pipefail

# ---------------------------------------------------------------------------
# Parse arguments (matches the interface of `tauri ios xcode-script`)
# ---------------------------------------------------------------------------
VERBOSE=0
PLATFORM=""
SDK_ROOT=""
FRAMEWORK_SEARCH_PATHS=""
HEADER_SEARCH_PATHS=""
GCC_PREPROCESSOR_DEFINITIONS=""
CONFIGURATION=""
ARCHS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--verbose) VERBOSE=1; shift ;;
    --platform) PLATFORM="$2"; shift 2 ;;
    --sdk-root) SDK_ROOT="$2"; shift 2 ;;
    --framework-search-paths) FRAMEWORK_SEARCH_PATHS="$2"; shift 2 ;;
    --header-search-paths) HEADER_SEARCH_PATHS="$2"; shift 2 ;;
    --gcc-preprocessor-definitions) GCC_PREPROCESSOR_DEFINITIONS="$2"; shift 2 ;;
    --configuration) CONFIGURATION="$2"; shift 2 ;;
    --force-color) shift ;;  # ignored
    # Valid architectures only — filter out stray words from unquoted Xcode vars
    # (e.g. PLATFORM_DISPLAY_NAME "iOS Simulator" splits into "iOS" + "Simulator"
    #  and the latter leaks into positional args)
    arm64|x86_64) ARCHS+=("$1"); shift ;;
    *) PLATFORM="${PLATFORM:+$PLATFORM }$1"; shift ;;  # append to platform name
  esac
done

if [[ -z "$PLATFORM" || -z "$SDK_ROOT" || -z "$CONFIGURATION" || ${#ARCHS[@]} -eq 0 ]]; then
  echo "ERROR: Missing required arguments."
  echo "Usage: $0 --platform <PLATFORM> --sdk-root <SDK_ROOT> --configuration <CONFIG> <ARCHS...>"
  exit 1
fi

# ---------------------------------------------------------------------------
# Determine project root
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# If called from Xcode, SRCROOT is set. Derive project root from it as fallback.
if [[ -n "${SRCROOT:-}" ]]; then
  PROJECT_ROOT="$(cd "$SRCROOT/../../../../../.." && pwd)"
fi

log() { [[ "$VERBOSE" -eq 1 ]] && echo "[tauri-ios-build] $*" || true; }

log "Project root: $PROJECT_ROOT"
log "Platform: $PLATFORM"
log "SDK root: $SDK_ROOT"
log "Configuration: $CONFIGURATION"
log "Architectures: ${ARCHS[*]}"

# ---------------------------------------------------------------------------
# Ensure cargo is available (Nix env may not be inherited by Xcode)
# ---------------------------------------------------------------------------
if ! command -v cargo &>/dev/null; then
  log "cargo not found in PATH, attempting to source Nix environment..."
  # Try common Nix profile locations
  for profile in \
    "$HOME/.nix-profile/etc/profile.d/nix.sh" \
    "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" \
    "/etc/profiles/per-user/$USER/etc/profile.d/nix.sh"; do
    if [[ -f "$profile" ]]; then
      # shellcheck source=/dev/null
      source "$profile"
      break
    fi
  done
  # Also try direnv if available
  if command -v direnv &>/dev/null; then
    eval "$(direnv export bash 2>/dev/null)" || true
  fi
  if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found. Ensure Rust is available in PATH."
    exit 1
  fi
fi

log "Using cargo: $(which cargo)"
log "Using rustc: $(rustc --version)"

# ---------------------------------------------------------------------------
# Detect simulator vs device
# ---------------------------------------------------------------------------
IS_SIMULATOR=0
if [[ "$PLATFORM" == *"Simulator"* ]] || [[ "$SDK_ROOT" == *"Simulator"* ]]; then
  IS_SIMULATOR=1
fi
log "Is simulator: $IS_SIMULATOR"

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
CARGO_PKG="multi_platform_app-mobile"
LIB_NAME="multi_platform_app_mobile_lib"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target}"
APPLE_DIR="$PROJECT_ROOT/src/app/mobile/src-tauri/gen/apple"

# Map Xcode configuration to Cargo profile
CARGO_PROFILE_FLAG=""
CARGO_OUT_DIR_NAME="debug"
if [[ "$CONFIGURATION" == "release" ]]; then
  CARGO_PROFILE_FLAG="--release"
  CARGO_OUT_DIR_NAME="release"
fi

# Fallback for non-Xcode invocations (Xcode sets this from project.yml's deploymentTarget).
# swift-rs reads this to set the Swift target triple; without it, it defaults to iOS 13.0,
# causing the compiler to emit swiftCompatibility56 auto-link directives.
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-16.0}"

# ---------------------------------------------------------------------------
# Build for each architecture
# ---------------------------------------------------------------------------
for ARCH in "${ARCHS[@]}"; do
  log "--- Building for arch: $ARCH ---"

  # Map arch to Rust triple
  RUST_TRIPLE=""
  case "$ARCH" in
    arm64)
      if [[ "$IS_SIMULATOR" -eq 1 ]]; then
        RUST_TRIPLE="aarch64-apple-ios-sim"
      else
        RUST_TRIPLE="aarch64-apple-ios"
      fi
      ;;
    x86_64)
      RUST_TRIPLE="x86_64-apple-ios"
      ;;
    *)
      echo "ERROR: Unsupported architecture: $ARCH"
      exit 1
      ;;
  esac
  log "Rust triple: $RUST_TRIPLE"

  # Compute env triple (replace - with _, keep lowercase)
  ENV_TRIPLE="${RUST_TRIPLE//-/_}"
  log "Env triple: $ENV_TRIPLE"

  # Set cross-compilation environment variables
  ISYSROOT="-isysroot $SDK_ROOT"
  export "CFLAGS_${ENV_TRIPLE}=${ISYSROOT}"
  export "CXXFLAGS_${ENV_TRIPLE}=${ISYSROOT}"
  export "OBJC_INCLUDE_PATH_${ENV_TRIPLE}=${SDK_ROOT}/usr/include"

  # Set host environment for macOS build scripts
  MACOS_SDK_ROOT="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null || echo "")"
  if [[ -n "$MACOS_SDK_ROOT" ]]; then
    HOST_ARCH="$(uname -m)"
    HOST_ENV_TRIPLE="${HOST_ARCH}_apple_darwin"
    export "CFLAGS_${HOST_ENV_TRIPLE}=-isysroot ${MACOS_SDK_ROOT}"
    export "CXXFLAGS_${HOST_ENV_TRIPLE}=-isysroot ${MACOS_SDK_ROOT}"
    export "OBJC_INCLUDE_PATH_${HOST_ENV_TRIPLE}=${MACOS_SDK_ROOT}/usr/include"
  fi

  # Override CARGO_TARGET_DIR to point to repo root target/
  export CARGO_TARGET_DIR="$CARGO_TARGET_DIR"

  # Unset SDKROOT so it doesn't confuse cargo/rustc for cross-compilation
  unset SDKROOT 2>/dev/null || true

  # Run cargo build
  log "Running: cargo build --lib -p $CARGO_PKG --target $RUST_TRIPLE $CARGO_PROFILE_FLAG"
  cargo build --lib \
    -p "$CARGO_PKG" \
    --target "$RUST_TRIPLE" \
    $CARGO_PROFILE_FLAG

  # Locate the compiled static library
  LIB_PATH="$CARGO_TARGET_DIR/$RUST_TRIPLE/$CARGO_OUT_DIR_NAME/lib${LIB_NAME}.a"
  if [[ ! -f "$LIB_PATH" ]]; then
    echo "ERROR: Expected library not found at: $LIB_PATH"
    exit 1
  fi
  log "Library built: $LIB_PATH"

  # Validate that start_app symbol exists
  if ! nm -gU "$LIB_PATH" 2>/dev/null | grep -q "start_app"; then
    echo "WARNING: Library does not contain 'start_app' symbol."
    echo "Ensure lib.rs has #[cfg_attr(mobile, tauri::mobile_entry_point)] on pub fn run()"
  fi

  # Copy to Externals directory where Xcode expects it
  EXTERNALS_DIR="$APPLE_DIR/Externals/$ARCH/$CONFIGURATION"
  mkdir -p "$EXTERNALS_DIR"
  cp "$LIB_PATH" "$EXTERNALS_DIR/libapp.a"
  log "Copied to: $EXTERNALS_DIR/libapp.a"
done

log "Build complete."

# ---------------------------------------------------------------------------
# Ensure CFBundleURLTypes exists in Info.plist (Tauri CLI normally injects this,
# but our custom build pipeline bypasses that codegen step)
# ---------------------------------------------------------------------------
PLIST="$APPLE_DIR/multi_platform_app-mobile_iOS/Info.plist"
TAURI_CONF="$PROJECT_ROOT/src/app/mobile/src-tauri/tauri.conf.json"

# Read scheme from tauri.conf.json (single source of truth)
SCHEME=$(python3 -c "
import json, sys
conf = json.load(open('$TAURI_CONF'))
mobile = conf.get('plugins', {}).get('deep-link', {}).get('mobile', [])
print(mobile[0]['scheme'] if mobile else '')
" 2>/dev/null || echo "")

if [[ -n "$SCHEME" ]] && ! /usr/libexec/PlistBuddy -c "Print :CFBundleURLTypes" "$PLIST" &>/dev/null; then
  log "Injecting CFBundleURLTypes ($SCHEME) into Info.plist..."
  /usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes array" "$PLIST"
  /usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0 dict" "$PLIST"
  /usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLName string com.multiplatformapp.ios-test" "$PLIST"
  /usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes array" "$PLIST"
  /usr/libexec/PlistBuddy -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string $SCHEME" "$PLIST"
fi
