//! Minimal `helm-manager` binary.
//!
//! Historically this binary listed available embedded charts. Chart discovery
//! has moved to the `firestream-charts` crate (driven by the flake-emitted
//! index at `/opt/firestream/charts`). This binary now just verifies the
//! helm CLI is reachable; it's kept as a smoke-test entry point so the crate
//! still produces a binary target.

use helm_manager::helm_client::HelmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let _helm = HelmClient::new()?;
    println!("helm-manager: helm binary located on PATH.");
    println!("Chart discovery now lives in `firestream-charts`.");

    Ok(())
}
