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

    options.firestreamCharts = lib.mkOption {
      type = lib.types.lazyAttrsOf lib.types.raw;
      default = { };
      description = "Per-system registry of evaluated Firestream charts.";
    };

    options.firestreamChartImages = lib.mkOption {
      type = lib.types.lazyAttrsOf lib.types.raw;
      default = { };
      description = "Per-system registry of chart consumer-override APIs.";
    };

    # Named stack definitions: ordered lists of chart names that compose a
    # deployable bundle (e.g. `dev` => the full local data-platform stack).
    # Phase 3 (Agent C): aggregate.nix reads this and surfaces it through
    # `index.json` so the Rust deploy layer can resolve "stack named X" to a
    # deterministic deploy order. Stack entries may reference charts that have
    # not yet been registered in `firestreamCharts` (Wave-3 charts land later);
    # the symlink farm just won't include them until they do.
    options.firestreamStacks = lib.mkOption {
      type = lib.types.attrsOf (lib.types.listOf lib.types.str);
      default = { };
      description = "Per-system registry of named chart stacks (ordered chart-name lists).";
    };
  };
}
