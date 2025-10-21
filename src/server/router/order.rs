use axum::{routing::post, Router, Json, http::StatusCode};
use serde::Deserialize;

use exchain_commons::structs::{order::{Price, Quantity}};

use super::paths;

#[derive(Deserialize)]
struct OrderRequest {
    quantity: Quantity,
    price: Price,
}

pub fn router() -> Router {
    Router::new().route(paths::ORDERS, post(post_handler))
}

// (owner: Uuid, quantity: Quantity, price: Price, side: Side, mode: Mode)
async fn post_handler(
    Json(payload): Json<OrderRequest>,
) -> Result<&'static str, (StatusCode, String)> {
    if payload.price <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Price must be positive".into()));
    }
    if payload.quantity <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Quantity must be greater than 0".into()));
    }

    // println!("Creating order: {} {} @ {}", payload.side, payload.quantity, payload.price);

    Ok("Order received")
}