# Base (un-overlaid) Helm chart builder
# Copyright Firestream. MIT License.
#
# Exposes a forked Bitnami chart as a STANDALONE, self-contained Nix derivation
# that renders against the chart's OWN bundled defaults — NO Firestream values
# overlay, NO image injection, NO chart-manifest. A downstream consumer can run:
#
#   helm install <release> /nix/store/...-<app>-base-chart
#   helm template <release> /nix/store/...-<app>-base-chart
#
# directly on the store path, because `$out` IS the chart directory (Chart.yaml
# at `$out/Chart.yaml`, templates at `$out/templates`, vendored deps under
# `$out/charts/`). This is the structural difference from eval-chart.nix, which
# nests the chart under `$out/chart` and renders against an overlaid values.yaml.
#
# It reuses eval-chart's subchart vendoring (./vendor-subcharts.nix), doc
# filtering (drop README.md / CHANGELOG.md), and Chart.lock removal. The build
# is GATED by an offline `helm template <releaseName> "$out"` with NO `-f` flag,
# so the chart renders against its native `values.yaml`.
#
# Signature:
#   { pkgs, lib }:
#     { name, chartSrc, subcharts ? [],
#       helm ? pkgs.kubernetes-helm, releaseName ? name }:
#       <derivation: a self-contained chart dir>
{ pkgs, lib }:

{
  name,
  chartSrc,
  subcharts ? [],
  helm ? pkgs.kubernetes-helm,
  releaseName ? name,
}:

let
  vendoredCharts = (import ./vendor-subcharts.nix { inherit pkgs lib; }) {
    inherit subcharts;
  };

  # Filter heavy docs out of the chart source before it enters the store: each
  # of README.md / CHANGELOG.md plays no role in rendering. Keep everything else
  # verbatim (templates, values, etc).
  filteredChartSrc = lib.cleanSourceWith {
    src = lib.cleanSource chartSrc;
    name = "${name}-base-chart-src";
    filter = path: _type:
      let base = baseNameOf path;
      in base != "README.md" && base != "CHANGELOG.md";
  };

  # Some charts declare a kubeVersion constraint; pass a satisfying value (the
  # cluster targets K8s 1.31, matching the k8s-openapi v1_31 feature flag).
  # Detection is line-anchored (see eval-chart.nix for rationale).
  hasKubeVersion =
    let chartYaml = chartSrc + "/Chart.yaml";
    in builtins.pathExists chartYaml
       && lib.any
            (line: lib.hasPrefix "kubeVersion:" line)
            (lib.splitString "\n" (builtins.readFile chartYaml));

  kubeVersionFlag = lib.optionalString hasKubeVersion ''--kube-version "1.31.0"'';
in
pkgs.runCommand "${name}-base-chart"
  {
    nativeBuildInputs = [ helm ];
    passthru = { inherit name releaseName; };
  } ''
  set -euo pipefail

  # 1. $out IS the chart directory (no nesting under $out/chart).
  mkdir -p "$out"
  cp -r --no-preserve=mode ${filteredChartSrc}/. "$out/"

  # 2. Drop the vendored subcharts into the chart's charts/ dir.
  mkdir -p "$out/charts"
  cp -r --no-preserve=mode ${vendoredCharts}/. "$out/charts/"

  # 3. Remove Chart.lock — we never run `helm dependency`; a stale lock only
  #    produces an "out of sync" warning.
  rm -f "$out/Chart.lock"

  # 4. Build-gate: render the chart fully offline against its NATIVE defaults.
  #    No -f flag, no --dependency-update. This proves the un-overlaid base
  #    chart renders standalone.
  export HOME="$TMPDIR"
  helm template ${lib.escapeShellArg releaseName} "$out" ${kubeVersionFlag} > /dev/null
''
