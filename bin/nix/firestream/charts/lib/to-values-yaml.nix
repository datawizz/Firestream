# values.yaml emitter
# Copyright Firestream. MIT License.
#
# Serialises a Nix attrset (the merged `values` tree produced by a chart's
# options modules) into a Helm-faithful values.yaml derivation.
#
# Signature: { pkgs, lib }: valuesTree: <derivation producing values.yaml>
#
# Two empirically-verified decisions for this nixpkgs:
#   1. We use `(pkgs.formats.yaml {}).generate` rather than
#      `lib.generators.toYAML`. On this nixpkgs, `lib.generators.toYAML` emits
#      single-line JSON (e.g. `{"flag":true,"q":"100m"}`) which, while valid
#      YAML, is unreadable and diff-hostile. `pkgs.formats.yaml` emits faithful
#      block YAML: `flag: true`, `np: '30080'` (numeric-looking strings quoted),
#      `q: 100m` (unquoted), `empty: []` (preserved).
#   2. We strip `null` values ONLY. A `null` in the values tree is the Nix
#      idiom for "option unset / omit this key" — emitting it would override an
#      upstream chart default with an explicit null. We deliberately do NOT
#      strip `""`, `[]`, or `{}`: those are meaningful values upstream charts
#      may compare against (e.g. `image.tag: ""` ⇒ use appVersion).
{ pkgs, lib }:

valuesTree:

let
  # Strip ONLY null leaves; keep "", [], {} intact.
  cleaned = lib.filterAttrsRecursive (_: v: v != null) valuesTree;
in
(pkgs.formats.yaml { }).generate "values.yaml" cleaned
