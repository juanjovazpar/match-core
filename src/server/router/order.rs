use axum::{routing::post, Router};

use super::paths;

pub fn router() -> Router {
    Router::new().route(paths::ORDERS, post(post_handler))
}

async fn post_handler() -> &'static str {
    "Creating order!"
}