use crate::config::config;
use crate::stuff::data_types::{OrderMessage, OrderState};
use crate::stuff::error::{Error, Result};
use crate::stuff::wa_types::SendMessage;
use log::error;
use reqwest::StatusCode;
pub trait Transport {
    fn send_message(&self, chat_id: String, msg: String) -> impl Future<Output = Result<()>> + Send;

    fn log_to_admin(&self, msg: String)-> impl Future<Output = ()> + Send;

    fn send_order(&self, order: OrderState) -> impl Future<Output = Result<String>> + Send;
}

#[derive(Clone)]
pub struct WhatsApp {
    api_url: String,
    token: String,
    admin_chat_id: String,
    worker_url: String,
}

impl WhatsApp {
    pub fn new() -> Self {
        Self {
            api_url: format!("{}/waInstance{}", &config().API_URL, &config().ID_INSTANCE),
            token: config().API_TOKEN_INSTANCE.to_owned(),
            admin_chat_id: config().ADMIN_CHAT_ID.to_owned(),
            worker_url: config().WORKER_URL.to_owned(),
        }
    }
}

impl Transport for WhatsApp {
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

    async fn log_to_admin(&self, msg: String) {
        let res = self.send_message(self.admin_chat_id.clone(), msg).await;
        if let Err(e) = res {
            error!("[log_to_admin] {:?}", e);
        }
    }

    async fn send_order(&self, order: OrderState) -> Result<String> {
        let send_result = reqwest::Client::new()
            .post(&self.worker_url)
            .json::<OrderMessage>(&order.clone().into())
            .send()
            .await;
        match send_result {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await?;
                if status == StatusCode::CREATED {
                    Ok(text)
                } else {
                    error!("[send_order] {:?}", text);
                    self.log_to_admin(text.clone()).await;
                    Err(Error::OrderFailed(text))
                }
            }
            Err(e) => {
                let msg = format!("Failed to send order to worker! Error: {}", e);
                error!("{msg}");
                self.log_to_admin(msg).await;
                Err(Error::Request(e))
            }
        }
    }
}

#[derive(Default)]
pub struct MockTransport;

impl Transport for MockTransport {
    async fn send_message(&self, chat_id: String, msg: String) -> Result<()> {
        println!("Sending message to: {}, {}", chat_id, &msg);
        Ok(())
    }

    async fn log_to_admin(&self, msg: String) {
        println!("Sending message to admin: {}", &msg);
    }

    async fn send_order(&self, order: OrderState) -> Result<String> {
        println!("Sending order to: {:?}", &order);
        Ok("".to_string())
    }
}
