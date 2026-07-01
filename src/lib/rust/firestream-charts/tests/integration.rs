//! Integration tests for the v1 chart manifest + index schema and runtime reader.

use std::fs;
use std::path::PathBuf;

use firestream_charts::{ChartManifest, Charts, Index};
use tempfile::TempDir;

const AIRFLOW_FIXTURE: &str = include_str!("fixtures/airflow.json");

/// Test 1: a real Agent A/B sample manifest deserializes into [`ChartManifest`].
#[test]
fn parses_valid_airflow_manifest() {
    let m: ChartManifest =
        serde_json::from_str(AIRFLOW_FIXTURE).expect("airflow fixture must parse");

    assert_eq!(m.schema_version, "1");
    assert_eq!(m.name, "airflow");
    assert_eq!(m.chart, "airflow");
    assert_eq!(m.version, "24.2.4");

    assert_eq!(m.release.namespace.as_deref(), Some("airflow"));
    assert_eq!(m.release.release_name.as_deref(), Some("airflow"));
    assert!(m.release.create_namespace);

    assert_eq!(
        m.bundle.chart_path,
        PathBuf::from("/nix/store/fake-airflow-chart-bundle/chart")
    );
    assert_eq!(
        m.bundle.values_path,
        PathBuf::from("/nix/store/fake-airflow-chart-bundle/values.yaml")
    );
    assert_eq!(
        m.bundle.rendered_path,
        PathBuf::from("/nix/store/fake-airflow-chart-bundle/rendered.yaml")
    );

    assert!(m.deployment.atomic);
    assert!(m.deployment.wait);
    assert!(m.deployment.wait_for_jobs);
    assert!(!m.deployment.force_upgrade);
    assert!(!m.deployment.hooks_disabled);
    assert!(!m.deployment.skip_crds);
    assert_eq!(m.deployment.timeout, "10m");

    assert_eq!(m.lifecycle.depends_on, vec!["postgresql", "redis"]);
    assert!(m.lifecycle.last_breaking_version.is_none());

    let airflow_img = m.images.get("airflow").expect("airflow image slot present");
    assert_eq!(airflow_img.component_path, vec!["image"]);
    assert_eq!(airflow_img.repository.as_deref(), Some("firestream-airflow"));
    assert_eq!(airflow_img.tag.as_deref(), Some("3.0.3"));
    assert!(airflow_img.registry.is_none());

    assert!(m.provenance.flake_revision.is_none());
    assert!(m.provenance.nixpkgs_revision.is_none());
}

/// Test 2: round-trip (parse -> serialize -> parse) is lossless for the
/// non-null fields. `skip_serializing_if = "Option::is_none"` means we never
/// re-emit nulls that the Nix side stripped.
#[test]
fn manifest_round_trip_is_lossless() {
    let original: ChartManifest = serde_json::from_str(AIRFLOW_FIXTURE).unwrap();
    let serialized = serde_json::to_string(&original).expect("must serialize");
    let reparsed: ChartManifest = serde_json::from_str(&serialized).expect("must reparse");

    // Spot-check that key fields survived the round trip.
    assert_eq!(reparsed.schema_version, original.schema_version);
    assert_eq!(reparsed.name, original.name);
    assert_eq!(reparsed.version, original.version);
    assert_eq!(reparsed.release.namespace, original.release.namespace);
    assert_eq!(reparsed.bundle.chart_path, original.bundle.chart_path);
    assert_eq!(reparsed.deployment.timeout, original.deployment.timeout);
    assert_eq!(reparsed.lifecycle.depends_on, original.lifecycle.depends_on);
    assert_eq!(
        reparsed.images.get("airflow").map(|i| i.tag.clone()),
        original.images.get("airflow").map(|i| i.tag.clone())
    );

    // Round-trip a second time and require byte-equality with the first
    // serialization — this proves the serializer is idempotent on the
    // canonical Rust shape.
    let serialized2 = serde_json::to_string(&reparsed).unwrap();
    assert_eq!(serialized, serialized2);
}

