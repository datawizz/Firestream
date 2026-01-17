# bin/nix/firestream/modules/darwin.nix
# Darwin-specific shell configuration for nixpkgs 25.11+
#
# In nixpkgs 25.11, the Darwin stdenv automatically sets DEVELOPER_DIR and
# SDKROOT to a Nix-bundled Apple SDK. This breaks tools like xcrun that
# expect the system Xcode installation.
#
# This module provides shell hooks to override those paths with the actual
# Xcode installation on the system.
{ pkgs, lib }:

let
  isDarwin = pkgs.stdenv.isDarwin;
  xcodeDir = "/Applications/Xcode.app/Contents/Developer";
  xcodeBetaDir = "/Applications/Xcode-beta.app/Contents/Developer";

  # Detect Xcode at evaluation time (impure but necessary)
  developerDir =
    if builtins.pathExists xcodeDir then xcodeDir
    else if builtins.pathExists xcodeBetaDir then xcodeBetaDir
    else null;

  envVars = lib.optionalString isDarwin ''
    # Use system Xcode (nixpkgs 25.11 sets Nix SDK by default)
    ${lib.optionalString (developerDir != null) ''
      export DEVELOPER_DIR="${developerDir}"
    ''}

    # Clear Nix SDK variables that conflict with system tools
    unset SDKROOT
    unset NIX_LDFLAGS
    unset NIX_CFLAGS_COMPILE

    # Use system clang
    export CC="/usr/bin/clang"
    export CXX="/usr/bin/clang++"
  '';

in {
  inherit isDarwin developerDir;
  inherit envVars;
  shellHook = envVars;
}
