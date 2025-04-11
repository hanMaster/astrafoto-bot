use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SendMessage {
    #[serde(rename = "chatId")]
    pub chat_id: String,
    pub message: String,
}