/// Test 3: minimal manifest (most fields absent) parses cleanly thanks to
/// `#[serde(default)]` everywhere optional.
#[test]
fn minimal_manifest_defaults_cleanly() {
    let minimal = serde_json::json!({
        "schemaVersion": "1",
        "name": "tinychart",
        "chart": "tinychart",
        "version": "0.0.1",
        "provenance": {}
        // No release, no bundle, no deployment, no lifecycle, no images.
    });

    let m: ChartManifest =
        serde_json::from_value(minimal).expect("minimal manifest must parse");

    assert_eq!(m.name, "tinychart");
    assert_eq!(m.version, "0.0.1");

    // Defaults
    assert!(m.release.namespace.is_none());
    assert!(m.release.release_name.is_none());
    assert!(!m.release.create_namespace);
    assert_eq!(m.bundle.chart_path, PathBuf::new());
    assert_eq!(m.deployment.timeout, "300s");
    assert!(!m.deployment.atomic);
    assert!(m.lifecycle.depends_on.is_empty());
    assert!(m.images.is_empty());
    assert!(m.provenance.flake_revision.is_none());
}

/// Test 3b: an index missing the `stacks` or `charts` key still parses.
#[test]
fn minimal_index_defaults_cleanly() {
    let minimal = serde_json::json!({
        "schemaVersion": "1"
    });

    let idx: Index = serde_json::from_value(minimal).expect("minimal index must parse");
    assert_eq!(idx.schema_version, "1");
    assert!(idx.charts.is_empty());
    assert!(idx.stacks.is_empty());
}

/// Test 3c: an index carrying the Phase-5 `baseCharts` block parses and
/// exposes it. Also proves backward compatibility: an index WITHOUT the key
/// yields an empty `base_charts` map (covered by `minimal_index_defaults_cleanly`).
#[test]
fn index_with_base_charts_parses() {
    let with_base = serde_json::json!({
        "schemaVersion": "1",
        "charts": {
            "postgresql": { "manifestPath": "postgresql/chart-manifest.json" }
        },
        "baseCharts": {
            "postgresql": { "baseChartPath": "postgresql-base" }
        },
        "stacks": { "dev": ["postgresql"] }
    });

    let idx: Index = serde_json::from_value(with_base).expect("index with baseCharts must parse");
    assert_eq!(idx.schema_version, "1");
    assert_eq!(
        idx.base_charts
            .get("postgresql")
            .map(|e| e.base_chart_path.as_path()),
        Some(PathBuf::from("postgresql-base").as_path())
    );

    // Round-trips back out under the camelCase keys.
    let serialized = serde_json::to_string(&idx).unwrap();
    assert!(serialized.contains("baseCharts"));
    assert!(serialized.contains("baseChartPath"));

    // An index that predates Phase 5 (no baseCharts key) still parses, with an
    // empty map.
    let legacy: Index = serde_json::from_value(serde_json::json!({ "schemaVersion": "1" }))
        .expect("legacy index without baseCharts must parse");
    assert!(legacy.base_charts.is_empty());
}

/// Build an on-disk charts farm in `tmp` with one valid chart (`airflow`)
/// and an index that registers it.
fn write_airflow_farm(tmp: &TempDir) {
    let root = tmp.path();
    fs::create_dir_all(root.join("airflow")).unwrap();
    fs::write(root.join("airflow/chart-manifest.json"), AIRFLOW_FIXTURE).unwrap();

    let index = serde_json::json!({
        "schemaVersion": "1",
        "charts": {
            "airflow": { "manifestPath": "airflow/chart-manifest.json" }
        },
        "stacks": {
            "dev": ["airflow"]
        }
    });
    fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&index).unwrap(),
    )
    .unwrap();
}

