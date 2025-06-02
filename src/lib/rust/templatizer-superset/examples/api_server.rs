//! Example API server for dashboard generation
//! 
//! Run with: cargo run --example api_server

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use templatizer_superset::{DashboardConfig, SupersetDashboardBuilder};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    superset_url: String,
}

#[derive(Deserialize)]
struct GenerateQuery {
    /// Whether to upload directly to Superset
    upload: Option<bool>,
    /// Whether to return the ZIP file
    download: Option<bool>,
}

#[derive(Serialize)]
struct GenerateResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    zip_data: Option<String>, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    dashboard_id: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    superset_url: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment
    dotenv::dotenv().ok();

    let superset_url = std::env::var("SUPERSET_URL")
        .unwrap_or_else(|_| "http://localhost:8088".to_string());

    let state = Arc::new(AppState {
        superset_url: superset_url.clone(),
    });

    // Build the router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/v1/generate", post(generate_dashboard))
        .route("/api/v1/validate", post(validate_config))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        );

    let addr = "0.0.0.0:3000";
    info!("Starting API server on {}", addr);
    info!("Superset URL: {}", superset_url);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Superset Dashboard Generator API\n\nPOST /api/v1/generate - Generate dashboard from JSON\nPOST /api/v1/validate - Validate dashboard configuration\nGET /health - Health check"
}

async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        superset_url: state.superset_url.clone(),
    })
}

async fn generate_dashboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GenerateQuery>,
    Json(config): Json<DashboardConfig>,
) -> Result<Json<GenerateResponse>, AppError> {
    info!("Generating dashboard: {}", config.dashboard.title);

    // Create builder from config
    let builder = SupersetDashboardBuilder::from_config(config)
        .map_err(|e| AppError::BadRequest(e.to_string()))?
        .with_database_from_env();

    // Generate ZIP
    let zip_data = builder.generate_zip()
        .map_err(|e| AppError::Internal(format!("Failed to generate ZIP: {}", e)))?;

    let mut response = GenerateResponse {
        success: true,
        message: "Dashboard generated successfully".to_string(),
        zip_data: None,
        dashboard_id: None,
    };

    // Upload if requested
    if params.upload.unwrap_or(false) {
        info!("Uploading dashboard to Superset");
        match builder.upload_to_superset(&state.superset_url).await {
            Ok(_) => {
                response.message = "Dashboard generated and uploaded successfully".to_string();
                response.dashboard_id = Some(builder.config().dashboard.uuid.to_string());
            }
            Err(e) => {
                return Err(AppError::Internal(format!("Upload failed: {}", e)));
            }
        }
    }

    // Include ZIP data if requested
    if params.download.unwrap_or(true) {
        response.zip_data = Some(general_purpose::STANDARD.encode(&zip_data));
    }

    Ok(Json(response))
}

async fn validate_config(
    Json(config): Json<DashboardConfig>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Validating dashboard configuration");

    // Try to create builder and validate
    let mut config = config;
    config.initialize_uuids();
    config.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "dashboard": {
            "title": config.dashboard.title,
            "slug": config.dashboard.slug,
            "uuid": config.dashboard.uuid.to_string(),
        },
        "database": {
            "name": config.database.name,
            "connection": config.get_safe_sqlalchemy_uri(),
        },
        "datasets": config.datasets.len(),
        "charts": config.charts.len(),
        "layout_rows": config.layout.rows.len(),
    })))
}

// Error handling
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: error_message.clone(),
            details: None,
        });

        error!("API error: {}", error_message);
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        // Simple test that health response can be created
        let response = HealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            superset_url: "http://test:8088".to_string(),
        };
        assert_eq!(response.status, "ok");
    }
}
