# bin/nix/firestream/node/mkNodePackage.nix
# Node.js Package Builder
#
# A wrapper around pkgs.buildNpmPackage that provides:
# - pnpm as primary package manager with npm fallback
# - Built-in TypeScript support with auto-detection
# - Consistent wrapper script generation
# - Source filtering for clean builds
#
# Usage:
#   mkNodePackage {
#     pname = "my-server";
#     version = "1.0.0";
#     src = ./.;
#     npmDepsHash = "sha256-...";
#     # Optional: Override TypeScript detection
#     # hasTypeScript = true;
#   }

{ pkgs, lib }:

let
  # Detect package manager from source
  # Returns "pnpm" if pnpm-lock.yaml exists, otherwise "npm"
  detectPackageManager = src:
    if builtins.pathExists "${src}/pnpm-lock.yaml"
    then "pnpm"
    else "npm";

  # Detect TypeScript from source
  # Returns true if tsconfig.json exists
  hasTypeScript = src:
    builtins.pathExists "${src}/tsconfig.json";

  # Default Node.js version
  defaultNodejs = pkgs.nodejs_22;

  # Source filter to exclude build artifacts and dependencies
  sourceFilter = src:
    lib.cleanSourceWith {
      inherit src;
      filter = path: type:
        let
          name = baseNameOf path;
        in
        # Exclude common build artifacts and dependencies
        !(name == "node_modules" ||
          name == "dist" ||
          name == ".git" ||
          name == ".pnpm-store" ||
          name == "coverage" ||
          lib.hasSuffix ".log" name);
    };

in {
  # Export detection utilities
  inherit detectPackageManager hasTypeScript;

  # Main package builder
  mkNodePackage = {
    pname,
    version,
    src,
    npmDepsHash ? lib.fakeHash,

    # Optional: Override Node.js version
    nodejs ? defaultNodejs,

    # Optional: Override package manager detection
    packageManager ? null,

    # Optional: Override TypeScript detection
    typescript ? null,

    # Optional: Custom build command (defaults based on TypeScript detection)
    buildPhase ? null,

    # Optional: Custom install phase
    installPhase ? null,

    # Optional: Entry point for the binary wrapper
    # If not specified, uses package.json "main" or "dist/index.js"
    entryPoint ? null,

    # Optional: Binary name (defaults to pname)
    binName ? null,

    # Optional: Additional build inputs
    buildInputs ? [],

    # Optional: Additional native build inputs
    nativeBuildInputs ? [],

    # Optional: Environment variables for build
    env ? {},

    # Optional: Additional npm flags
    npmFlags ? [],

    # Optional: Skip binary wrapper generation
    skipBinWrapper ? false,

    # Optional: Metadata
    meta ? {},

    # Pass through other args
    ...
  }@args:
    let
      # Determine package manager
      detectedPackageManager =
        if packageManager != null
        then packageManager
        else detectPackageManager src;

      # Determine if TypeScript project
      isTypeScript =
        if typescript != null
        then typescript
        else hasTypeScript src;

      # Clean the source
      cleanedSrc = sourceFilter src;

      # Determine the final entry point
      finalEntryPoint =
        if entryPoint != null
        then entryPoint
        else if isTypeScript
        then "dist/index.js"
        else "index.js";

      # Determine binary name
      finalBinName = if binName != null then binName else pname;

      # Default build phase based on TypeScript detection
      defaultBuildPhase =
        if isTypeScript
        then ''
          runHook preBuild

          # TypeScript compilation
          if [ -f "package.json" ] && grep -q '"build"' package.json; then
            echo "Running npm build script..."
            npm run build
          elif command -v tsc &> /dev/null; then
            echo "Running tsc directly..."
            tsc
          else
            echo "Warning: TypeScript project but no build command found"
          fi

          runHook postBuild
        ''
        else ''
          runHook preBuild
          # No build step needed for plain JavaScript
          runHook postBuild
        '';

      # Default install phase
      defaultInstallPhase = ''
        runHook preInstall

        mkdir -p $out/lib/${pname}

        # Copy application files
        ${if isTypeScript then ''
          # TypeScript: copy compiled output and dependencies
          cp -r dist node_modules package.json $out/lib/${pname}/
        '' else ''
          # JavaScript: copy source and dependencies
          cp -r . $out/lib/${pname}/
          rm -rf $out/lib/${pname}/node_modules/.cache 2>/dev/null || true
        ''}

        ${lib.optionalString (!skipBinWrapper) ''
          # Create binary wrapper
          mkdir -p $out/bin
          cat > $out/bin/${finalBinName} << 'EOF'
#!/bin/sh
exec ${nodejs}/bin/node $out/lib/${pname}/${finalEntryPoint} "$@"
EOF
          chmod +x $out/bin/${finalBinName}
        ''}

        runHook postInstall
      '';

    in pkgs.buildNpmPackage ({
      inherit pname version nodejs npmDepsHash;
      src = cleanedSrc;

      buildInputs = buildInputs;
      nativeBuildInputs = nativeBuildInputs ++ lib.optionals isTypeScript [
        pkgs.nodePackages.typescript
      ];

      buildPhase = if buildPhase != null then buildPhase else defaultBuildPhase;
      installPhase = if installPhase != null then installPhase else defaultInstallPhase;

      # Pass through npm flags
      npmFlags = npmFlags;

      # Merge environment
      env = {
        # Disable Puppeteer download (common issue)
        PUPPETEER_SKIP_CHROMIUM_DOWNLOAD = "true";
        # Skip optional dependencies that may fail on different platforms
        npm_config_optional = "false";
      } // env;

      meta = {
        description = args.description or "Firestream ${pname}";
        homepage = args.homepage or "https://github.com/Cogent-Creation-Co/Firestream";
        license = lib.licenses.asl20;
        maintainers = [ "Firestream Team" ];
        mainProgram = finalBinName;
        platforms = lib.platforms.unix;
      } // meta;

    } // (builtins.removeAttrs args [
      "pname" "version" "src" "npmDepsHash" "nodejs" "packageManager"
      "typescript" "buildPhase" "installPhase" "entryPoint" "binName"
      "buildInputs" "nativeBuildInputs" "env" "npmFlags" "skipBinWrapper" "meta"
      "description" "homepage"
    ]));
}
