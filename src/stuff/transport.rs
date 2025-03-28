use crate::config::config;
use crate::data_types::{RootMsg, SendMessage};
use crate::stuff::data_types::{Message, ReceivedMessage};
use crate::stuff::error::{Error, Result};
use reqwest::StatusCode;
pub trait Transport {
    async fn receive_message(&self) -> Result<Message>;
    async fn send_message(&self, chat_id: String, msg: String) -> Result<()>;
}

pub struct WhatsApp {
    api_url: String,
    token: String,
    admin_chat_id: String,
    timeout_seconds: u16,
}

impl WhatsApp {
    pub fn new() -> Self {
        Self {
            api_url: format!("{}/waInstance{}", &config().API_URL, &config().ID_INSTANCE),
            token: config().API_TOKEN_INSTANCE.to_owned(),
            admin_chat_id: config().ADMIN_CHAT_ID.to_owned(),
            timeout_seconds: 5,
        }
    }

    async fn delete_notification(&self, receipt_id: i64) {
        let url = format!(
            "{}/deleteNotification/{}/{}",
            self.api_url, self.token, receipt_id
        );

        let response = reqwest::Client::new().delete(&url).send().await;
        if let Err(e) = response {
            eprintln!("[delete_notification] {:?}", e);
            self.log_to_admin(e.to_string()).await;
        }
    }

    pub async fn log_to_admin(&self, msg: String) {
        let _ = self.send_message(self.admin_chat_id.clone(), msg).await;
    }
}

impl Transport for WhatsApp {
    async fn receive_message(&self) -> Result<Message> {
        let url = format!(
            "{}/receiveNotification/{}?receiveTimeout={}",
            self.api_url, self.token, self.timeout_seconds
        );
        let payload = reqwest::Client::new().get(&url).send().await?;

        match payload.status() {
            StatusCode::OK => {
                let msg_result = payload.json::<RootMsg>().await;
                match msg_result {
                    Ok(mut m) => {
                        self.delete_notification(m.receipt_id).await;
                        match m.body.message_data.type_message.as_ref() {
                            "imageMessage" => Ok(Message::Image(ReceivedMessage {
                                chat_id: m.body.sender_data.chat_id,
                                customer_name: m.body.sender_data.sender_name,
                                message: m
                                    .body
                                    .message_data
                                    .file_message_data
                                    .take()
                                    .unwrap()
                                    .download_url,
                            })),
                            "textMessage" => Ok(Message::Text(ReceivedMessage {
                                chat_id: m.body.sender_data.chat_id,
                                customer_name: m.body.sender_data.sender_name,
                                message: m
                                    .body
                                    .message_data
                                    .text_message_data
                                    .take()
                                    .unwrap()
                                    .text_message,
                            })),
                            _ => Ok(Message::Empty),
                        }
                    }
                    Err(_) => {
                        println!("Новых сообщений нет");
                        Ok(Message::Empty)
                    }
                }
            }
            _ => Err(Error::FailedToGetNewMessage(
                payload.status(),
                payload.text().await?,
            )),
        }
    }

    async fn send_message(&self, chat_id: String, message: String) -> Result<()> {
        let url = format!("{}/sendMessage/{}", &self.api_url, &self.token);
        let msg = SendMessage { chat_id, message };

        reqwest::Client::new()
            .post(&url)
            .json::<SendMessage>(&msg)
            .send()
            .await?;
        Ok(())
    }
}