/// Test 4: `Charts::open` + `Charts::get` resolves a manifest correctly.
#[test]
fn open_and_get_resolves_manifest() {
    let tmp = TempDir::new().unwrap();
    write_airflow_farm(&tmp);

    let charts = Charts::open(tmp.path()).expect("Charts::open must succeed");

    assert_eq!(charts.index().schema_version, "1");
    assert_eq!(charts.list(), vec!["airflow"]);
    assert_eq!(charts.list_stacks(), vec!["dev"]);

    let airflow = charts.get("airflow").expect("get airflow");
    assert_eq!(airflow.name, "airflow");
    assert_eq!(airflow.version, "24.2.4");

    // Second call should hit the cache (no easy way to assert that without
    // inspecting internals; just ensure correctness).
    let airflow2 = charts.get("airflow").expect("second get");
    assert_eq!(airflow2.name, airflow.name);
}

/// Test 5: `Charts::stack` skips entries not registered in the index
/// without erroring. Per Agent C: stacks may reference charts that
/// Wave-3 hasn't wired up yet.
#[test]
fn stack_skips_unregistered_charts() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Same airflow fixture, but the stack also references postgresql and
    // a not-yet-wired chart `kafka` — both should be skipped.
    fs::create_dir_all(root.join("airflow")).unwrap();
    fs::write(root.join("airflow/chart-manifest.json"), AIRFLOW_FIXTURE).unwrap();

    let index = serde_json::json!({
        "schemaVersion": "1",
        "charts": {
            "airflow": { "manifestPath": "airflow/chart-manifest.json" }
        },
        "stacks": {
            "dev": ["postgresql", "redis", "airflow", "kafka"]
        }
    });
    fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&index).unwrap(),
    )
    .unwrap();

    let charts = Charts::open(tmp.path()).unwrap();
    let resolved = charts.stack("dev").expect("stack resolves without erroring");

    // Only airflow is in the index, so we should get exactly one manifest.
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].name, "airflow");
}

/// Test 6: missing `index.json` returns the typed error.
#[test]
fn missing_index_errors_cleanly() {
    let tmp = TempDir::new().unwrap();
    let err = Charts::open(tmp.path()).expect_err("must fail with no index.json");
    match err {
        firestream_charts::Error::IndexNotFound(p) => {
            assert!(p.ends_with("index.json"));
        }
        other => panic!("expected IndexNotFound, got {:?}", other),
    }
}

/// Test 7: requesting an unknown chart returns ChartNotInIndex.
#[test]
fn get_unknown_chart_errors() {
    let tmp = TempDir::new().unwrap();
    write_airflow_farm(&tmp);
    let charts = Charts::open(tmp.path()).unwrap();

    let err = charts
        .get("does-not-exist")
        .expect_err("must error on unknown chart");
    assert!(matches!(
        err,
        firestream_charts::Error::ChartNotInIndex(_)
    ));
}

/// Test 8: requesting an unknown stack returns StackNotInIndex.
#[test]
fn unknown_stack_errors() {
    let tmp = TempDir::new().unwrap();
    write_airflow_farm(&tmp);
    let charts = Charts::open(tmp.path()).unwrap();

    let err = charts
        .stack("nope")
        .expect_err("must error on unknown stack");
    assert!(matches!(
        err,
        firestream_charts::Error::StackNotInIndex(_)
    ));
}

/// Test 9: a chart registered in the index whose manifest file is missing
/// returns ManifestNotFound.
#[test]
fn manifest_file_missing_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let index = serde_json::json!({
        "schemaVersion": "1",
        "charts": {
            "airflow": { "manifestPath": "airflow/chart-manifest.json" }
        },
        "stacks": {}
    });
    fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&index).unwrap(),
    )
    .unwrap();
    // NOTE: We intentionally don't create the airflow subdir.

    let charts = Charts::open(tmp.path()).unwrap();
    let err = charts.get("airflow").expect_err("must fail");
    assert!(matches!(
        err,
        firestream_charts::Error::ManifestNotFound(_)
    ));
}
