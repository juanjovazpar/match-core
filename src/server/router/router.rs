use axum::Router;

use super::healthz;
use super::order;
pub fn create() -> Router {
    Router::new()
        .merge(healthz::router())
        .merge(order::router())//.layer(middleware::from_fn(auth_middleware))
}