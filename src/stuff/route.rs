use crate::stuff::data_types::{Message, ReceivedMessage};
use crate::stuff::wa_types::RootMsg;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use tokio::sync::mpsc::Sender;

pub fn get_router(tx: Sender<Message>) -> Router {
    Router::new().route("/", post(handle_root)).with_state(tx)
}

async fn handle_root(
    State(tx): State<Sender<Message>>,
    Json(mut m): Json<RootMsg>,
) -> impl IntoResponse {
    let msg = match m.body.message_data.type_message.as_ref() {
        "imageMessage" => Message::Image(ReceivedMessage {
            chat_id: m.body.sender_data.chat_id,
            customer_name: m.body.sender_data.sender_name,
            message: m
                .body
                .message_data
                .file_message_data
                .take()
                .unwrap()
                .download_url,
        }),
        "textMessage" => Message::Text(ReceivedMessage {
            chat_id: m.body.sender_data.chat_id,
            customer_name: m.body.sender_data.sender_name,
            message: m
                .body
                .message_data
                .text_message_data
                .take()
                .unwrap()
                .text_message,
        }),
        _ => Message::Empty,
    };

    let res = tx.send(msg).await;
    if let Err(e) = res {
        println!("Failed to send message {e}",);
    }

    "Ok".into_response()
}
