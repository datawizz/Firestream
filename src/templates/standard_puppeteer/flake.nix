{
  description = "example_shop web scraper development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        # Node.js with specific version
        nodejs = pkgs.nodejs_18;
        
        # Custom puppeteer wrapper that uses system chromium
        puppeteerEnv = pkgs.writeShellScriptBin "puppeteer-env" ''
          export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
          export PUPPETEER_EXECUTABLE_PATH=${pkgs.chromium}/bin/chromium
          exec "$@"
        '';
        
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Node.js and package managers
            nodejs
            nodePackages.npm
            nodePackages.typescript
            nodePackages.typescript-language-server
            
            # Browser for Puppeteer
            chromium
            
            # Virtual display for headless operation
            xvfb-run
            xorg.xorgserver
            
            # System dependencies for Chromium/Puppeteer
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
            
            # Additional rendering dependencies
            libGL
            libuuid
            libxkbcommon
            fontconfig
            freetype
            harfbuzz
            
            # Fonts for proper text rendering
            liberation_ttf
            dejavu_fonts
            noto-fonts
            noto-fonts-emoji
            
            # AWS CLI for S3 operations
            awscli2
            
            # Development tools
            git
            jq
            httpie
            
            # Custom scripts
            puppeteerEnv
            
            # Process management
            hivemind
            
            # Monitoring tools
            htop
            ncdu
          ];
          
          shellHook = ''
            # Set up Puppeteer to use system Chromium
            export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
            export PUPPETEER_EXECUTABLE_PATH=${pkgs.chromium}/bin/chromium
            export PUPPETEER_SKIP_DOWNLOAD=true
            
            # Set up display for headless operation
            export DISPLAY=:99
            
            # Font configuration for proper rendering
            export FONTCONFIG_PATH=${pkgs.fontconfig.out}/etc/fonts
            export FONTCONFIG_FILE=${pkgs.fontconfig.out}/etc/fonts/fonts.conf
            
            # Node.js memory settings
            export NODE_OPTIONS="--max-old-space-size=4096"
            
            # AWS config (if not already set)
            export AWS_REGION=''${AWS_REGION:-us-east-1}
            
            # Development settings
            export NODE_ENV=''${NODE_ENV:-development}
            export LOG_LEVEL=''${LOG_LEVEL:-debug}
            
            echo "example_shop scraper development environment"
            echo "Node.js: $(node --version)"
            echo "Chromium: ${pkgs.chromium}/bin/chromium"
            echo "AWS CLI: $(aws --version 2>/dev/null | head -1)"
            echo ""
            echo "Available commands:"
            echo "  npm install    - Install dependencies"
            echo "  npm run dev    - Run in development mode"
            echo "  npm run build  - Build for production"
            echo "  npm test       - Run tests"
            echo "  puppeteer-env  - Run commands with Puppeteer environment"
            echo ""
            
            # Install dependencies if not already installed
            if [ ! -d "node_modules" ]; then
              echo "Installing dependencies..."
              npm install
            fi
          '';
          
          # Environment variables
          AWS_SDK_LOAD_CONFIG = "1";
          WORKFLOW_ID = "ecommerce_product_scraper";
        };
        
        # Package the application
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "example_shop-scraper";
          version = "1.0.0";
          
          src = ./.;
          
          buildInputs = [ nodejs ];
          
          buildPhase = ''
            export HOME=$TMPDIR
            npm ci --production=false
            npm run build
          '';
          
          installPhase = ''
            mkdir -p $out/bin $out/lib
            
            # Copy built files
            cp -r dist $out/lib/
            cp -r node_modules $out/lib/
            cp package*.json $out/lib/
            
            # Create wrapper script
            cat > $out/bin/example_shop-scraper <<EOF
            #!${pkgs.bash}/bin/bash
            export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
            export PUPPETEER_EXECUTABLE_PATH=${pkgs.chromium}/bin/chromium
            export NODE_PATH=$out/lib/node_modules
            exec ${nodejs}/bin/node $out/lib/dist/index.js "\$@"
            EOF
            
            chmod +x $out/bin/example_shop-scraper
          '';
        };
        
        # Docker image (using nix2container would be better, but keeping it simple)
        packages.dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "local/example_shop-scraper";
          tag = "1.0.0";
          
          contents = with pkgs; [
            # Base system
            bashInteractive
            coreutils
            
            # Node.js runtime
            nodejs
            
            # Chromium
            chromium
            
            # Required libraries
            nss
            freetype
            harfbuzz
            fontconfig
            
            # AWS CLI for S3 operations
            awscli2
            
            # The application
            self.packages.${system}.default
          ];
          
          config = {
            Cmd = [ "${self.packages.${system}.default}/bin/example_shop-scraper" ];
            Env = [
              "NODE_ENV=production"
              "PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true"
              "PUPPETEER_EXECUTABLE_PATH=${pkgs.chromium}/bin/chromium"
              "WORKFLOW_ID=ecommerce_product_scraper"
            ];
            WorkingDir = "/app";
            
            # Health check
            Healthcheck = {
              Test = [ "CMD" "${nodejs}/bin/node" "-e" "console.log('OK')" ];
              Interval = "30s";
              Timeout = "10s";
              Retries = 3;
            };
          };
        };
      });
}