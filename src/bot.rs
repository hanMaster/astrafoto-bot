use crate::config::config;
use crate::data_types::{Order, OrderMessage, RootMsg, SendMessage};
use dashmap::DashMap;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::time::SystemTime;

pub struct Bot {
    api_url: String,
    token: String,
    worker_url: String,
    admin_chat_id: String,
    paper: BTreeMap<String, Vec<String>>,
    paper_vec: Vec<String>,
    orders: DashMap<String, Order>,
}



impl Bot {
    pub fn new() -> Self {
        let paper: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let paper_vec = paper.iter().map(|p| p.0.to_string()).collect();
        Self {
            api_url: format!("{}/waInstance{}", &config().API_URL, &config().ID_INSTANCE),
            token: config().API_TOKEN_INSTANCE.to_string(),
            worker_url: config().WORKER_URL.to_string(),
            admin_chat_id: config().ADMIN_CHAT_ID.to_string(),
            paper,
            paper_vec,
            orders: DashMap::new(),
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
                                println!("{:#?}", self.orders);
                            } else {
                                let msg = self.handle_message(&mut m).await;

                                if msg.eq("ORDER_RECEIVED_MESSAGE") {
                                    let (_, order) =
                                        self.orders.remove(&m.body.sender_data.chat_id).unwrap();
                                    self.log_to_admin(format!("Заказ {}", order)).await;
                                    self.send_order(order).await;
                                }

                                println!("{:#?}", self.orders);
                                self.delete_notification(m.receipt_id).await;
                                self.send_message(m.body.sender_data.chat_id, msg).await;
                            }
                        }
                        Err(_) => {
                            println!("Новых сообщений нет");
                            // println!(
                            //     "{:#?} {}",
                            //     self.orders,
                            //     SystemTime::now()
                            //         .duration_since(UNIX_EPOCH)
                            //         .unwrap()
                            //         .as_secs()
                            // );
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
        let mut saved = self.orders.entry(chat_id.clone()).or_default();
        saved.chat_id = chat_id;
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
        let mut saved = self.orders.entry(chat_id.clone()).or_default();
        saved.chat_id = chat_id;
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

        match saved.state {
            "new" => {
                saved.state = "paper_requested";
                saved.iter_count = 1;
                saved.last_update_time = SystemTime::now();
                self.paper_prompt()
            }
            "paper_requested" => {
                let paper_type: usize = msg.parse().unwrap_or(0);
                if paper_type > 0 && paper_type <= self.paper_vec.len() {
                    saved.paper = self.paper_vec[paper_type - 1].clone();
                    saved.state = "size_requested";
                    saved.iter_count = 1;
                    saved.last_update_time = SystemTime::now();
                    let paper = saved.paper.clone();
                    self.size_prompt(&paper)
                } else {
                    self.paper_prompt()
                }
            }
            "size_requested" => {
                let size: usize = msg.parse().unwrap_or(0);
                let paper = saved.paper.clone();
                let sizes = sizes_vec(&self.paper, &paper);
                if size > 0 && size <= sizes.len() {
                    saved.size = sizes[size - 1].clone();
                    saved.state = "size_selected";
                    saved.iter_count = 1;
                    saved.last_update_time = SystemTime::now();
                    "READY".to_string()
                } else {
                    self.size_prompt(&paper)
                }
            }
            "size_selected" => {
                if msg.to_lowercase().eq("готово") && !saved.images.is_empty() {
                    "ORDER_RECEIVED_MESSAGE".to_string()
                } else {
                    "READY".to_string()
                }
            }

            _ => "Неопознанное состояние".to_string(),
        }
    }

    async fn maybe_need_ask(&mut self) {
        let mut orders_to_remove = vec![];
        for mut o in self.orders.iter_mut() {
            if !o.images.is_empty() && o.state.eq("new") {
                self.send_message(o.chat_id.clone(), self.paper_prompt())
                    .await;
                o.state = "paper_requested";
                o.iter_count = 1;
                o.last_update_time = SystemTime::now();
            } else if !o.images.is_empty()
                && (o.state.eq("paper_requested")
                    || o.state.eq("size_requested")
                    || o.state.eq("size_selected"))
            {
                if o.iter_count < 4 && o.last_update_time.elapsed().unwrap().as_secs() > 30 {
                    o.iter_count += 1;
                    o.last_update_time = SystemTime::now();

                    let msg = if o.state.eq("paper_requested") {
                        self.paper_prompt()
                    } else if o.state.eq("size_requested") {
                        self.size_prompt(&o.paper)
                    } else {
                        "READY".to_string()
                    };

                    self.send_message(o.chat_id.clone(), msg).await;
                } else if o.iter_count < 4 && o.last_update_time.elapsed().unwrap().as_secs() < 30 {
                } else {
                    orders_to_remove.push(o.chat_id.clone());
                }
            } else if o.images.is_empty() {
                // Delete order without images after 1 minute
                if o.last_update_time.elapsed().unwrap().as_secs() > 60 {
                    orders_to_remove.push(o.chat_id.clone());
                }
            }
        }
        for chat_id in orders_to_remove {
            self.orders.remove(&chat_id);
            self.send_message(
                chat_id.clone(),
                "Заказ отменен, из-за длительного ожидания".to_string(),
            )
            .await;
        }
    }

    fn paper_prompt(&self) -> String {
        self.paper_vec.iter().enumerate().fold(
            "Выберите тип бумаги: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {}", idx + 1, b);
                output
            },
        )
    }

    fn size_prompt(&self, paper: &str) -> String {
        sizes_vec(&self.paper, paper).iter().enumerate().fold(
            "Выберите размер фотографий: \n".to_string(),
            |mut output, (idx, b)| {
                let _ = writeln!(output, "{} - {b}", idx + 1);
                output
            },
        )
    }

    async fn send_order(&self, order: Order) {
        let send_result = reqwest::Client::new()
            .post(&self.worker_url)
            .json::<OrderMessage>(&order.into())
            .send()
            .await;
        match send_result {
            Ok(response) => {
                println!(
                    "Order sent successfully! Response: {}",
                    response.text().await.unwrap()
                );
            }
            Err(e) => {
                let msg = format!("Failed to send order to worker! Error: {}", e);
                eprintln!("{msg}");
                self.log_to_admin(msg).await;
            }
        }
    }
}

fn sizes_vec(p: &BTreeMap<String, Vec<String>>, paper: &str) -> Vec<String> {
    let s = vec![];
    p.get(paper)
        .unwrap_or(&s)
        .iter()
        .map(|p| p.to_string())
        .collect()
}


