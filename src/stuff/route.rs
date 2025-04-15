use crate::config::config;
use crate::stuff::data_types::Message;
use crate::stuff::hook_types::HookRoot;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{middleware, Json, Router};
use log::info;
use tokio::sync::mpsc::Sender;

pub fn get_router(tx: Sender<Message>) -> Router {
    Router::new()
        .route("/hook", post(handle_root))
        .route_layer(middleware::from_fn(auth_guard))
        .with_state(tx)
}

#[axum::debug_handler]
async fn handle_root(
    State(tx): State<Sender<Message>>,
    Json(m): Json<HookRoot>,
) -> impl IntoResponse {
    let res = tx.send(m.into()).await;
    if let Err(e) = res {
        println!("Failed to send message {e}",);
    }

    "Ok".into_response()
}

pub async fn auth_guard(headers: HeaderMap, req: Request<Body>, next: Next) -> impl IntoResponse {
    let token_header = headers.get("Authorization");
    let token = match token_header {
        Some(header) => header.to_str().unwrap_or("Bearer empty").to_string(),
        None => return Err((StatusCode::UNAUTHORIZED, "Not authenticated!").into_response()),
    };

    let token = token.split_whitespace().skip(1).next().unwrap();
    info!("Token: {token}");
    info!("Secret: {}", config().SECRET_TOKEN);

    if token.ne(&config().SECRET_TOKEN) {
        return Err((StatusCode::UNAUTHORIZED, "Not authenticated!!!!").into_response());
    }

    Ok(next.run(req).await)
}
