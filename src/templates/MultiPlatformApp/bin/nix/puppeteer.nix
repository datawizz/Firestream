# puppeteer.nix - Puppeteer/Chromium environment configuration
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Platform detection - Chromium is only available on Linux
  isLinux = pkgs.stdenv.isLinux;

  # Dependencies required by Chromium/Puppeteer (Linux only)
  chromiumDeps = if isLinux then with pkgs; [
    # Core browser
    chromium

    # System libraries required by Chromium
    glib
    nss
    nspr
    atk
    at-spi2-atk
    cups
    dbus
    expat
    libdrm
    xorg.libX11
    xorg.libXcomposite
    xorg.libXdamage
    xorg.libXext
    xorg.libXfixes
    xorg.libXrandr
    xorg.libxcb
    mesa
    pango
    cairo
    alsa-lib
    at-spi2-core
    xorg.libXcursor
    xorg.libXi
    xorg.libXrender
    xorg.libXScrnSaver
    xorg.libXtst

    # Additional dependencies
    fontconfig
    freetype
    harfbuzz
    libGL
    libuuid
    libxkbcommon

    # Virtual display for headless operation
    xorg.xorgserver
    xvfb-run

    # Font packages for proper text rendering
    liberation_ttf
    dejavu_fonts
    noto-fonts
    noto-fonts-cjk-sans
    noto-fonts-color-emoji

    # Additional font rendering support
    fontconfig.lib
  ] else [];  # Empty on macOS - Chromium not available

  # Puppeteer environment setup (Linux only)
  puppeteerEnv = if isLinux then ''
    # Skip Puppeteer's Chromium download
    export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
    export PUPPETEER_SKIP_DOWNLOAD=true
    export PUPPETEER_EXECUTABLE_PATH=${pkgs.chromium}/bin/chromium

    # Set up display for headless operation
    export DISPLAY=:99

    # Chromium flags for container environment
    export CHROME_BIN=${pkgs.chromium}/bin/chromium
    export CHROMIUM_FLAGS="--no-sandbox --disable-setuid-sandbox --disable-dev-shm-usage"

    # Font configuration
    export FONTCONFIG_PATH=${pkgs.fontconfig.out}/etc/fonts
    export FONTCONFIG_FILE=${pkgs.fontconfig.out}/etc/fonts/fonts.conf
  '' else "";

  # Wrapper script for running Puppeteer (Linux only)
  puppeteerWrapper = if isLinux then pkgs.writeScriptBin "puppeteer-run" ''
    #!${pkgs.bash}/bin/bash
    ${puppeteerEnv}
    exec "$@"
  '' else null;

  # Wrapper script for running Puppeteer with Xvfb (Linux only)
  xvfbPuppeteerWrapper = if isLinux then pkgs.writeScriptBin "xvfb-puppeteer-run" ''
    #!${pkgs.bash}/bin/bash
    ${puppeteerEnv}
    # Run with xvfb-run for proper headless rendering
    exec ${pkgs.xvfb-run}/bin/xvfb-run \
      -a \
      --server-args="-screen 0 1920x1080x24 -ac -nolisten tcp -dpi 96" \
      "$@"
  '' else null;

in
{
  # Packages to be included (filter out nulls for macOS)
  packages = chromiumDeps
    ++ pkgs.lib.optional (puppeteerWrapper != null) puppeteerWrapper
    ++ pkgs.lib.optional (xvfbPuppeteerWrapper != null) xvfbPuppeteerWrapper;

  # Shell hook for development
  shellHook = if isLinux then ''
    ${puppeteerEnv}
  '' else "";

  # Profile script for persistent environment
  profileScript = puppeteerEnv;

  # Setup script for initialization
  setupScript = pkgs.writeScript "setup-puppeteer" ''
    #!${pkgs.bash}/bin/bash
    set -e

    ${if isLinux then ''
      echo "Setting up Puppeteer environment..."
      echo "Chromium binary: ${pkgs.chromium}/bin/chromium"
      echo "Puppeteer will use system Chromium (download skipped)"

      # Verify Chromium is accessible
      if [ -x "${pkgs.chromium}/bin/chromium" ]; then
        echo "Chromium is installed and accessible"
      else
        echo "Chromium not found or not executable"
        exit 1
      fi
    '' else ''
      echo "Puppeteer/Chromium not available on macOS"
      echo "Consider using Playwright with its bundled browsers instead"
    ''}
  '';

  # Apps for Puppeteer module (Linux only)
  apps = if isLinux then {
    puppeteer-run = {
      type = "app";
      program = "${puppeteerWrapper}/bin/puppeteer-run";
    };
    xvfb-puppeteer-run = {
      type = "app";
      program = "${xvfbPuppeteerWrapper}/bin/xvfb-puppeteer-run";
    };
  } else {};
}
