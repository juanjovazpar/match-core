/* use axum::{
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(mut req: RequestPartsExt, next: Next) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    match token {
        Some(t) if t == "Bearer my_secret_token" => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
} */