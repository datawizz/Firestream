{ system ? builtins.currentSystem, config }:
let
  # Import nixpkgs and get lib
  pkgs = import <nixpkgs> { inherit system; };
  inherit (pkgs) lib;

  # Import <nixpkgs/nixos> so we can build a NixOS configuration
  nixos = import <nixpkgs/nixos> {
    inherit system;
    configuration = {};
  };
  inherit (nixos) config modules options;

  # Helper function to safely read files
  safeReadFile = path:
    if builtins.pathExists path
    then builtins.readFile path
    else "";

  # Function to create the base system configuration
  makeBaseSystem = { config, extraModules ? [] }:
    let
      # Parse the config argument
      settings = config;
      # Extract common settings
      hostname = settings.hostname or "prodbox";
      build_type = settings.build_type or "gcp";

      # Extract credentials if they exist
      serviceAccountContent = settings.google_application_credentials or "";

      # Common configuration for all build types
      baseModules = [
        ./modules/common.nix
        {
          _module.args = {
            inherit hostname;
            inherit serviceAccountContent;
          };
        }
      ];

      # Combine with any extra modules
      allModules = baseModules ++ extraModules;
    in {
      inherit hostname build_type;

      # Instead of calling "lib.nixosSystem" or passing "modules",
      # we use `import <nixpkgs/nixos>` with a configuration attribute:
      system = import <nixpkgs/nixos> {
        inherit system;
        configuration = {
          # We set stateVersion for forward-compatibility
          system.stateVersion = "24.11";

          # Then import all the modules
          imports = allModules;
        };
      };
    };

in rec {
  # GCP Image builder
  gcpImage =
    let
      baseSystem = makeBaseSystem {
        inherit config;
        extraModules = [
          <nixpkgs/nixos/modules/virtualisation/google-compute-image.nix>
          ./modules/gcp-specific.nix
        ];
      };
    in
      baseSystem.system.config.system.build.googleComputeImage;

  # QEMU Image builder
  qemuImage =
    let
      baseSystem = makeBaseSystem {
        inherit config;
        extraModules = [
          <nixpkgs/nixos/modules/profiles/qemu-guest.nix>
          ./modules/qemu-specific.nix
        ];
      };
    in
      baseSystem.system.config.system.build.qcow2;
}
