# ios.nix - iOS development tools (macOS only)
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Only enable on macOS
  isDarwin = pkgs.stdenv.isDarwin;

  iosDeps = if isDarwin then [
    pkgs.xcodegen          # Xcode project generator
    pkgs.libimobiledevice  # iOS device communication (used by Tauri)
  ] else [];

  envVars = if isDarwin then ''
    # iOS development paths
    export DEVELOPER_DIR="/Applications/Xcode.app/Contents/Developer"
    export SDKROOT="$DEVELOPER_DIR/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk"
  '' else "";

in
{
  packages = iosDeps;

  shellHook = ''
    ${envVars}
  '';

  profileScript = envVars;

  setupScript = pkgs.writeScript "setup-ios" ''
    #!${pkgs.bash}/bin/bash
    ${if isDarwin then ''
      echo "iOS tools configured"
      echo "Xcode must be installed separately from the App Store"
    '' else ''
      echo "iOS development not available on this platform"
    ''}
  '';

  apps = {};
}
