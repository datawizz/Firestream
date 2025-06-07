use crate::controllers::tasks;
use crate::middlewares::auth::auth;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Initializes the application's routes.
///
/// This function maps paths (e.g. "/greet") and HTTP methods (e.g. "GET") to functions in [`crate::controllers`] as well as includes middlewares defined in [`crate::middlewares`] into the routing layer (see [`axum::Router`]).
pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);
    Router::new()
        .route("/tasks", post(tasks::create))
        .route("/tasks", put(tasks::create_batch))
        .route("/tasks/{id}", delete(tasks::delete))
        .route("/tasks/{id}", put(tasks::update))
        .route_layer(middleware::from_fn_with_state(
            shared_app_state.clone(),
            auth,
        ))
        .route("/tasks", get(tasks::read_all))
        .route("/tasks/{id}", get(tasks::read_one))
        .with_state(shared_app_state)
}
