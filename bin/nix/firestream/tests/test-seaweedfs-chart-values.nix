# Regression guard: SeaweedFS chart Firestream-default overlay + image injection.
# Copyright Firestream. MIT License.
#
# SeaweedFS is the 9th Firestream supported app and the default local S3 object
# store. Unlike the Bitnami charts it is NOT driven by perContainerHelpers /
# libhelpers / path-remaps; the `weed` binary is invoked directly by the chart's
# `command:` block. This test pins the two things Firestream actually overrides
# on top of the upstream chart, plus the image-injection seam:
#
#   1. The chart's `nix/default.nix` overlay turns the all-in-one + S3 topology
#      ON (`allInOne.enabled`, `allInOne.s3.enabled`, S3 on 8333, auth on, the
#      default `firestream` bucket, and the `firestream`/`firestream-secret`
#      credentials). We evaluate the REAL chart option modules (the same ones
#      `evalChart` consumes on the build path) and assert the projected
#      `config.seaweedfs.values` tree.
#
#   2. The Firestream image-injector (`charts/lib/inject-container-images.nix`)
#      writes a `firestream-seaweedfs` image triple at `componentPath = [ "image" ]`.
#      The chart overlay deliberately leaves `image` null (so injection fills it),
#      so the injected-image assertion is exercised here against the SAME pure
#      injector the flake-module uses — proving the `[ "image" ]` slot lands
#      `firestream-seaweedfs` at `.Values.image.repository`. (The full rendered.yaml
#      with the injected image is additionally verified by `nix build .#seaweedfs-chart`.)
#
# Tracking the source option modules (not a copy) means a future overlay change
# that drops the all-in-one toggle, moves S3 off 8333, or breaks the image slot
# fails this build.

{ pkgs, firestream }:

let
  lib = pkgs.lib;

  # Minimal stub of the `_meta`/`values` option slots that `evalChart`
  # (bin/nix/firestream/charts/eval-chart.nix) declares. Declaring them as
  # freeform attrs lets the chart's `nix/default.nix` set `_meta.deployment`,
  # `_meta.lifecycle`, and project `values` without dragging in the full
  # eval-chart engine.
  metaValuesStub = { lib, ... }: {
    options.seaweedfs._meta = lib.mkOption {
      type = lib.types.attrsOf lib.types.anything;
      default = { };
    };
    options.seaweedfs.values = lib.mkOption {
      type = lib.types.attrsOf lib.types.anything;
      default = { };
    };
  };

  chartTypes = import ../charts/lib/types { inherit lib; };

  evaled = lib.evalModules {
    modules = [
      metaValuesStub
      ../../../../src/charts/firestream/seaweedfs/nix/default.nix
    ];
    specialArgs = { inherit pkgs lib chartTypes; };
  };

  values = evaled.config.seaweedfs.values;

  # Exercise the real image injector exactly as the flake-module does: the
  # canonical `seaweedfs` slot writes the triple at `componentPath = [ "image" ]`.
  injectContainerImages = import ../charts/lib/inject-container-images.nix { inherit lib; };
  injected = injectContainerImages {
    inherit values;
    containerRefs = {
      seaweedfs = {
        registry = "docker.io";
        repository = "firestream-seaweedfs";
        tag = "4.36-17-g5797fb24e";
        componentPath = [ "image" ];
      };
    };
  };

  # --- assertions (eval-time; a failure aborts the build) ---
  firstBucket = builtins.head values.allInOne.s3.createBuckets;

  checks = [
    { name = "allInOne.enabled";            ok = values.allInOne.enabled == true; }
    { name = "allInOne.s3.enabled";         ok = values.allInOne.s3.enabled == true; }
    { name = "allInOne.s3.createBuckets[0]=firestream"; ok = firstBucket.name == "firestream"; }
    { name = "s3.port=8333";                ok = values.s3.port == 8333; }
    { name = "s3.enableAuth";               ok = values.s3.enableAuth == true; }
    { name = "s3.credentials.admin.accessKey=firestream"; ok = values.s3.credentials.admin.accessKey == "firestream"; }
    { name = "s3.credentials.admin.secretKey=firestream-secret"; ok = values.s3.credentials.admin.secretKey == "firestream-secret"; }
    # the chart overlay leaves image unset (null leaf, stripped) so injection fills it
    { name = "image unset pre-injection";   ok = !(values ? image); }
    { name = "injected image.repository=firestream-seaweedfs"; ok = injected.image.repository == "firestream-seaweedfs"; }
    { name = "injected image.tag set";      ok = injected.image.tag == "4.36-17-g5797fb24e"; }
    { name = "injected image.registry set"; ok = injected.image.registry == "docker.io"; }
    # injection must not disturb the S3 port
    { name = "s3.port survives injection";  ok = injected.s3.port == 8333; }
  ];

  failures = builtins.filter (c: !c.ok) checks;
  report = lib.concatMapStringsSep "\n" (c: "  ${if c.ok then "ok" else "FAIL"}: ${c.name}") checks;
in
assert (
  if failures == [ ]
  then true
  else builtins.throw "test-seaweedfs-chart-values FAILED:\n${report}"
);
pkgs.runCommand "test-seaweedfs-chart-values" { } ''
  echo "=== SeaweedFS chart Firestream overlay + image injection ==="
  cat <<'EOF'
${report}
EOF
  echo "=== SeaweedFS chart values OK ==="
  touch $out
''
