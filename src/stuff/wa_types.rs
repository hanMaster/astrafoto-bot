use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextMessageData {
    #[serde(rename = "textMessage")]
    pub text_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageData {
    #[serde(rename = "typeMessage")]
    pub type_message: String,
    #[serde(rename = "textMessageData")]
    pub text_message_data: Option<TextMessageData>,
    #[serde(rename = "fileMessageData")]
    pub file_message_data: Option<FileMessageData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SenderData {
    #[serde(rename = "chatId")]
    pub chat_id: String,
    #[serde(rename = "chatName")]
    pub chat_name: String,
    pub sender: String,
    #[serde(rename = "senderName")]
    pub sender_name: String,
    #[serde(rename = "senderContactName")]
    pub sender_contact_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceData {
    #[serde(rename = "idInstance")]
    pub id_instance: i64,
    pub wid: String,
    #[serde(rename = "typeInstance")]
    pub type_instance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "typeWebhook")]
    pub type_webhook: String,
    #[serde(rename = "instanceData")]
    pub instance_data: InstanceData,
    pub timestamp: i64,
    #[serde(rename = "idMessage")]
    pub id_message: String,
    #[serde(rename = "senderData")]
    pub sender_data: SenderData,
    #[serde(rename = "messageData")]
    pub message_data: MessageData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootMsg {
    #[serde(rename = "receiptId")]
    pub receipt_id: i64,
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMessageData {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    pub caption: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "jpegThumbnail")]
    pub jpeg_thumbnail: String,
    #[serde(rename = "isAnimated")]
    pub is_animated: bool,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "forwardingScore")]
    pub forwarding_score: i64,
    #[serde(rename = "isForwarded")]
    pub is_forwarded: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct SendMessage {
    #[serde(rename = "chatId")]
    pub chat_id: String,
    pub message: String,
}
