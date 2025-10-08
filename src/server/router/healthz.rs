use axum::{routing::get, Router};

use crate::{server::channel::get_sender, Message};
use super::paths;

pub fn router() -> Router {
    Router::new().route(paths::HEALTH, get(get_handler))
}

async fn get_handler() -> &'static str {
    let tx = get_sender();

    let _ = tx.send(Message::new(String::from("hello engine"))).await;

    "Healthy!"
}