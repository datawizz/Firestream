# Container image injection
# Copyright Firestream. MIT License.
#
# Pure function that merges container-registry-derived image overrides into a
# chart's `values` tree before serialisation.
#
# Inputs:
#   values         :: the chart-options-evaluated `values` attrset.
#   containerRefs  :: cfg._meta.containerRefs - an attrset of slots, each of
#                     shape { registry; repository; tag; componentPath; }
#                     declared in bin/nix/firestream/charts/eval-chart.nix.
#                     `componentPath` is the list of attribute names locating
#                     the chart's image slot in the values tree (e.g.
#                     [ "image" ] for the airflow top-level image; [ "redis"
#                     "image" ] for a redis subchart image).
#
# Semantics:
#   - Slots with `componentPath == []` are RECORDED in the manifest (the
#     emitter reads cfg._meta.containerRefs directly) but contribute no value
#     overlay - they exist purely as catalogue entries.
#   - For each slot with a non-empty `componentPath`, we set the image triple
#     { registry; repository; tag; } at that path, then merge with the
#     existing values via `recursiveUpdate values overlay`. Per nixpkgs
#     `recursiveUpdate` semantics, leaves on the RHS win - i.e. the injected
#     image overrides the chart-defaults-shaped value at that path.
#   - HOWEVER, an explicit non-null user value already present in `values`
#     should win over the injection ("user can still pin a different tag").
#     To achieve that without re-implementing recursiveUpdate, we compute the
#     injected overlay FIRST, then `recursiveUpdate overlay values` so the
#     existing values' leaves take precedence over the injected ones at
#     overlapping paths. Paths that didn't exist before injection now carry
#     the injected triple verbatim.
#   - Null leaves on the injected slot are STRIPPED before merging so a slot
#     that only specifies a tag (registry=null, repository=null, tag="x")
#     does not silently overwrite an existing image.repository with null.
#
# Output: the updated `values` attrset, suitable for `to-values-yaml`.
{ lib }:

{ values, containerRefs }:

let
  # Strip null leaves from an attrset recursively. Mirrors the null-strip
  # logic in `lib/to-values-yaml.nix` and `lib/to-chart-manifest.nix`.
  stripNulls = lib.filterAttrsRecursive (_: v: v != null);

  # For one slot, build a single-path overlay of the image triple at its
  # componentPath. Returns `{}` if componentPath is empty.
  slotOverlay = _slotName: slot:
    if slot.componentPath == []
    then {}
    else
      let
        triple = stripNulls {
          inherit (slot) registry repository tag;
        };
      in
      # An empty triple (all three null) contributes nothing.
      if triple == {}
      then {}
      else lib.setAttrByPath slot.componentPath triple;

  # All slot overlays merged together. `foldl' recursiveUpdate {}` is safe
  # here because distinct slots write to distinct componentPaths in practice;
  # if two slots happen to target the same path the later one wins, matching
  # the natural attrset-merge convention.
  injectedOverlay =
    lib.foldl' lib.recursiveUpdate {}
      (lib.mapAttrsToList slotOverlay containerRefs);
in
# `recursiveUpdate a b` merges b into a; on leaf collisions the value from b
# wins. We want existing user values in `values` to win over `injectedOverlay`,
# so `values` is the RHS.
lib.recursiveUpdate injectedOverlay values
