use axum::Router;

use super::healthz;

pub fn create() -> Router {
    Router::new()
        .merge(healthz::router())
}