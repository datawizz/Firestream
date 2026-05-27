# Odoo Source Derivation
# Copyright Firestream. MIT License.
#
# Builds the Odoo source tree for a given major version. Relocated VERBATIM from
# the pre-migration flake.nix (odooSrc rev/sha256 map + odooSource mkDerivation)
# so that the evalContainer path passes a byte-identical `odooSource` derivation
# to the module via extraFactoryArgs.
#
# Usage:
#   odooSource = import ./source.nix { inherit pkgs; version = "18"; };

{ pkgs, version }:

let
  # Pinned source commits per Odoo branch
  odooSrc = pkgs.fetchFromGitHub {
    owner = "odoo";
    repo = "odoo";
    rev = {
      "15" = "3a28e5b0adbb36bdb1155a6854cdfbe4e7f9b187";  # 15.0 branch
      "16" = "f28eb1478fe47c4627fc3377fa505c78a6a7ea82";  # 16.0 branch
      "17" = "92e49e9c5087a4adedf89ff0fbf9462e8487a8da";  # 17.0 branch
      "18" = "f9e25ebf2f22b75ee74742a38182366a3b6a732c";  # 18.0 branch as of 2025-12-23
    }.${version};
    sha256 = {
      "15" = "sha256-nfdElQQIkV/zTVzozRt2CUaGmNPw2G3Zza+feTsBxTs=";
      "16" = "sha256-cXb7TEomwhe0zTEmDhoTy1UcDlQ5mTeyFAdsnhvhBlE=";
      "17" = "sha256-lc1mlEwK3Pgh9bD7lfIHqJ4zjUIheL7DCM8C7tdpf20=";
      "18" = "sha256-XK2+FwV9UDapbl037RR9wRhJZXGIKoHl/1gtB40+g1M=";
    }.${version};
  };
in
# Package Odoo source for installation
pkgs.stdenv.mkDerivation {
  pname = "odoo-source";
  version = "${version}.0";
  src = odooSrc;

  installPhase = ''
    mkdir -p $out/opt/odoo
    cp -r . $out/opt/odoo/
    chmod +x $out/opt/odoo/odoo-bin
  '';

  dontBuild = true;
  dontConfigure = true;
  dontFixup = true;
}
