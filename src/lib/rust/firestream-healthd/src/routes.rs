//! HTTP route handling for firestream-healthd.
//!
//! Routes:
//! - `GET /healthz`              -> 200 "ok\n"            (liveness)
//! - `GET /readyz`               -> 200 "ready\n" or 503  (per-config readiness check)
//! - `GET /metadata`             -> streams /opt/firestream/metadata.json
//! - `GET /sbom[?format=cyclonedx|spdx]` -> streams the corresponding SBOM
//! - `GET /closure`              -> streams /opt/firestream/closure.json
//!
//! Anything else -> 404.

use std::io::Cursor;
use std::path::PathBuf;

use firestream_metadata::reader::{
    CLOSURE_FILE, CYCLONEDX_SBOM_FILE, METADATA_FILE, SPDX_SBOM_FILE,
};
use tiny_http::{Header, Response, StatusCode};

use crate::readiness::{ReadinessCheck, ReadinessOutcome};

/// Shared per-request context.
pub struct Ctx {
    pub metadata_path: PathBuf,
    pub readiness: ReadinessCheck,
}

/// Dispatch one request. Sends the response on the supplied `Request` and
/// returns. Any IO error encountered while writing the response is logged
/// (the caller doesn't get a way to recover; the request is already past).
pub fn handle(req: tiny_http::Request, ctx: &Ctx) {
    let method = req.method().clone();
    let url = req.url().to_string();
    let (path, query) = split_query(&url);

    tracing::debug!(method = %method, url = %url, "request");

    if method != tiny_http::Method::Get {
        respond(req, StatusCode(405), "text/plain; charset=utf-8", b"method not allowed\n");
        return;
    }

    match path {
        "/healthz" => {
            respond(req, StatusCode(200), "text/plain; charset=utf-8", b"ok\n");
        }
        "/readyz" => {
            match ctx.readiness.run() {
                ReadinessOutcome::Ready => {
                    respond(req, StatusCode(200), "text/plain; charset=utf-8", b"ready\n");
                }
                ReadinessOutcome::NotReady(reason) => {
                    let body = format!("not ready: {}\n", reason);
                    respond(
                        req,
                        StatusCode(503),
                        "text/plain; charset=utf-8",
                        body.as_bytes(),
                    );
                }
            }
        }
        "/metadata" => stream_file(req, &ctx.metadata_path.join(METADATA_FILE), METADATA_FILE),
        "/sbom" => {
            let format = query
                .as_deref()
                .and_then(|q| {
                    q.split('&')
                        .filter_map(|kv| kv.split_once('='))
                        .find(|(k, _)| *k == "format")
                        .map(|(_, v)| v.to_string())
                })
                .unwrap_or_else(|| "cyclonedx".to_string());
            match format.as_str() {
                "cyclonedx" => stream_file(
                    req,
                    &ctx.metadata_path.join(CYCLONEDX_SBOM_FILE),
                    CYCLONEDX_SBOM_FILE,
                ),
                "spdx" => stream_file(
                    req,
                    &ctx.metadata_path.join(SPDX_SBOM_FILE),
                    SPDX_SBOM_FILE,
                ),
                other => {
                    let body = format!("unsupported sbom format: {}\n", other);
                    respond(
                        req,
                        StatusCode(400),
                        "text/plain; charset=utf-8",
                        body.as_bytes(),
                    );
                }
            }
        }
        "/closure" => stream_file(req, &ctx.metadata_path.join(CLOSURE_FILE), CLOSURE_FILE),
        _ => {
            respond(req, StatusCode(404), "text/plain; charset=utf-8", b"not found\n");
        }
    }
}

fn split_query(url: &str) -> (&str, Option<&str>) {
    match url.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (url, None),
    }
}

fn stream_file(req: tiny_http::Request, path: &std::path::Path, name: &str) {
    match std::fs::read(path) {
        Ok(bytes) => respond(req, StatusCode(200), "application/json", &bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let body = format!("missing {}: {}\n", name, path.display());
            respond(
                req,
                StatusCode(503),
                "text/plain; charset=utf-8",
                body.as_bytes(),
            );
        }
        Err(e) => {
            let body = format!("error reading {}: {}\n", name, e);
            respond(
                req,
                StatusCode(500),
                "text/plain; charset=utf-8",
                body.as_bytes(),
            );
        }
    }
}

fn respond(req: tiny_http::Request, status: StatusCode, content_type: &str, body: &[u8]) {
    let header = match Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Ok(h) => h,
        Err(_) => {
            // Should never happen for static strings; fall back to no header.
            let resp = Response::new(
                status,
                Vec::new(),
                Cursor::new(body.to_vec()),
                Some(body.len()),
                None,
            );
            if let Err(e) = req.respond(resp) {
                tracing::warn!(error = %e, "failed to send response");
            }
            return;
        }
    };
    let resp = Response::new(
        status,
        vec![header],
        Cursor::new(body.to_vec()),
        Some(body.len()),
        None,
    );
    if let Err(e) = req.respond(resp) {
        tracing::warn!(error = %e, "failed to send response");
    }
}
