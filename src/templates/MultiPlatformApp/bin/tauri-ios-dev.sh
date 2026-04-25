#!/usr/bin/env bash
# tauri-ios-dev.sh — Replaces `pnpm tauri ios dev`
# Orchestrates: Vite dev server + xcodebuild + iOS simulator launch
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MOBILE_DIR="$PROJECT_ROOT/src/app/mobile"
XCODEPROJ="$MOBILE_DIR/src-tauri/gen/apple/multi_platform_app-mobile.xcodeproj"
SCHEME="multi_platform_app-mobile_iOS"
BUNDLE_ID="com.multiplatformapp.ios-test"
VITE_PORT=1430
DERIVED_DATA="$MOBILE_DIR/src-tauri/gen/apple/build"

# ---------------------------------------------------------------------------
# Cleanup on exit
# ---------------------------------------------------------------------------
VITE_PID=""
cleanup() {
  echo ""
  echo "[tauri-ios-dev] Shutting down..."
  if [[ -n "$VITE_PID" ]] && kill -0 "$VITE_PID" 2>/dev/null; then
    kill "$VITE_PID" 2>/dev/null || true
    wait "$VITE_PID" 2>/dev/null || true
  fi
  # Kill any remaining vite processes on our port
  lsof -ti:"$VITE_PORT" 2>/dev/null | xargs kill -9 2>/dev/null || true
  echo "[tauri-ios-dev] Done."
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Build shared libraries (must happen before Vite starts)
# ---------------------------------------------------------------------------
echo "[tauri-ios-dev] Building shared libraries..."
cd "$PROJECT_ROOT" && pnpm turbo build --filter=@multi-platform-app/shared --filter=@multi-platform-app/auth 2>&1 | tail -5
echo "[tauri-ios-dev] Shared libraries built."

# ---------------------------------------------------------------------------
# Detect local IP for TAURI_DEV_HOST
# ---------------------------------------------------------------------------
LOCAL_IP=""
# Try WiFi first, then Ethernet
for iface in en0 en1; do
  LOCAL_IP="$(ipconfig getifaddr "$iface" 2>/dev/null || echo "")"
  [[ -n "$LOCAL_IP" ]] && break
done
if [[ -z "$LOCAL_IP" ]]; then
  LOCAL_IP="127.0.0.1"
  echo "[tauri-ios-dev] WARNING: Could not detect local IP, using 127.0.0.1 (HMR may not work on simulator)"
fi
export TAURI_DEV_HOST="$LOCAL_IP"
echo "[tauri-ios-dev] TAURI_DEV_HOST=$LOCAL_IP"

# ---------------------------------------------------------------------------
# Kill any existing process on vite port
# ---------------------------------------------------------------------------
lsof -ti:"$VITE_PORT" 2>/dev/null | xargs kill -9 2>/dev/null || true

# ---------------------------------------------------------------------------
# Start Vite dev server in background
# ---------------------------------------------------------------------------
echo "[tauri-ios-dev] Starting Vite dev server on port $VITE_PORT..."
cd "$MOBILE_DIR"
pnpm dev &
VITE_PID=$!
cd "$PROJECT_ROOT"

# Wait for Vite to be ready
echo "[tauri-ios-dev] Waiting for Vite to start..."
for i in $(seq 1 30); do
  if lsof -ti:"$VITE_PORT" &>/dev/null; then
    echo "[tauri-ios-dev] Vite is ready on port $VITE_PORT"
    break
  fi
  if ! kill -0 "$VITE_PID" 2>/dev/null; then
    echo "ERROR: Vite process exited unexpectedly"
    exit 1
  fi
  sleep 1
done
if ! lsof -ti:"$VITE_PORT" &>/dev/null; then
  echo "ERROR: Vite did not start within 30 seconds"
  exit 1
fi

# ---------------------------------------------------------------------------
# Find an available iPhone simulator
# ---------------------------------------------------------------------------
echo "[tauri-ios-dev] Finding iPhone simulator..."
SIM_UDID="${SIMULATOR_UDID:-}"
if [[ -z "$SIM_UDID" ]]; then
  # Try to find a booted simulator first
  SIM_UDID="$(xcrun simctl list devices booted -j 2>/dev/null | \
    python3 -c "
import sys, json
data = json.loads(sys.stdin.read())
for runtime, devices in data.get('devices', {}).items():
    for d in devices:
        if 'iPhone' in d['name'] and d['state'] == 'Booted':
            print(d['udid'])
            sys.exit(0)
" 2>/dev/null || echo "")"

  # If no booted simulator, find an available one
  if [[ -z "$SIM_UDID" ]]; then
    SIM_UDID="$(xcrun simctl list devices available -j 2>/dev/null | \
      python3 -c "
import sys, json
data = json.loads(sys.stdin.read())
# Sort runtimes to prefer newer iOS versions
runtimes = sorted(data.get('devices', {}).keys(), reverse=True)
for runtime in runtimes:
    if 'iOS' not in runtime:
        continue
    for d in data['devices'][runtime]:
        if 'iPhone' in d['name']:
            print(d['udid'])
            sys.exit(0)
print('')
" 2>/dev/null || echo "")"
  fi
fi

if [[ -z "$SIM_UDID" ]]; then
  echo "ERROR: No iPhone simulator found. Install one via Xcode > Settings > Platforms."
  exit 1
fi
SIM_NAME="$(xcrun simctl list devices -j | python3 -c "
import sys, json
data = json.loads(sys.stdin.read())
udid = '$SIM_UDID'
for devices in data.get('devices', {}).values():
    for d in devices:
        if d['udid'] == udid:
            print(d['name'])
            sys.exit(0)
" 2>/dev/null || echo "$SIM_UDID")"
echo "[tauri-ios-dev] Using simulator: $SIM_NAME ($SIM_UDID)"

# ---------------------------------------------------------------------------
# Patch tauri.conf.json to use host IP instead of localhost
# (generate_context!() bakes devUrl at Rust compile time;
#  iOS simulator can't reach the Mac's localhost)
# ---------------------------------------------------------------------------
TAURI_CONF="$MOBILE_DIR/src-tauri/tauri.conf.json"
sed -i '' "s|http://localhost:$VITE_PORT|http://$LOCAL_IP:$VITE_PORT|g" "$TAURI_CONF"
echo "[tauri-ios-dev] Patched devUrl to http://$LOCAL_IP:$VITE_PORT"

# Restore tauri.conf.json on exit (append to existing trap)
restore_conf() {
  sed -i '' "s|http://$LOCAL_IP:$VITE_PORT|http://localhost:$VITE_PORT|g" "$TAURI_CONF" 2>/dev/null || true
}
trap 'cleanup; restore_conf' EXIT INT TERM

# ---------------------------------------------------------------------------
# Build with xcodebuild
# ---------------------------------------------------------------------------
echo "[tauri-ios-dev] Building iOS app (first build may take a few minutes)..."
xcodebuild \
  -project "$XCODEPROJ" \
  -scheme "$SCHEME" \
  -sdk iphonesimulator \
  -configuration debug \
  -destination "id=$SIM_UDID" \
  -derivedDataPath "$DERIVED_DATA"

# ---------------------------------------------------------------------------
# Boot simulator if needed
# ---------------------------------------------------------------------------
xcrun simctl boot "$SIM_UDID" 2>/dev/null || true
open -a Simulator

# ---------------------------------------------------------------------------
# Install and launch the app
# ---------------------------------------------------------------------------
APP_PATH="$(find "$DERIVED_DATA" -name "*.app" -ipath "*debug-iphonesimulator*" -not -path "*/Intermediates.noindex/*" | head -1)"
if [[ -z "$APP_PATH" ]]; then
  echo "ERROR: Built .app not found in $DERIVED_DATA"
  exit 1
fi

echo "[tauri-ios-dev] Installing app..."
xcrun simctl install "$SIM_UDID" "$APP_PATH"

echo "[tauri-ios-dev] Launching app..."
xcrun simctl launch "$SIM_UDID" "$BUNDLE_ID"

echo "[tauri-ios-dev] App is running. Vite HMR active at $LOCAL_IP:$VITE_PORT"
echo "[tauri-ios-dev] Press Ctrl+C to stop."

# Keep alive - wait for Vite process
wait "$VITE_PID"
