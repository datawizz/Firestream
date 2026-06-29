# seaweedfs-weed Nix package
# Copyright Firestream. MIT License.
#
# Builds the SeaweedFS `weed` binary from a pinned upstream GitHub source.
# SeaweedFS is Apache-2.0, single Go binary, S3-compatible object store —
# the Firestream local/object-store backend (9th supported app).
#
# Pinned to the exact commit vendored at `_WIP/seaweedfs`:
#   seaweedfs/seaweedfs @ 5797fb24ec8974c7cd510225853069ce6b5d8bbd
#   (describe: 4.36-17-g5797fb24e), go.mod requires Go >= 1.25.5.
#
# Build notes:
#   - No build tags  => leveldb2 filer backend (zero-config single-node default),
#     CGO-free (no RocksDB / sqlite / tikv).
#   - `subPackages = [ "weed" ]` builds only the CLI entrypoint.
#   - The repo's pinned nixpkgs (release-25.11) ships pkgs.go == 1.25.5, which
#     satisfies go.mod, so the default toolchain is used (no `go` override).

{ pkgs, lib }:

pkgs.buildGoModule {
  pname = "seaweedfs-weed";
  version = "4.36-17-g5797fb24e";

  src = pkgs.fetchFromGitHub {
    owner = "seaweedfs";
    repo = "seaweedfs";
    rev = "5797fb24ec8974c7cd510225853069ce6b5d8bbd";
    hash = "sha256-pAoyzrKwCYYHp5cumW2NoflZ3o5Y8gbpFUOzTSKp+v8=";
  };

  vendorHash = "sha256-peRhKuZ1D+y8Uhw1+P8Ogc1HrOh1/kYVd29lR89+rIo=";

  subPackages = [ "weed" ];

  # No build tags => leveldb2 filer, CGO-free single-node default.
  tags = [ ];
  env.CGO_ENABLED = "0";
  ldflags = [ "-s" "-w" ];
  doCheck = false;

  # The upstream SeaweedFS image installs the binary at /usr/bin/weed, and the
  # forked Helm chart's all-in-one Deployment invokes that ABSOLUTE path
  # (`/usr/bin/weed server ...`). buildGoModule installs only to $out/bin, which
  # the container surfaces on PATH as /bin/weed — so the chart's hardcoded
  # /usr/bin/weed is missing and the pod crashes with exit 127. Expose the binary
  # at usr/bin/weed too so the firestream image is a drop-in for the chart
  # without patching the vendored templates. (buildLayeredImage surfaces this
  # store path's usr/bin tree at the image root; see containers/base.nix
  # imageContents.)
  postInstall = ''
    mkdir -p $out/usr/bin
    ln -s $out/bin/weed $out/usr/bin/weed
  '';

  meta = {
    description = "SeaweedFS `weed` binary (S3-compatible object store)";
    homepage = "https://github.com/seaweedfs/seaweedfs";
    license = lib.licenses.asl20;
    mainProgram = "weed";
  };
}
