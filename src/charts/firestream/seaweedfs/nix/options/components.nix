# SeaweedFS chart options: distributed-component on/off switches.
#
# The upstream chart defaults master/volume/filer to enabled=true. Those are
# the DISTRIBUTED topology workloads (separate StatefulSets). When
# `allInOne.enabled=true` the chart does NOT auto-disable them, so leaving the
# defaults would render BOTH the single all-in-one pod AND the distributed
# master/volume/filer StatefulSets (a conflicting, duplicate topology).
#
# Firestream runs the single-pod all-in-one topology, so the overlay turns
# these off (see ../default.nix). Each is modelled minimally with freeform
# passthrough so the rest of each component's value block still flows through
# from upstream values.yaml (the all-in-one pod reads e.g. `.Values.master.port`,
# `.Values.filer.port`, which remain defined even when `enabled=false`).
{ lib, ... }:

let
  inherit (lib) mkOption types;

  enabledOnly = desc: mkOption {
    default = null;
    description = desc;
    type = types.nullOr (types.submodule {
      freeformType = types.attrsOf types.anything;
      options.enabled = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Render this component's workload.";
      };
    });
  };
in {
  options.seaweedfs.master = enabledOnly "Distributed master StatefulSet (disabled under all-in-one).";
  options.seaweedfs.volume = enabledOnly "Distributed volume StatefulSet (disabled under all-in-one).";
  options.seaweedfs.filer = enabledOnly "Distributed filer StatefulSet (disabled under all-in-one).";
}
