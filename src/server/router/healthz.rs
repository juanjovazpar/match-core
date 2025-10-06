use axum::{routing::get, Router};

use super::paths;

pub fn router() -> Router {
    Router::new().route(paths::HEALTH, get(get_handler))
}

async fn get_handler() -> &'static str {
    "Healthy!"
}