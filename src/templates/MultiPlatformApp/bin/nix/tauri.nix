# tauri.nix - Tauri development dependencies
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Tauri dependencies
  tauriDeps = with pkgs; [
    # Build tools
    pkg-config

    # Linux-specific
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    webkitgtk_4_1
    gtk3
    cairo
    gdk-pixbuf
    glib
    dbus
    openssl
    librsvg

  # macOS-specific (frameworks are now provided by default SDK in nixpkgs 25.11+)
  ];

  # Environment variables
  envVars = if pkgs.stdenv.isLinux then ''
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    export PKG_CONFIG_PATH="${pkgs.lib.makeLibraryPath tauriDeps}/pkgconfig:$PKG_CONFIG_PATH"
  '' else "";

in
{
  packages = tauriDeps;

  shellHook = ''
    ${envVars}
  '';

  profileScript = envVars;

  setupScript = pkgs.writeScript "setup-tauri" ''
    #!${pkgs.bash}/bin/bash
    echo "Tauri dependencies configured"
  '';

  apps = {};
}
