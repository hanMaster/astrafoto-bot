use crate::data_types::{Order, RootMsg, SendMessage};
use std::collections::HashMap;
use std::fmt::Write;

pub struct Bot {
    api_url: String,
    token: String,
    admin_chat_id: String,
    paper: Vec<String>,
    size: Vec<String>,
    orders: HashMap<String, Order>,
}

const READY: &str = "Если Вы загрузили все фотографии, то отправьте слово: Готово";

impl Bot {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let api_url = std::env::var("API_URL").expect("API_URL must be set");
        let id_instance = std::env::var("ID_INSTANCE").expect("ID_INSTANCE must be set");
        let api_token_instance =
            std::env::var("API_TOKEN_INSTANCE").expect("API_TOKEN_INSTANCE must be set");
        let admin_chat_id = std::env::var("ADMIN_CHAT_ID").expect("ADMIN_CHAT_ID must be set");
        Self {
            api_url: format!("{}/waInstance{}", api_url, id_instance),
            token: api_token_instance,
            admin_chat_id,
            paper: vec![
                "глянцевая".to_string(),
                "матовая".to_string(),
                "шелковая".to_string(),
            ],
            size: vec!["9x12".into(), "10x15".into()],
            orders: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.receive_notification().await;
        }
    }

    pub async fn receive_notification(&mut self) {
        let url = format!(
            "{}/receiveNotification/{}?receiveTimeout=5",
            self.api_url, self.token
        );

        let response = reqwest::Client::new().get(&url).send().await;
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let msg_result = response.json::<RootMsg>().await;
                    match msg_result {
                        Ok(mut m) => {
                            if m.body.message_data.type_message.eq("imageMessage") {
                                self.handle_image(&mut m).await;
                                self.delete_notification(m.receipt_id).await;
                            } else {
                                let msg = self.handle_message(&mut m).await;

                                if msg.eq("Ваш заказ принят") {
                                    let order =
                                        self.orders.get(&m.body.sender_data.chat_id).unwrap();
                                    self.log_to_admin(format!("Заказ {}", order)).await;
                                    self.orders.remove(&m.body.sender_data.chat_id);
                                }

                                println!("{:#?}", self.orders);
                                self.delete_notification(m.receipt_id).await;
                                self.send_message(m.body.sender_data.chat_id, msg).await;
                            }
                        }
                        Err(_) => {
                            println!("Новых сообщений нет");
                            self.maybe_need_ask().await;
                        }
                    }
                } else {
                    eprintln!("Green-API failed");
                }
            }
            Err(error) => {
                eprintln!("[receive_notification] {:?}", error);
                self.log_to_admin(error.to_string()).await;
            }
        }
    }

    pub async fn delete_notification(&mut self, receipt_id: i64) {
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

    pub async fn send_message(&self, chat_id: String, message: String) {
        let url = format!("{}/sendMessage/{}", &self.api_url, &self.token);
        let msg = SendMessage { chat_id, message };

        let res = reqwest::Client::new()
            .post(&url)
            .json::<SendMessage>(&msg)
            .send()
            .await;

        match res {
            Ok(res) => {
                if !res.status().is_success() {
                    eprintln!(
                        "[send_message] status: {:?} {:?}",
                        res.status(),
                        res.text().await
                    );
                }
            }
            Err(e) => {
                eprintln!("[send_message] {:?}", e);
            }
        }
    }

    pub async fn log_to_admin(&self, msg: String) {
        self.send_message(self.admin_chat_id.clone(), msg).await;
    }

    async fn handle_image(&mut self, message: &mut RootMsg) {
        let chat_id = message.body.sender_data.chat_id.clone();
        let saved = self.orders.entry(chat_id.clone()).or_default();
        saved.chat_id = chat_id.clone();
        saved.customer_name = message.body.sender_data.sender_name.clone();
        // safe to unwrap since message type is imageMessage
        let image_url = message
            .body
            .message_data
            .file_message_data
            .take()
            .unwrap()
            .download_url;
        saved.images.push(image_url);
    }

    async fn handle_message(&mut self, message: &mut RootMsg) -> String {
        let chat_id = message.body.sender_data.chat_id.clone();
        let saved = self.orders.entry(chat_id.clone()).or_default();
        saved.chat_id = chat_id.clone();
        saved.customer_name = message.body.sender_data.sender_name.clone();

        let msg = if message.body.message_data.type_message.eq("textMessage") {
            message
                .body
                .message_data
                .text_message_data
                .take()
                .unwrap()
                .text_message
        } else {
            "".to_string()
        };

        match saved.state.as_str() {
            "new" => {
                saved.state = "paper_requested".to_string();
                self.paper_prompt()
            }
            "paper_requested" => {
                let paper_type: usize = msg.parse().unwrap_or(0);
                if paper_type > 0 && paper_type <= self.paper.len() {
                    saved.paper = self.paper[paper_type - 1].clone();
                    saved.state = "size_requested".to_string();
                    self.size_prompt()
                } else {
                    self.paper_prompt()
                }
            }
            "size_requested" => {
                let size: usize = msg.parse().unwrap_or(0);
                if size > 0 && size <= self.size.len() {
                    saved.size = self.size[size - 1].clone();
                    saved.state = "size_selected".to_string();
                    READY.to_string()
                } else {
                    self.size_prompt()
                }
            }
            "size_selected" => {
                if (msg.eq("готово") | msg.eq("Готово")) && !saved.images.is_empty() {
                    "Ваш заказ принят".to_string()
                } else {
                    READY.to_string()
                }
            }

            _ => "Неопознанное состояние".to_string(),
        }
    }

    async fn maybe_need_ask(&mut self) {
        let mut clients_to_update: Vec<String> = vec![];
        for o in self.orders.values() {
            if !o.images.is_empty() && o.state.eq("new") {
                self.send_message(o.chat_id.clone(), self.paper_prompt())
                    .await;
                clients_to_update.push(o.chat_id.clone());
            } else if !o.images.is_empty() && o.state.eq("paper_requested") {
                self.send_message(o.chat_id.clone(), self.paper_prompt())
                    .await;
            } else if !o.images.is_empty() && o.state.eq("size_requested") {
                self.send_message(o.chat_id.clone(), self.size_prompt())
                    .await;
            } else if !o.images.is_empty() && o.state.eq("size_selected") {
                self.send_message(o.chat_id.clone(), READY.to_string())
                    .await;
            }
        }

        clients_to_update.iter().for_each(|c| {
            let order = self.orders.entry(c.clone()).or_default();
            order.state = "paper_requested".to_string();
        });
    }

    fn paper_prompt(&self) -> String {
        self.paper.iter().enumerate().fold(
            "Выберите тип бумаги: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {b}", idx + 1);
                output
            },
        )
    }

    fn size_prompt(&self) -> String {
        self.size.iter().enumerate().fold(
            "Выберите размер фотографий: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {b}", idx + 1);
                output
            },
        )
    }
}
