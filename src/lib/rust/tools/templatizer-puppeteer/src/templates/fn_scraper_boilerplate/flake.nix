{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        browser = pkgs.google-chrome;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nodejs_20
            browser
            # Core dependencies for Chrome on Debian
            alsa-lib at-spi2-atk cairo cups dbus gdk-pixbuf
            gtk3 libdrm libxkbcommon mesa nspr nss pango
            xorg.libX11 xorg.libXcomposite xorg.libXdamage
            xorg.libXrandr xorg.libXtst
          ];

          shellHook = ''
            export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
            export PUPPETEER_EXECUTABLE_PATH="${browser}/bin/google-chrome-stable"
          '';
        };
      }
    );
}
