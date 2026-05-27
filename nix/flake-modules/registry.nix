# Container registry flake-module
# Copyright Firestream. MIT License.
#
# Declares two perSystem options that container flake-modules populate and that
# aggregate.nix / flake.lib read back. flake-parts merges these option
# definitions across all perSystem modules, so each container module can
# contribute its own entry without coordination.
#
#   firestreamContainers.<name> = <evalContainer result>   (full module result)
#   firestreamImages.<name>     = { dockerImage; eval; options; }  (consumer API)
#
# `lazyAttrsOf raw` keeps values lazy (so referencing one container's attrNames
# never forces another's heavy derivation closure) and untyped.
{ ... }: {
  perSystem = { lib, ... }: {
    options.firestreamContainers = lib.mkOption {
      type = lib.types.lazyAttrsOf lib.types.raw;
      default = { };
      description = "Per-system registry of evaluated Firestream containers.";
    };

    options.firestreamImages = lib.mkOption {
      type = lib.types.lazyAttrsOf lib.types.raw;
      default = { };
      description = "Per-system registry of Firestream image consumer APIs.";
    };
  };
}
