{ pkgs }:

let
  lib = pkgs.lib;
  firestream = import ../. { inherit pkgs; };

  # Import individual test modules
  logTests = import ./test-log.nix { inherit pkgs firestream; };
  validationsTests = import ./test-validations.nix { inherit pkgs firestream; };
  fsTests = import ./test-fs.nix { inherit pkgs firestream; };
  osTests = import ./test-os.nix { inherit pkgs firestream; };
  netTests = import ./test-net.nix { inherit pkgs firestream; };
  serviceTests = import ./test-service.nix { inherit pkgs firestream; };
  fileTests = import ./test-file.nix { inherit pkgs firestream; };
  persistenceTests = import ./test-persistence.nix { inherit pkgs firestream; };
  integrationTests = import ./test-integration.nix { inherit pkgs firestream; };
  configTests = import ./test-config.nix { inherit pkgs firestream; };
  containerTests = import ./test-containers.nix { inherit pkgs firestream; };

in {
  # Individual test derivations
  inherit logTests validationsTests fsTests osTests netTests serviceTests fileTests persistenceTests integrationTests;
  inherit configTests containerTests;

  # All tests combined
  all = pkgs.runCommand "firestream-all-tests" {
    buildInputs = [
      logTests validationsTests fsTests osTests netTests serviceTests
      fileTests persistenceTests integrationTests configTests containerTests
    ];
  } ''
    echo "================================================"
    echo "All Firestream module tests passed!"
    echo "================================================"
    echo "Log tests:          PASSED"
    echo "Validations tests:  PASSED"
    echo "FS tests:           PASSED"
    echo "OS tests:           PASSED"
    echo "Net tests:          PASSED"
    echo "Service tests:      PASSED"
    echo "File tests:         PASSED"
    echo "Persistence tests:  PASSED"
    echo "Integration tests:  PASSED"
    echo "Config tests:       PASSED"
    echo "Container tests:    PASSED"
    echo "================================================"
    touch $out
  '';
}
