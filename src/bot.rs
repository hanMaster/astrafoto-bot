use crate::data_types::{Order, OrderMessage, RootMsg, SendMessage};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs::File;
use std::io::BufRead;

pub struct Bot {
    api_url: String,
    token: String,
    worker_url: String,
    admin_chat_id: String,
    paper: BTreeMap<String, Vec<String>>,
    paper_vec: Vec<String>,
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
        let worker_url = std::env::var("WORKER_URL").expect("WORKER_URL must be set");

        let paper = init_paper();
        let paper_vec = paper.iter().map(|p| p.0.to_string()).collect();
        Self {
            api_url: format!("{}/waInstance{}", api_url, id_instance),
            token: api_token_instance,
            worker_url,
            admin_chat_id,
            paper,
            paper_vec,
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
                                    let ord =
                                        self.orders.remove(&m.body.sender_data.chat_id).unwrap();
                                    self.send_order(ord).await;
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

        match saved.state.as_str() {
            "new" => {
                saved.state = "paper_requested".to_string();
                self.paper_prompt()
            }
            "paper_requested" => {
                let paper_type: usize = msg.parse().unwrap_or(0);
                if paper_type > 0 && paper_type <= self.paper_vec.len() {
                    saved.paper = self.paper_vec[paper_type - 1].clone();
                    saved.state = "size_requested".to_string();
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
                    saved.state = "size_selected".to_string();
                    READY.to_string()
                } else {
                    self.size_prompt(&paper)
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
                self.send_message(o.chat_id.clone(), self.size_prompt(&o.paper))
                    .await;
            } else if !o.images.is_empty() && o.state.eq("size_selected") {
                self.send_message(o.chat_id.clone(), READY.to_string())
                    .await;
            }
        }

        clients_to_update.iter().for_each(|c| {
            self.orders
                .entry(c.clone())
                .and_modify(|o| o.state = "paper_requested".to_string());
        });
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
                eprintln!("Order could not be sent: {}", e);
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

fn init_paper() -> BTreeMap<String, Vec<String>> {
    let lines = std::io::BufReader::new(
        File::open("paper.txt").expect("File paper.txt not found in working directory"),
    )
    .lines();

    let mut data: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for line in lines {
        if let Ok(line) = line {
            let parts = line.split(':').collect::<Vec<&str>>();
            if parts.len() != 2 {
                panic!(
                    "Ошибка формата файла paper.txt\nПример строки:\nглянцевая:10x15 - 22руб;13x18 - 30руб;15x21 - 36руб;15x23 - 40руб"
                );
            }
            let paper_name = parts[0].to_string();
            let sizes = parts[1]
                .split(";")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            data.insert(paper_name, sizes);
        }
    }
    data
}
