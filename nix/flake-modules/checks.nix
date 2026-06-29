# Checks flake-module
# Copyright Firestream. MIT License.
#
# Ports the legacy flake's 12 checks (the Firestream module-system test suite),
# sourced from bin/nix/firestream/tests.
#
# Gated to Linux: the test suite (SBOM/source archiving, container closures)
# pulls Linux-only packages (e.g. glibc), so it cannot evaluate on Darwin.
# Container builds are Linux-only too, so this matches reality and keeps
# `nix flake check` green on all systems (Darwin simply has no checks).
# NOTE: the legacy flake defined these for all systems via forAllSystems, which
# made `nix flake check --all-systems` fail on Darwin; gating fixes that.
{ inputs, ... }: {
  perSystem = { pkgs, system, lib, ... }:
    let
      isLinux = pkgs.stdenv.hostPlatform.isLinux;
      tests = import ../../bin/nix/firestream/tests {
        inherit pkgs system;
        inherit (inputs) fenix crane;
      };
    in
    {
      checks = lib.optionalAttrs isLinux {
        firestream-tests = tests.all;

        # Individual test modules (for granular testing)
        firestream-log-tests = tests.logTests;
        firestream-validations-tests = tests.validationsTests;
        firestream-fs-tests = tests.fsTests;
        firestream-os-tests = tests.osTests;
        firestream-net-tests = tests.netTests;
        firestream-service-tests = tests.serviceTests;
        firestream-file-tests = tests.fileTests;
        firestream-persistence-tests = tests.persistenceTests;
        firestream-integration-tests = tests.integrationTests;
        firestream-config-tests = tests.configTests;
        firestream-container-tests = tests.containerTests;
        firestream-postgresql-env-alias-tests = tests.postgresqlEnvAliasTests;
        firestream-seaweedfs-chart-values-tests = tests.seaweedfsChartValuesTests;
      };
    };
}
