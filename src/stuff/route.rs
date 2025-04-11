use crate::stuff::data_types::Message;
use crate::stuff::hook_types::HookRoot;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use tokio::sync::mpsc::Sender;

pub fn get_router(tx: Sender<Message>) -> Router {
    Router::new().route("/hook", post(handle_root)).with_state(tx)
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
