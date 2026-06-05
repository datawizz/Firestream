# Offline subchart vendoring
# Copyright Firestream. MIT License.
#
# Produces a directory of fully-vendored Helm subcharts intended to be dropped
# into a parent chart's `charts/`. Everything is sourced from the IN-REPO
# maintained Bitnami fork (`src/charts/bitnami`, FORK.md commit af55f1f7…) — we
# deliberately do NOT fetch from the network and do NOT reuse the upstream pin
# in nix/flake-modules/lib/shell-env.nix. CLAUDE.md: "use the local fork in
# src/charts/, not upstream Bitnami."
#
# Signature:
#   { pkgs, lib }:
#     { subcharts, chartsSrcDir ? <repo>/src/charts/bitnami }:
#       <derivation: a dir to drop into a chart's charts/>
#
#   subcharts :: list of { name :: str; }   (e.g. [ { name = "common"; } ... ])
#
# CRITICAL — nested `common`. The fork's postgresql and redis charts each
# declare a `common` dependency in Chart.yaml and ship NO vendored charts/ dir;
# their templates call `common.*` helpers. Helm resolves a subchart's own
# dependencies from THAT subchart's charts/ dir, so a flat top-level vendor
# fails with "template not found". We therefore recursively vendor: for every
# vendored chart, we read its Chart.yaml dependency names and, for any dep that
# exists as a sibling chart in chartsSrcDir and is not already present under the
# chart's own charts/, we copy that dep chart into <chart>/charts/<dep>,
# recursing into it. Result layout for [common postgresql redis]:
#
#   <out>/common/
#   <out>/postgresql/                 + <out>/postgresql/charts/common/
#   <out>/redis/                      + <out>/redis/charts/common/
{ pkgs, lib }:

{
  subcharts,
  chartsSrcDir ? ../../../../../src/charts/bitnami,
}:

let
  # Source path for a single named subchart in the fork.
  srcOf = name: chartsSrcDir + "/${name}";

  requested = map (s: s.name) subcharts;

  # A clean copy of the entire fork chart tree, used as the recursion source so
  # the build can resolve arbitrary nested dependency charts by name.
  chartsSrc = lib.cleanSource chartsSrcDir;
in
pkgs.runCommand "vendored-subcharts"
{
  inherit requested;
  src = chartsSrc;
} ''
  set -euo pipefail
  mkdir -p "$out"

  # vendor_recursive <dest_charts_dir> <chart_name>
  #   Copies $src/<chart_name> into <dest_charts_dir>/<chart_name>, then for each
  #   dependency declared in that chart's Chart.yaml that exists as a sibling
  #   chart in $src and is not already vendored under its charts/, vendors it
  #   recursively into <chart>/charts/.
  vendor_recursive() {
    local dest_charts="$1"
    local chart="$2"
    local chart_dir="$dest_charts/$chart"

    if [ -d "$chart_dir" ]; then
      return 0
    fi
    if [ ! -d "$src/$chart" ]; then
      echo "vendor-subcharts: chart '$chart' not found in source tree $src" >&2
      exit 1
    fi

    mkdir -p "$dest_charts"
    cp -r --no-preserve=mode "$src/$chart" "$chart_dir"

    local chart_yaml="$chart_dir/Chart.yaml"
    [ -f "$chart_yaml" ] || return 0

    # Extract dependency names from the dependencies: block. Bitnami Chart.yaml
    # lists each dep as `- name: <dep>` (or `name: <dep>` within the item). We
    # isolate the block between the top-level `dependencies:` key and the next
    # top-level key (a line starting in column 0 that is not `dependencies:`),
    # then pull every `name:` value out of it. This is a superset-safe
    # heuristic: only deps that ALSO exist as sibling charts in $src get
    # vendored, so any stray `name:` value is ignored because it won't be a
    # chart dir. Implemented with sed+grep (portable; no gawk-only features).
    local deps
    deps=$(sed -n '/^dependencies:/,/^[^[:space:]#-]/p' "$chart_yaml" \
      | grep -oE 'name:[[:space:]]*[A-Za-z0-9_.-]+' \
      | sed -E 's/name:[[:space:]]*//' || true)

    local dep
    for dep in $deps; do
      if [ -d "$src/$dep" ] && [ ! -d "$chart_dir/charts/$dep" ]; then
        vendor_recursive "$chart_dir/charts" "$dep"
      fi
    done
  }

  for chart in $requested; do
    vendor_recursive "$out" "$chart"
  done
''
