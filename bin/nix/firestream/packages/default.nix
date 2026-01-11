# Firestream Packages Index
# Copyright Firestream. Apache-2.0 License.
#
# This module provides custom packages built from source within the Firestream
# repository. These packages are used as dependencies in the container module system.
#
# mkRustPackage is provided by the Rust module (Fenix + Crane) for incremental
# builds with dependency caching.

{ pkgs, lib, mkRustPackage }:

{
  # wait-for-port: Rust-based port availability checker
  # Usage: wait-for-port PORT [--host HOST] [--state inuse|free] [--timeout SECS]
  wait-for-port = import ./wait-for-port.nix { inherit pkgs lib mkRustPackage; };
}
