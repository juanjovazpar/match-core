mod paths;
mod healthz;
mod order;

use axum::Router;

pub fn create() -> Router {
    Router::new()
        .merge(healthz::router())
        .merge(order::router())//.layer(middleware::from_fn(auth_middleware))
}